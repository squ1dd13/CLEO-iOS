//! Implements a new streaming system that allows resources to be swapped at runtime.
//!
//! We don't technically need to re-implement the whole system - we could write low-level code that
//! interfaces with the game a lot, but the streaming system is multithreaded and the game uses
//! global variables for all of it. Those two approaches don't work well together, and trying to
//! write Rust code to interface with that just means a lot of unsafe code.
//!
//! Since the system is fairly simple, we just implement a safer and more stable Rust-based
//! solution rather than attempting to work closely with game code.

use std::{
    collections::HashMap,
    fs::File,
    io,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use byteorder::{LittleEndian, ReadBytesExt};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::files::ModRes;

/// Type for sizes or offsets that are measured in sectors rather than bytes.
type Sectors = u32;

/// A request for a specific resource to be loaded from a stream.
struct Request {
    /// The offset of the resource in the archive in sectors.
    offset_sectors: Sectors,

    /// The slice into which bytes will be read from the archive file. This will be filled, so
    /// should be of the exact size required.
    output: &'static mut [u8],
}

/// Information about a file that is loaded as a replacement for a resource from an archive.
struct ResFile {
    /// The name of the file, including its extension. This is mainly included for logging purposes.
    name: String,

    /// The handle of the open file.
    ///
    /// We need to keep resource files open so that the user cannot modify them while the game is
    /// running, which could invalidate the file sizes we record when we load the streaming
    /// system. Those file sizes are used to tell the game how much memory to allocate for
    /// resources, so if they become invalid then we could end up with memory corruption.
    file: File,
}

/// A structure responsible for loading resources from a particular archive.
struct Stream {
    /// The name (and extension) of the archive file backing this stream.
    name: String,

    /// The archive file that this stream is in charge of.
    file: File,

    /// A sender for giving back the results of processed requests.
    sender: Sender<io::Result<Request>>,

    /// The receiving end of the channel used to send requests to this stream.
    receiver: Receiver<Request>,

    /// A map containing the offsets of resources and the files to read data for those resources
    /// from.
    ///
    /// A resource whose offset appears in this map will have its data loaded from the associated
    /// file instead of from the main archive file.
    swaps: HashMap<Sectors, ResFile>,
}

impl Stream {
    /// Creates a new stream for an archive at the given path, returning the stream and the
    /// sender that can be used to send requests to the stream.
    fn new(archive_path: &impl AsRef<Path>) -> io::Result<(Stream, StreamRemote)> {
        // Create the channel that other code can use to give us requests to process.
        let (remote_sender, receiver) = crossbeam_channel::unbounded();

        // Create the channel that we use to send back processed requests.
        let (sender, remote_receiver) = crossbeam_channel::unbounded();

        let mut stream = Stream {
            name: archive_path
                .as_ref()
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_lowercase(),
            file: File::open(archive_path)?,
            sender,
            receiver,
            swaps: HashMap::new(),
        };

        stream.load_swaps()?;

        Ok((
            stream,
            StreamRemote {
                sender: remote_sender,
                receiver: remote_receiver,
            },
        ))
    }

    /// Reads the table of contents from the archive file in order to populate the swap map.
    fn load_swaps(&mut self) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;

        // Read the file identifier.
        let ver2 = self.file.read_u32::<LittleEndian>()?;

        // Check against "VER2" (which is 4 bytes, so we use an integer to represent it).
        if ver2 != 0x32524556 {
            log::warn!("VER2 identifier not found at start of '{}'", self.name);
        }

        // Read the entry count so that we can read the whole TOC into memory.
        let entry_count = self.file.read_u32::<LittleEndian>()?;

        // Load the whole table of contents into memory to speed up reading.
        let mut toc_reader = {
            let toc_bytes = entry_count as usize * 32;

            let mut buf = vec![0; toc_bytes];
            self.file.read_exact(&mut buf)?;

            io::Cursor::new(buf)
        };

        // Find the resource files that replace entries in this archive.
        let swapped_names: HashMap<String, PathBuf> = super::res_iter()
            .filter_map(|item| match item {
                // Look for archive swap resources.
                ModRes::ArchSwap(img_name, path) if img_name == self.name => Some((
                    path.file_name().and_then(|s| s.to_str())?.to_lowercase(),
                    path,
                )),
                _ => None,
            })
            .collect();

        // Read the entries from the table of contents to find ones that we need to swap.
        for _ in 0..entry_count {
            let entry_offset: Sectors = toc_reader.read_u32::<LittleEndian>()?;
            let size: Sectors = toc_reader.read_u32::<LittleEndian>()?;

            let name = {
                // There are 24 bytes of space for the name, but it may end early with a null byte.
                let mut name_buf = [0; 24];
                toc_reader.read_exact(&mut name_buf)?;

                // Take the part of the name that comes before the null byte.
                let before_null: Vec<_> = name_buf.into_iter().take_while(|&b| b != 0).collect();
                String::from_utf8_lossy(&before_null).to_lowercase()
            };

            // Check if we're swapping this resource.
            let path = match swapped_names.get(&name) {
                Some(p) => p,
                None => continue,
            };

            let file = match File::open(path) {
                Ok(f) => f,
                Err(err) => {
                    // We just log errors and move on, because we don't want to stop further
                    // replacements loading.
                    log::error!(
                        "Failed to open resource replacement file '{}': {}",
                        path.display(),
                        err
                    );

                    continue;
                }
            };

            // Link the offset of the resource to the file we'll actually get the resource bytes
            // from.
            self.swaps.insert(entry_offset, ResFile { name, file });

            // todo: Ensure the game's model info for this resource has the correct size.
            // todo: Update the allocation size for the shared resource buffer to account for the
            //       replacement's size if it is larger than the current buffer size.
        }

        Ok(())
    }

    /// Attempts to process a single load request.
    ///
    /// If there are no requests waiting to be processed, this method will return `None`.
    fn proc_next(&mut self) -> Option<io::Result<Request>> {
        // Get a single request, if there is one. Return early if not.
        let request = match self.receiver.try_recv() {
            Ok(req) => req,
            Err(TryRecvError::Empty) => return None,

            // If the error isn't that the receiver is empty, there's something very wrong.
            Err(err) => panic!("error receiving request: {}", err),
        };

        Some(self.proc_req(request))
    }

    /// Processes the given load request in the context of the current stream.
    ///
    /// If successful, this method will return the request after loading the data into the output
    /// destination specified by the request. Upon failure, an error will be returned.
    fn proc_req(&mut self, request: Request) -> io::Result<Request> {
        // fixme: The game may attempt to load multiple contiguous resources in one go, so
        //  requests cannot expect to operate on a single-resource basis.

        // If the resource has been swapped, read from the file instead.
        if let Some(ResFile { name, file }) = self.swaps.get_mut(&request.offset_sectors) {
            // Make sure we're reading from the beginning of the resource file.
            file.seek(SeekFrom::Start(0))?;

            // Read the resource bytes. We don't use `read_exact` here because individual
            // resource files may not use sector boundaries, so it's entirely possible that a
            // valid swap would fail under `read_exact`.
            let bytes_read = file.read(request.output)?;

            let bytes_expected = request.output.len();
            let bytes_missing = bytes_expected - bytes_read;

            // If we're a whole sector off, then something might be up.
            if bytes_missing >= 2048 {
                log::warn!(
                    "Expected to read {} bytes from '{}', but only read {} ({} missing)",
                    bytes_expected,
                    name,
                    bytes_read,
                    bytes_missing
                );
            }

            return Ok(request);
        }

        // Seek to the location of the resource in the archive file.
        let offset_bytes = request.offset_sectors * 2048;
        self.file.seek(SeekFrom::Start(offset_bytes as u64))?;

        // Attempt to fill the output slice with the resource bytes.
        self.file.read_exact(request.output)?;

        // Return the fulfilled request.
        Ok(request)
    }
}

/// Channel endpoints used for communicating with a stream that lives on a different thread.
struct StreamRemote {
    /// A sender for giving the stream requests to process.
    sender: Sender<Request>,

    /// A receiver that yields the results of processed requests.  
    receiver: Receiver<io::Result<Request>>,
}

/// A manager for all of the currently-loaded streams. Responsible for routing requests and
/// sending loaded data back to the game.
struct Manager {
    /// Connections between the manager and streams that exist on another thread.
    remotes: Vec<StreamRemote>,

    /// A sender for giving newly-constructed streams to the thread that owns them.
    ///
    /// We use a channel for moving new streams from the thread that creates them to the thread
    /// that uses them because the background thread launches before most (if not all) streams
    /// are created. We need to be able to read from those new streams even though they did not
    /// exist when the looping thread was created, so we buffer newly-created streams and allow
    /// the background thread to take them when it's ready.
    stream_sender: Sender<Stream>,

    /// The receiver that yields any new streams that have been sent using `stream_sender`.
    ///
    /// This should be `None` all the time except for between the manager's construction and when
    /// the background thread starts running, as the background thread takes this receiver.
    stream_receiver: Option<Receiver<Stream>>,
}

impl Manager {
    fn shared_mut<'mgr>() -> parking_lot::MutexGuard<'mgr, Manager> {
        static SHARED: Lazy<Mutex<Manager>> = Lazy::new(|| {
            let (stream_sender, stream_receiver) = crossbeam_channel::unbounded();

            Mutex::new(Manager {
                remotes: vec![],
                stream_sender,
                stream_receiver: Some(stream_receiver),
            })
        });

        SHARED.lock()
    }

    /// Schedules the given stream for processing on the streaming thread, and stores the given
    /// sender so that load requests to this stream may take place immediately.
    ///
    /// Requests sent to this stream will be buffered until the background thread starts (if it
    /// hasn't already started).
    fn add_stream(&mut self, stream: Stream, remote: StreamRemote) {
        self.stream_sender
            .send(stream)
            .expect("New stream send failed");

        self.remotes.push(remote);
    }

    /// Start looking for requests for the streams to process on a background thread.
    fn start_thread(&mut self) {
        log::info!("Loading streams");

        // Consume the stream receiver so we can use it on the background thread.
        let stream_receiver = self
            .stream_receiver
            .take()
            .expect("Streaming thread cannot start without stream receiver");

        // Collect any streams that are already waiting.
        let mut streams: Vec<_> = stream_receiver.try_iter().collect();

        // Start a thread where we wait for load requests and act on them as they come.
        // fixme: Constantly looping while waiting for new requests is wasteful.
        std::thread::spawn(move || {
            log::info!("Stream thread started");

            loop {
                // Process a waiting request from each stream that has one, and send the result from
                // each back to the thread that wants the resources. Two requests for the same stream
                // will never be handled one after another unless no other streams have requests
                // waiting - every other stream gets a chance to handle a request before the second
                // request from a single stream is processed.
                for proc_output in streams.iter_mut().filter_map(Stream::proc_next) {
                    if let Err(err) = proc_output {
                        log::error!("Error processing resource request: {}", err);
                    }
                }

                // Add any new streams to the vector so we can iterate over them as well.
                for new_stream in stream_receiver.try_iter() {
                    streams.push(new_stream);
                }
            }
        });
    }

    /// Block the calling thread until the given stream produces a load result.
    fn wait_for_result(&self, stream_index: usize) -> io::Result<Request> {
        // fixme: This will block indefinitely if the stream does not have any requests to process.

        self.remotes[stream_index]
            .receiver
            .recv()
            .expect("Failed to receive result")
    }

    /// Send a request to the stream with the given index.
    fn send_req(&mut self, stream_index: usize, request: Request) {
        self.remotes[stream_index].sender.send(request).unwrap();
    }
}

/// Represents the location of a resource across the whole streaming system by combining the
/// image index with the resource's offset within that image file.
struct ResLoc(u32);

impl ResLoc {
    /// Creates a new resource location from an index and offset.
    fn new(image_index: u8, sector_offset: u32) -> ResLoc {
        ResLoc(((image_index as u32) << 24) | sector_offset)
    }

    /// Returns the index of the image that this resource is found in.
    fn image_index(&self) -> u8 {
        // Upper 8 bits.
        (self.0 >> 24) as u8
    }

    /// Returns the offset of the resource from the beginning of its parent image file.
    fn offset(&self) -> Sectors {
        // Lower 24 bits.
        self.0 & 0xffffff
    }
}

crate::declare_hook!(
    /// Reads the table of contents from an archive and registers the resources found with
    /// the appropriate systems.
    LOAD_ARCHIVE_TABLE,
    fn(path: *const u8, archive_id: i32),
    0x1002f0e18
);

fn load_archive_table(path: *const u8, archive_id: i32) {
    LOAD_ARCHIVE_TABLE.original()(path, archive_id);
}

crate::declare_hook!(
    /// Sets up the streaming system with the given number of streams and starts the
    /// streaming thread.
    INIT_STREAMS,
    fn(count: i32),
    0x100177eb8
);

fn init_streams(_count: i32) {
    Manager::shared_mut().start_thread();
}

crate::declare_hook!(
    /// Requests that a particular resource is loaded from a stream.
    STREAM_READ,
    fn(stream_index: u32, out_buf: *mut u8, location: ResLoc, size: Sectors) -> bool,
    0x100178048
);

fn stream_read(stream_index: u32, out_buf: *mut u8, location: ResLoc, size: Sectors) -> bool {
    let request = Request {
        offset_sectors: location.offset(),
        output: unsafe { std::slice::from_raw_parts_mut(out_buf, size as usize * 2048) },
    };

    Manager::shared_mut().send_req(stream_index as usize, request);

    // The game returns `false` for rejected reads. We never reject reads, so always return `true`.
    true
}

crate::declare_hook!(
    /// Opens an archive file and adds the file handle to the global image handle array.
    /// Returns the location of the first resource in the image file.
    STREAM_OPEN,
    fn(path: *const u8) -> ResLoc,
    0x1001782b0
);

fn stream_open(path_str: *const u8) -> ResLoc {
    let path_str =
        unsafe { std::ffi::CStr::from_ptr(path_str as *const libc::c_char).to_string_lossy() };

    let absolute_path = super::loader::find_absolute_path(&path_str.to_lowercase())
        .expect("Unable to resolve stream file path");

    log::info!("Adding path {} to stream manager", absolute_path);

    let path = PathBuf::from(absolute_path);

    let (stream, remote) = Stream::new(&path).expect("Failed to create new stream");

    let mut manager = Manager::shared_mut();
    manager.add_stream(stream, remote);

    ResLoc::new(manager.remotes.len() as u8, 0)
}

crate::declare_hook!(
    /// Blocks the calling thread while waiting for the given stream to finish loading something.
    WAIT_FOR_RESPONSE,
    fn(stream_index: u32) -> u32,
    0x100178178
);

pub fn init() {
    LOAD_ARCHIVE_TABLE.install(load_archive_table);
    INIT_STREAMS.install(init_streams);
    STREAM_READ.install(stream_read);
    STREAM_OPEN.install(stream_open);
    WAIT_FOR_RESPONSE.install(|stream_index: u32| {
        Manager::shared_mut()
            .wait_for_result(stream_index as usize)
            .unwrap();
        0u32
    });
}
