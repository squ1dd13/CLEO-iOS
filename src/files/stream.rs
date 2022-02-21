//! Implements a new streaming system that allows resources to be swapped at runtime.
//!
//! We don't technically need to re-implement the whole system - we could write low-level code that
//! interfaces with the game a lot, but the streaming system is multithreaded and the game uses
//! global variables for all of it. Those two approaches don't work well together, and trying to
//! write Rust code to interface with that just means a lot of unsafe code.
//!
//! Since the system is fairly simple, we just implement a safer and more stable Rust-based
//! solution rather than attempting to work closely with game code.

use std::collections::HashMap;
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use byteorder::{LittleEndian, ReadBytesExt};
use crossbeam_channel::{Receiver, Sender, TryRecvError};

use crate::files::ModRes;

/// Identifies a single stream to direct a request to.
struct StreamId {
    // Stuff
}

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
    fn new(archive_path: &impl AsRef<Path>) -> io::Result<(Stream, Sender<Request>)> {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let mut stream = Stream {
            name: archive_path
                .as_ref()
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_lowercase(),
            file: File::open(archive_path)?,
            receiver,
            swaps: HashMap::new(),
        };

        stream.load_swaps()?;

        Ok((stream, sender))
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
    /// Otherwise, the request that was processed will be returned after the requested resource
    /// has been loaded into its output slice.
    fn proc_next(&mut self) -> io::Result<Option<Request>> {
        // Get a single request, if there is one. Return early if not.
        let request = match self.receiver.try_recv() {
            Ok(req) => req,
            Err(TryRecvError::Empty) => return Ok(None),

            // If the error isn't that the receiver is empty, there's something very wrong.
            Err(err) => panic!("error receiving request: {}", err),
        };

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

            return Ok(Some(request));
        }

        // Seek to the location of the resource in the archive file.
        let offset_bytes = request.offset_sectors * 2048;
        self.file.seek(SeekFrom::Start(offset_bytes as u64))?;

        // Attempt to fill the output slice with the resource bytes.
        self.file.read_exact(request.output)?;

        // Return the fulfilled request.
        Ok(Some(request))
    }
}

/// A manager for all of the currently-loaded streams. Responsible for routing requests and
/// sending loaded data back to the game.
struct StreamPool {
    /// The streams that this pool controls.
    streams: Vec<Stream>,
}

impl StreamPool {}
