use std::{ffi::CStr, io::Read};

use eyre::{Context, Result};

use crate::hook;

use super::{
    game::{FilePointer, GlobalStreamQueue, ImageRegion, Stream, StreamSource},
    load::ReplacementMapper,
};

/// Represents the combination of an image file handle and a name.
struct Image {
    /// The file handle for this image.
    handle: &'static mut ImageHandle,

    /// The character array for this image's name.
    name_bytes: &'static mut [u8; 64],
}

impl Image {
    /// Clears all image data from memory.
    fn clear_all() {
        // Clear the image handles.
        for handle in ImageHandle::handles() {
            *handle = ImageHandle(FilePointer::null());
        }

        // Clear the names.
        for name_bytes in Image::image_name_bytes() {
            name_bytes[0] = 0;
        }
    }

    /// Sets the image's handle to `file_handle`.
    fn set_file_handle(&mut self, handle: FilePointer) {
        *self.handle = ImageHandle(handle);
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
    fn open_file(path: impl AsRef<CStr>) -> Result<FilePointer> {
        // Create a new handle by opening the file.
        let open_result = FilePointer::open(path.as_ref(), 0, 0);

        open_result.wrap_err_with(|| {
            format!(
                "When opening archive image '{}'",
                path.as_ref().to_str().unwrap_or("<unrepresentable>")
            )
        })
    }

    /// Attempts to open the archive file at `path`, returning the index of the resulting image
    /// data in the game's image array.
    fn add_archive(path: impl AsRef<CStr>) -> Result<usize> {
        // Find the first empty image slot, or return an error.
        let (index, mut image) = Image::images_mut()
            .enumerate()
            .find(|(_, Image { handle, .. })| handle.file_handle() == FilePointer::null())
            .ok_or(eyre::format_err!(
                "Cannot add new image file because there are no free image slots"
            ))?;

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

    /// Returns the name of the image.
    fn name(&self) -> &str {
        let windows_path = CStr::from_bytes_until_nul(self.name_bytes)
            .unwrap()
            .to_str()
            .expect("Invalid UTF8 in image name");

        windows_path.split('\\').last().unwrap()
    }
}

/// An image file handle.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImageHandle(FilePointer);

impl ImageHandle {
    /// Returns a slice of mutable references to the game's image handles.
    fn handles() -> &'static mut [ImageHandle] {
        let image_handle_ptrs: *mut ImageHandle = hook::slide(0x100939140);

        unsafe { std::slice::from_raw_parts_mut(image_handle_ptrs, 8) }
    }

    /// Returns the `Image` that this handle is from.
    fn image(&self) -> Image {
        Image::images_mut()
            .find(|Image { handle, .. }| *handle == self)
            .expect("No image with matching handle")
    }

    /// Returns a mutable reference to the underlying file handle.
    fn file_handle(&self) -> FilePointer {
        self.0
    }
}

impl Stream {
    /// Returns the path to the file that should be used instead of the image when reading
    /// `region`, or `None` if this region is not swapped.
    fn region_swap(&self, region: ImageRegion) -> Option<std::path::PathBuf> {
        // Get the image handle and use it to find the image.
        let image = ImageHandle(self.image_file).image();

        // Consult the shared replacement mapper to find the swap path.
        ReplacementMapper::shared()
            .replacement_path(image.name(), region)
            .cloned()
    }

    /// Loads all of the data from `path` into the stream's buffer. It is **very important** that
    /// the buffer is at least as large as the file.
    fn load_from_file(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        log::info!("loading from file {:?}", path.as_ref());

        // We use the region size instead of the file size because we need to make sure the full
        // region's worth of valid data is delivered into the buffer, not just the file size's
        // worth. Any bytes within the region size but after the end of the file can just be zero.
        let read_size = self.file_region().size_bytes();

        // Get a slice from the buffer so that we can manipulate it more easily.
        let buf_slice = unsafe { std::slice::from_raw_parts_mut(self.buffer, read_size) };

        // Read from the file into our buffer.
        let written = std::fs::File::open(path)?.read(buf_slice)?;

        // Fill any remaining space in the buffer with zeros.
        buf_slice[written..].fill(0);

        Ok(())
    }

    /// Loads the data from `region` in the stream's image file into the stream's buffer.
    fn load_region(&mut self, region: ImageRegion) -> Result<()> {
        // First check if there's a file that we're using to replace the data from this region.
        if let Some(swap_file_path) = self.region_swap(region) {
            // There is, so load from the file instead of the image.
            return self.load_from_file(swap_file_path);
        }

        // Seek to the start of the requested region in the image file.
        let seek_err = self.image_file.seek_to(region.offset_bytes());

        if seek_err != 0 {
            return Err(eyre::format_err!(
                "Unable to seek to offset {}",
                region.offset_bytes()
            ));
        }

        // Fill the buffer with all of the data from the requested region.
        let read_err = self.image_file.read(self.buffer, region.size_bytes());

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
    fn load_request(&mut self) -> Result<()> {
        self.load_region(self.file_region())
    }

    /// Handles a request for data on this stream.
    fn handle_current_request(&mut self) -> Result<()> {
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
        image: ImageHandle,
        region: ImageRegion,
        buffer: *mut u8,
    ) -> Result<()> {
        if self.is_busy() {
            return Err(eyre::Error::msg("Stream is busy"));
        }

        self.clear_error();

        self.image_file = image.file_handle();
        self.set_region(region);

        self.buffer = buffer;
        self.in_use = false;

        Ok(())
    }
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
    let handle = ImageHandle::handles()[image_index];

    let request_result = stream.request_load(handle, region, buffer);

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
