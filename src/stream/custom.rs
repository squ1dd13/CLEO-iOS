use std::{ffi::CStr, io::Read};

use byteorder::{LittleEndian, ReadBytesExt};
use eyre::Context;

use crate::hook;

use super::game::{FileHandle, GlobalStreamQueue, ImageRegion, Stream, StreamSource};

/// Represents the combination of an image file handle and a name.
struct Image {
    /// The file handle for this image.
    handle: &'static mut Option<&'static mut ImageHandle>,

    /// The character array for this image's name.
    name_bytes: &'static mut [u8; 64],
}

impl Image {
    /// Clears all image data from memory.
    fn clear_all() {
        // Clear the image handles.
        for handle in ImageHandle::handles() {
            *handle = None;
        }

        // Clear the names.
        for name_bytes in Image::image_name_bytes() {
            name_bytes[0] = 0;
        }
    }

    /// Sets the image's handle to `file_handle`.
    fn set_file_handle(&mut self, handle: &'static mut FileHandle) {
        // SAFETY: We can transmute here because `Image` and `FileHandle` are exactly the same
        // thing, and we only use different types in order to define different methods for
        // them.
        let image_handle =
            unsafe { std::mem::transmute::<&'_ mut FileHandle, &'_ mut ImageHandle>(handle) };

        *self.handle = Some(image_handle);
    }

    /// Sets the image's name to `name`
    fn set_name(&mut self, name: impl AsRef<CStr>) {
        // Get the bytes from the path string, including the null pointer so after the copy the
        // string terminates in the right place.
        let path_chars = name.as_ref().to_bytes_with_nul();

        // Copy the image name into game memory.
        self.name_bytes[..path_chars.len()].copy_from_slice(path_chars);
    }

    /// Attempts to open the image at `path`, returning the resulting file handle.
    fn open_file(path: impl AsRef<CStr>) -> eyre::Result<&'static mut FileHandle> {
        // Create a new handle by opening the file.
        let open_result = FileHandle::open(path.as_ref(), 0, 0);

        open_result.wrap_err_with(|| {
            format!(
                "When opening archive image '{}'",
                path.as_ref().to_str().unwrap_or("<unrepresentable>")
            )
        })
    }

    /// Attempts to open the archive file at `path`, returning the index of the resulting image
    /// data in the game's image array.
    fn add_archive(path: impl AsRef<CStr>) -> eyre::Result<usize> {
        // Find the first empty image slot, or return an error.
        let (index, mut image) = Image::images_mut()
            .enumerate()
            .find(|(_, Image { handle, .. })| handle.is_none())
            .ok_or(eyre::format_err!(
                "Cannot add new image file because there are no free image slots"
            ))?;

        // Open the file.
        let opened_handle = Image::open_file(&path)?;

        // Update the image data.
        image.set_file_handle(opened_handle);
        image.set_name(path);

        Ok(index)
    }

    /// Returns an iterator over the game's images.
    fn images_mut() -> impl Iterator<Item = Image> {
        ImageHandle::handles()
            .iter_mut()
            .zip(Image::image_name_bytes().iter_mut())
            .map(|(handle, name_bytes)| Image { handle, name_bytes })
    }

    /// Returns a slice of mutable references to the game's image name character arrays.
    fn image_name_bytes() -> &'static mut [[u8; 64]] {
        let image_name_arr: *mut [u8; 64] = hook::slide(0x1006ac0e0);

        unsafe { std::slice::from_raw_parts_mut(image_name_arr, 32) }
    }

    /// Returns an iterator over the game's image name array.
    fn image_names() -> impl Iterator<Item = &'static CStr> {
        Image::image_name_bytes()
            .iter_mut()
            .map(|bytes| CStr::from_bytes_until_nul(bytes).expect("Invalid image name in array"))
    }
}

/// An image file handle.
#[derive(Debug)]
struct ImageHandle(FileHandle);

impl ImageHandle {
    /// Returns a slice of mutable references to the game's image handles.
    fn handles() -> &'static mut [Option<&'static mut ImageHandle>] {
        let image_handle_ptrs: *mut Option<&'static mut ImageHandle> = hook::slide(0x100939140);

        unsafe { std::slice::from_raw_parts_mut(image_handle_ptrs, 8) }
    }

    /// Returns a mutable reference to the underlying file handle.
    fn file_handle_mut(&mut self) -> &mut FileHandle {
        &mut self.0
    }
}

impl Stream {
    /// Loads the data from `region` in the stream's image file into the stream's buffer.
    fn load_region(&mut self, region: ImageRegion) -> eyre::Result<()> {
        let image_file = self.image_file.as_mut().expect("Stream has no image file");

        // Seek to the start of the requested region in the image file.
        let seek_err = image_file.seek_to(region.offset_bytes());

        if seek_err != 0 {
            return Err(eyre::format_err!(
                "Unable to seek to offset {}",
                region.offset_bytes()
            ));
        }

        // Fill the buffer with all of the data from the requested region.
        let read_err = image_file.read(self.buffer, region.size_bytes());

        if read_err != 0 {
            Err(eyre::format_err!(
                "Unable to read {} bytes at {}",
                region.size_bytes(),
                region.offset_bytes()
            ))
        } else {
            Ok(())
        }
    }

    /// Locates the data requested and copies it into the stream's output buffer.
    fn load_request(&mut self) -> eyre::Result<()> {
        self.load_region(self.file_region())
    }

    /// Handles a request for data on this stream.
    fn handle_current_request(&mut self) -> eyre::Result<()> {
        self.enter_processing_state();

        // Load the requested data if the previous read was successful.
        let result = if self.is_ok() {
            self.load_request()
        } else {
            Ok(())
        };

        if result.is_err() {
            self.status = 0xfe;
        }

        self.exit_processing_state();

        result
    }

    /// Requests that the stream loads the data from `region` from `image` into `buffer`. Returns
    /// an error if the stream is currently busy.
    fn request_load(
        &mut self,
        image: &'static mut ImageHandle,
        region: ImageRegion,
        buffer: *mut u8,
    ) -> eyre::Result<()> {
        if self.is_busy() {
            log::trace!("not ready");
            return Err(eyre::Error::msg("Stream is busy"));
        }

        self.clear_error();

        self.image_file = Some(image.file_handle_mut());
        self.set_region(region);

        self.buffer = buffer;
        self.in_use = false;

        Ok(())
    }
}

/// Describes an individual file within an archive.
struct DirectoryEntry {
    /// The name of the file.
    name: String,

    /// The region of the parent image containing the file's data.
    region: ImageRegion,
}

impl DirectoryEntry {
    /// Reads a single directory entry from `reader`, using `name_buf` as a temporary buffer for
    /// copying name characters into.
    fn read(
        reader: &mut impl std::io::BufRead,
        name_buf: &mut [u8; 24],
    ) -> eyre::Result<DirectoryEntry> {
        let region = ImageRegion {
            offset_sectors: reader.read_u32::<LittleEndian>()? as usize,

            // There are actually two u16 values here, streaming size and archived size, but the
            // streaming size is unused, so we can actually read the archived size as a u32.
            size_sectors: reader.read_u32::<LittleEndian>()? as usize,
        };

        // Read the name characters into our byte buffer.
        reader.read_exact(name_buf)?;

        // Most names will not use all 24 bytes, so will have a null terminator before index
        // 23. We can use `CStr` to parse null-terminated strings, which will work fine for
        // most cases. However, that won't work if all 24 bytes have been used, in which case
        // we try to directly create a `str` from the bytes.
        let name = CStr::from_bytes_until_nul(name_buf).map_or_else(
            |_| {
                // Couldn't parse as a null-terminated C string, so assume all 24 bytes have
                // been used. In this case, we can convert the entire slice.
                std::str::from_utf8(name_buf)
            },
            // If the name parsed as a C string, convert that C string to a string slice.
            |c_string| c_string.to_str(),
        )?;

        let name = name.to_string();

        Ok(DirectoryEntry { name, region })
    }

    /// Attempts to read `count` directory entries from `reader`.
    fn read_entries(
        count: usize,
        reader: &mut impl std::io::BufRead,
    ) -> eyre::Result<impl Iterator<Item = eyre::Result<DirectoryEntry>> + '_> {
        // Create a single buffer for temporarily holding the bytes of each name entry during
        // processing.
        let mut name_buf = [0u8; 24];

        let read_entry = move |_| -> eyre::Result<DirectoryEntry> {
            DirectoryEntry::read(reader, &mut name_buf)
        };

        Ok((0..count).map(read_entry))
    }
}

fn load_archive_file(path: &str, image_id: i32) -> eyre::Result<()> {
    // Use a buffered reader because we do a lot of small sequential reads.
    let mut reader = std::io::BufReader::new(std::fs::File::open(path)?);

    let mut identifier = [0u8; 4];
    reader.read_exact(&mut identifier)?;

    if &identifier != b"VER2" {
        return Err(eyre::format_err!(
            "Bad identifier in archive '{}': {:?} (expected 'VER2')",
            path,
            identifier
        ));
    }

    let entry_count = reader.read_u32::<LittleEndian>()? as usize;

    let entries = DirectoryEntry::read_entries(entry_count, &mut reader)?;

    Ok(())
}

/// Launches the streaming thread with our custom behaviour.
fn launch_streaming_thread() {
    // Get a function pointer for `OS_ThreadLaunch`.
    let launch =
        hook::slide::<fn(fn(usize), usize, u32, *const u8, i32, u32) -> *mut u8>(0x1004e8888);

    // Launch the thread using our own function instead of the game's streaming thread function. I
    // don't know what all of the parameters for `OS_ThreadLaunch` are, and I can't be bothered to
    // find out. The "unknown" values are just what the game uses.
    let thread = launch(
        streaming_thread_hook,
        0x0,                      // unknown
        3,                        // unknown
        hook::slide(0x10058a2eb), // name - "CdStream" here
        0,                        // unknown
        3,                        // priority
    );

    if thread.is_null() {
        panic!("Failed to start streaming thread!");
    }

    let global_stream_thread: *mut *mut u8 = hook::slide(0x100939138);

    unsafe {
        global_stream_thread.write(thread);
    }
}

/// Sets up the streaming system ready for images to be loaded.
fn init_streams(count: usize) {
    Image::clear_all();
    Stream::create_streams(count);
    GlobalStreamQueue::init();

    launch_streaming_thread();
}

/// Waits for requests for data from the streams and handles them.
fn poll_stream_queue() {
    log::info!("Polling for stream requests...");

    let mut queue = GlobalStreamQueue::global();

    loop {
        // Handle stream requests as they arrive.
        let stream = queue.pop_blocking();

        if let Err(err) = stream.handle_current_request() {
            log::error!("Streaming error: {:?}", err);
        }
    }
}

/// Requests a resource load from a specific stream and image, returning `false` if the selected
/// stream is not ready for a new request.
fn try_stream_request(
    stream_index: usize,
    image_index: usize,
    buffer: *mut u8,
    region: ImageRegion,
) -> bool {
    Stream::set_global_position(image_index, region);

    let stream = &mut Stream::streams()[stream_index];
    let image = ImageHandle::handles()[image_index]
        .as_mut()
        .expect("Image not loaded");

    let request_result = stream.request_load(image, region, buffer);

    if request_result.is_err() {
        return false;
    }

    let mut queue = GlobalStreamQueue::global();
    queue.push_index(stream_index);

    true
}

/// Hook for `CdStreamInit`.
fn stream_init_hook(count: i32) {
    init_streams(count as usize);
}

/// Replacement for the game's streaming thread function. This function is given to game code to
/// run on a new thread when the streaming system has been initialised.
fn streaming_thread_hook(_: usize) {
    poll_stream_queue();
}

/// Hook for `CdStreamRead`.
fn stream_read_hook(
    stream_index: u32,
    buffer: *mut u8,
    source: StreamSource,
    sector_count: u32,
) -> bool {
    try_stream_request(
        stream_index as usize,
        source.image_index() as usize,
        buffer,
        ImageRegion {
            offset_sectors: source.sector_offset() as usize,
            size_sectors: sector_count as usize,
        },
    )
}

/// Hook for `CdStreamOpen`.
fn stream_open_hook(path: *const i8, _: bool) -> i32 {
    // Try to add the archive.
    let index = match Image::add_archive(unsafe { CStr::from_ptr(path) }) {
        Ok(index) => index,
        Err(err) => {
            log::error!("Couldn't add image file: {:?}", err);
            return 0;
        }
    };

    // Create and return the stream position for the start of the image.
    let start = StreamSource::new(index as u8, 0);

    start.as_u32() as i32
}

/// Hooks the streaming system.
pub fn hook() {
    const CD_STREAM_INIT: hook::Target<fn(i32)> = hook::Target::Address(0x100177eb8);
    CD_STREAM_INIT.hook_hard(stream_init_hook);

    type StreamReadFn = fn(u32, *mut u8, StreamSource, u32) -> bool;
    const CD_STREAM_READ: hook::Target<StreamReadFn> = hook::Target::Address(0x100178048);
    CD_STREAM_READ.hook_hard(stream_read_hook);

    const CD_STREAM_OPEN: hook::Target<fn(*const i8, bool) -> i32> =
        hook::Target::Address(0x1001782b0);
    CD_STREAM_OPEN.hook_hard(stream_open_hook);
}
