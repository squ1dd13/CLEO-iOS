use std::{collections::HashMap, ffi::CStr, io::Read};

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

    /// Returns the name of the image.
    fn name(&self) -> &str {
        CStr::from_bytes_until_nul(self.name_bytes)
            .unwrap()
            .to_str()
            .expect("Invalid UTF8 in image name")
    }

    /// Returns a reference to a map containing the regions of this image file that should be read
    /// from external files, and the external file that should be used for each.
    fn region_swaps(&self) -> Option<&'static HashMap<ImageRegion, std::path::PathBuf>> {
        let name = self.name();
        log::trace!("name = {}", name);

        super::load::region_swaps_for_image_name(name)
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

    /// Converts the given file handle to an image handle.
    fn from_file(file_handle: &'static FileHandle) -> &'static ImageHandle {
        // SAFETY: Image handles _are_ file handles (we just treat them differently so we can
        // define image-specific methods), so this is fine.
        unsafe { std::mem::transmute(file_handle) }
    }

    /// Returns the `Image` that this handle is from.
    fn image(&self) -> Image {
        // log::trace!("this addr = {:#x}", self as *const _ as usize);

        // for image in Image::images_mut() {
        //     log::trace!("that addr = {:#x}", unsafe {
        //         *std::mem::transmute::<_, *const usize>(image.handle)
        //     });
        // }

        Image::images_mut()
            .find(|Image { handle, .. }| {
                handle
                    .as_ref()
                    .map(|handle| std::ptr::eq(*handle as *const _, self))
                    .unwrap_or(false)
            })
            .expect("No images match handle")
    }

    /// Returns a mutable reference to the underlying file handle.
    fn file_handle_mut(&mut self) -> &mut FileHandle {
        &mut self.0
    }
}

impl Stream {
    /// Returns the path to the file that should be used instead of the image when reading
    /// `region`, or `None` if this region is not swapped.
    fn region_swap(&self, region: ImageRegion) -> Option<&'static std::path::PathBuf> {
        // hack: This...
        let imgf =
            unsafe { *std::mem::transmute::<_, &Option<&'static ImageHandle>>(&self.image_file) };

        // Get the image handle and use it to find the image.
        let image = imgf?.image();

        let swaps = image.region_swaps();

        if swaps.is_none() {
            log::trace!("no swaps for {}", image.name());
            return None;
        }

        let swaps = swaps.unwrap();

        log::trace!("{} swaps", swaps.len());

        // The image can then give us the region swap map.
        swaps.get(&region)
    }

    /// Loads all of the data from `path` into the stream's buffer. It is **very important** that
    /// the buffer is at least as large as the file.
    fn load_from_file(&mut self, path: impl AsRef<std::path::Path>) -> eyre::Result<()> {
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
    fn load_region(&mut self, region: ImageRegion) -> eyre::Result<()> {
        // First check if there's a file that we're using to replace the data from this region.
        if let Some(swap_file_path) = self.region_swap(region) {
            // There is, so load from the file instead of the image.
            return self.load_from_file(swap_file_path);
        }

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
