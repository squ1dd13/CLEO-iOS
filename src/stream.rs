//! Replaces parts of the game's streaming system to allow the loading of replacement files inside IMGs,
//! and also manages the loaded replacements.

// hack: The `stream` module is messy, poorly documented and full of hacky code.
// bug: Opcode 0x04ee seems to break when animations have been swapped.

use std::{
    collections::HashMap,
    ffi::CStr,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::Mutex,
};

use byteorder::{LittleEndian, ReadBytesExt};
use eyre::Context;
use libc::c_char;

use crate::{call_original, hook, targets};

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

/// Opaque type used to refer to semaphores used by the game.
#[derive(Debug)]
struct Semaphore;

impl Semaphore {
    /// Creates a new semaphore using game code and returns a mutable reference to it.
    fn new_mut() -> &'static mut Semaphore {
        hook::slide::<fn() -> &'static mut Semaphore>(0x1004e8b18)()
    }

    /// Blocks until the semaphore count becomes greater than zero, then decrements it and returns.
    fn wait(&mut self) {
        hook::slide::<fn(&mut Semaphore)>(0x1004e8b84)(self);
    }

    /// Increments the semaphore count.
    fn post(&mut self) {
        hook::slide::<fn(&mut Semaphore)>(0x1004e8b5c)(self);
    }
}

/// A queue of streams.
///
/// When data is requested from a stream, the stream is added to the queue and the stream queue
/// semaphore is incremented.
struct GlobalStreamQueue {
    /// The global stream queue semaphore.
    semaphore_ref: &'static mut Semaphore,

    /// The global queue of stream indices that represent the streams in this queue.
    index_queue_ref: &'static mut Queue,
}

impl GlobalStreamQueue {
    /// Initialises the global queue.
    fn init() {
        let queue = Queue::with_capacity(Stream::global_count() + 1);
        let semaphore = Semaphore::new_mut();

        let global_queue_ptr = hook::slide::<*mut Queue>(0x100939120);
        let global_semaphore_loc_ptr: *mut *mut Semaphore = hook::slide(0x1006ac8e0);

        unsafe {
            *global_queue_ptr = queue;
            *global_semaphore_loc_ptr = semaphore;
        }
    }

    /// Obtains the global queue.
    fn global() -> GlobalStreamQueue {
        GlobalStreamQueue {
            semaphore_ref: unsafe {
                hook::deref_global::<*mut Semaphore>(0x1006ac8e0)
                    .as_mut()
                    .unwrap()
            },

            index_queue_ref: unsafe { hook::slide::<*mut Queue>(0x100939120).as_mut().unwrap() },
        }
    }

    /// Returns a mutable reference to the next stream to be serviced, removing it from the queue.
    /// Blocks if there are no streams waiting.
    fn pop_blocking(&mut self) -> &'static mut Stream {
        self.semaphore_ref.wait();

        // Get the stream index from the queue and remove it.
        let stream_index = self.index_queue_ref.first() as usize;
        self.index_queue_ref.remove_first();

        // Find the stream corresponding to the index.
        &mut Stream::streams()[stream_index]
    }

    /// Adds the given stream index to the queue.
    fn push_index(&mut self, stream_index: usize) {
        self.index_queue_ref.add(stream_index as i32);
        self.semaphore_ref.post();
    }
}

/// Opaque type used for referring to game mutexes.
#[derive(Debug)]
struct GameMutex;

impl GameMutex {
    /// Creates a new mutex using game code and returns a mutable reference to it.
    fn new_mut() -> &'static mut GameMutex {
        hook::slide::<fn(usize) -> &'static mut GameMutex>(0x1004e8a5c)(0x0)
    }

    /// Locks the mutex, blocking if necessary.
    fn lock(&mut self) {
        hook::slide::<fn(&mut GameMutex)>(0x1004fbd34)(self);
    }

    /// Unlocks the mutex.
    fn unlock(&mut self) {
        hook::slide::<fn(&mut GameMutex)>(0x1004fbd40)(self);
    }
}

/// Opaque type used when referring to the file handles that the game uses.
#[derive(Debug)]
struct FileHandle;

impl FileHandle {
    /// Opens the file at `path` (in `data_area`) with `mode`.
    fn open(path: &CStr, data_area: u64, mode: u64) -> eyre::Result<&'static mut FileHandle> {
        let func: fn(u64, *mut *mut FileHandle, *const c_char, u64) = hook::slide(0x1004e4f94);

        // Create a null pointer for the function to write the resulting file handle pointer to.
        let mut handle_ptr: *mut FileHandle = std::ptr::null_mut();

        // Call the function.
        func(data_area, &mut handle_ptr, path.as_ptr(), 0);

        // Convert the handle to a reference, or `None` if it is null.
        let handle = unsafe { handle_ptr.as_mut() };

        // If the handle is null, there was an error opening the file.
        handle.ok_or_else(|| {
            eyre::format_err!(
                "Unable to open file '{}' in data area {} with mode {:#x}",
                path.to_str().unwrap_or("<unrepresentable>"),
                data_area,
                mode
            )
        })
    }

    /// Reads `count` bytes from the file and stores them at `output`.
    fn read(&mut self, output: *mut u8, count: usize) -> u32 {
        hook::slide::<fn(&mut FileHandle, *mut u8, usize) -> u32>(0x1004e5300)(self, output, count)
    }

    /// Seeks `position` bytes from the start of the file.
    fn seek_to(&mut self, position: usize) -> u32 {
        hook::slide::<fn(&mut FileHandle, usize) -> u32>(0x1004e51dc)(self, position)
    }
}

/// Describes the position and size of a resource in an image file.
#[derive(Clone, Copy)]
struct ImageRegion {
    /// The offset of the resource from the start of its parent image file.
    offset_sectors: usize,

    /// The size of the resource, in sectors.
    size_sectors: usize,
}

impl ImageRegion {
    /// Returns the offset of the region in bytes.
    fn offset_bytes(self) -> usize {
        self.offset_sectors * 2048
    }

    /// Returns the size of the region in bytes.
    fn size_bytes(self) -> usize {
        self.size_sectors * 2048
    }
}

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
    /// Creates a new stream with no image file.
    fn new() -> Stream {
        Stream {
            sector_offset: 0,
            sectors_to_read: 0,
            buffer: std::ptr::null_mut(),
            _do_not_use_img_index: 0,
            in_use: false,
            processing: false,
            _pad: 0,
            status: 0,
            semaphore: Semaphore::new_mut(),
            request_mutex: GameMutex::new_mut(),
            image_file: None,
        }
    }

    /// Returns a mutable reference to the global stream count.
    fn count_mut() -> &'static mut u32 {
        unsafe { &mut *hook::slide::<*mut u32>(0x100939110) }
    }

    /// Returns the global stream count.
    fn global_count() -> u32 {
        hook::deref_global(0x100939110)
    }

    /// Returns a pointer to a block of allocated memory of a suitable size for storing `count`
    /// streams. The memory is allocated using `malloc`.
    fn allocate_streams(count: usize) -> *mut Stream {
        let stream_object_size = std::mem::size_of::<Stream>();

        if stream_object_size != 0x30 {
            panic!(
                "Stream structure should be of size 0x30, not {:#x}",
                stream_object_size
            );
        }

        unsafe { libc::malloc(stream_object_size * count).cast() }
    }

    /// Creates `count` empty streams and stores them in the game's memory. The streams can be
    /// accessed using the `streams` method.
    fn create_streams(count: usize) {
        // Set the global stream count, which will also set the length of the global stream slice.
        *Stream::count_mut() = count as u32;

        // Allocate the stream array.
        unsafe {
            let stream_array_pointer: *mut *mut Stream = hook::slide(0x100939118);
            *stream_array_pointer = Stream::allocate_streams(count);
        }

        // Get the global stream slice. Note that this is currently full of junk data, since we
        // only just allocated it.
        let streams = Stream::streams();

        // Initialise all of the streams.
        streams.fill_with(Stream::new);
    }

    /// Returns a mutable slice of the game's streams.
    fn streams() -> &'static mut [Stream] {
        let stream_array: *mut Stream = hook::deref_global(0x100939118);
        let stream_count: usize = unsafe { *hook::slide::<*mut i32>(0x100939110) } as usize;

        unsafe { std::slice::from_raw_parts_mut(stream_array, stream_count) }
    }

    /// Stores the end of the given resource region as the current stream request position.
    fn set_global_position(image_index: usize, region: ImageRegion) {
        let source = StreamSource::new(image_index as u8, region.offset_sectors as u32);

        unsafe {
            hook::slide::<*mut u32>(0x100939240).write(source.0 + region.size_sectors as u32);
        }
    }

    /// Returns the location in the image file of the requested resource.
    fn file_region(&self) -> ImageRegion {
        ImageRegion {
            offset_sectors: self.sector_offset as usize,
            size_sectors: self.sectors_to_read as usize,
        }
    }

    /// Returns `true` if the stream did not encounter an error on the most recent read.
    fn is_ok(&self) -> bool {
        self.status == 0
    }

    /// Returns `true` if this stream is currently processing a request or has unloaded data.
    fn is_busy(&self) -> bool {
        self.sectors_to_read != 0 || self.processing
    }

    /// Loads the data from `region` in the stream's image file into the stream's buffer.
    fn load_region(&mut self, region: ImageRegion) -> eyre::Result<()> {
        let image_file = self.image_file.as_mut().expect("Stream has no image file");

        // Seek to the start of the requested region in the image file.
        let seek_err = image_file.file_handle_mut().seek_to(region.offset_bytes());

        if seek_err != 0 {
            return Err(eyre::format_err!(
                "Unable to seek to offset {}",
                region.offset_bytes()
            ));
        }

        // Fill the buffer with all of the data from the requested region.
        let read_err = image_file
            .file_handle_mut()
            .read(self.buffer, region.size_bytes());

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

    /// Sets the stream to the processing state.
    fn enter_processing_state(&mut self) {
        self.processing = true;
    }

    /// Sets the stream back to the idle state and clears request information.
    fn exit_processing_state(&mut self) {
        self.request_mutex.lock();

        self.sectors_to_read = 0;

        if self.in_use {
            self.semaphore.post();
        }

        self.processing = false;

        self.request_mutex.unlock();
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

    /// Clears the error code if there is one set.
    fn clear_error(&mut self) {
        self.status = 0;
    }

    /// Sets the location of the next file read within the current image to `region`.
    fn set_region(&mut self, region: ImageRegion) {
        self.sector_offset = region.offset_sectors as u32;
        self.sectors_to_read = region.size_sectors as u32;
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

        self.image_file = Some(image);
        self.set_region(region);

        self.buffer = buffer;
        self.in_use = false;

        Ok(())
    }
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
fn stream_open_hook(path: *const c_char, _: bool) -> i32 {
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

    start.0 as i32
}

fn get_archive_path(path: &str) -> Option<(String, String)> {
    let path = path.to_lowercase();
    let absolute = std::path::Path::new(&crate::loader::find_absolute_path(&path)?).to_owned();

    Some((
        absolute.display().to_string(),
        absolute.file_name()?.to_str()?.to_lowercase(),
    ))
}

fn with_model_names<T>(with: impl Fn(&mut HashMap<StreamSource, String>) -> T) -> T {
    lazy_static::lazy_static! {
        static ref NAMES: Mutex<HashMap<StreamSource, String>> = Mutex::new(HashMap::new());
    }

    let mut locked = NAMES.lock();
    with(locked.as_mut().unwrap())
}

type ArchiveReplacements = HashMap<String, HashMap<String, ArchiveFileReplacement>>;

fn with_replacements<T>(with: &mut impl FnMut(&mut ArchiveReplacements) -> T) -> T {
    lazy_static::lazy_static! {
        static ref REPLACEMENTS: Mutex<ArchiveReplacements> = Mutex::new(HashMap::new());
    }

    let mut locked = REPLACEMENTS.lock();
    with(locked.as_mut().unwrap())
}

// fixme: Loading archives causes a visible delay during loading.
fn load_archive_into_database(path: &str, img_id: i32) -> eyre::Result<()> {
    // We use a BufReader because we do many small reads.
    let mut file = std::io::BufReader::new(std::fs::File::open(path)?);

    let identifier = file.read_u32::<LittleEndian>()?;

    // 0x32524556 is VER2 as an unsigned integer.
    if identifier != 0x32524556 {
        log::error!("Archive does not have a VER2 identifier! Processing will continue anyway.");
    }

    let entry_count = file.read_u32::<LittleEndian>()?;

    log::info!("Archive has {} entries.", entry_count);

    for _ in 0..entry_count {
        let offset = file.read_u32::<LittleEndian>()?;

        // Ignore the two u16 size values.
        file.seek(SeekFrom::Current(4))?;

        let name = {
            let mut name_buf = [0u8; 24];
            file.read_exact(&mut name_buf)?;

            let name = unsafe { CStr::from_ptr(name_buf.as_ptr().cast()) }
                .to_str()
                .unwrap();

            name.to_string()
        };

        let source = StreamSource::new(img_id as u8, offset);

        with_model_names(|models| {
            models.insert(source, name.clone());
        });
    }

    Ok(())
}

fn load_directory(path_c: *const i8, archive_id: i32) {
    let path = unsafe { CStr::from_ptr(path_c) }.to_str().unwrap();

    let (path, archive_name) = get_archive_path(path).expect("Unable to resolve path name.");

    log::info!("Registering contents of archive '{}'.", archive_name);

    if let Err(err) = load_archive_into_database(&path, archive_id) {
        log::error!("Failed to load archive: {}", err);
        call_original!(targets::load_cd_directory, path_c, archive_id);
        return;
    } else {
        log::info!("Registered archive contents successfully.");
    }

    call_original!(targets::load_cd_directory, path_c, archive_id);

    let model_info_arr: *mut ModelInfo = hook::slide(0x1006ac8f4);

    with_model_names(|model_names| {
        with_replacements(&mut |replacements| {
            let replacement_map = if let Some(map) = replacements.get(&archive_name) {
                map
            } else {
                return;
            };

            // 26316 is the total number of models in the model array.
            for i in 0..26316 {
                let info = unsafe { model_info_arr.offset(i as isize).as_mut().unwrap() };
                let stream_source = StreamSource::new(info.img_id, info.cd_pos);

                if let Some(name) = model_names.get_mut(&stream_source) {
                    let name = name.to_lowercase();

                    if let Some(child) = replacement_map.get(&name) {
                        log::info!(
                            "{} at ({}, {}) will be replaced",
                            name,
                            info.img_id,
                            info.cd_pos
                        );

                        let size_segments = child.size_in_segments();
                        info.cd_size = size_segments;
                    }
                }

                // Increase the size of the streaming buffer to accommodate the model's data (if it isn't big enough
                //  already).
                let streaming_buffer_size: u32 =
                    hook::deref_global::<u32>(0x10072d320).max(info.cd_size);

                unsafe {
                    *hook::slide::<*mut u32>(0x10072d320) = streaming_buffer_size;
                }
            }
        });
    });
}

pub fn init() {
    log::info!("installing stream hooks...");

    const CD_STREAM_INIT: hook::Target<fn(i32)> = hook::Target::Address(0x100177eb8);
    CD_STREAM_INIT.hook_hard(stream_init_hook);

    type StreamReadFn = fn(u32, *mut u8, StreamSource, u32) -> bool;
    const CD_STREAM_READ: hook::Target<StreamReadFn> = hook::Target::Address(0x100178048);
    CD_STREAM_READ.hook_hard(stream_read_hook);

    const CD_STREAM_OPEN: hook::Target<fn(*const c_char, bool) -> i32> =
        hook::Target::Address(0x1001782b0);
    CD_STREAM_OPEN.hook_hard(stream_open_hook);

    targets::load_cd_directory::install(load_directory);
}

struct ArchiveFileReplacement {
    size_bytes: u32,

    // We keep the file open so it can't be modified while the game is running, because that could cause
    //  issues with the buffer size.
    file: std::fs::File,
}

impl ArchiveFileReplacement {
    fn reset(&mut self) {
        if let Err(err) = self.file.seek(SeekFrom::Start(0)) {
            log::error!(
                "Could not seek to start of archive replacement file: {}",
                err
            );
        }
    }

    fn size_in_segments(&self) -> u32 {
        (self.size_bytes + 2047) / 2048
    }
}

pub fn load_replacement(image_name: &str, path: &impl AsRef<Path>) -> eyre::Result<()> {
    with_replacements(&mut |replacements| {
        let size = path.as_ref().metadata()?.len();

        let replacement = ArchiveFileReplacement {
            size_bytes: size as u32,
            file: std::fs::File::open(path)?,
        };

        let name = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase();

        if let Some(map) = replacements.get_mut(image_name) {
            map.insert(name, replacement);
        } else {
            let mut map = HashMap::new();
            map.insert(name, replacement);

            replacements.insert(image_name.into(), map);
        }

        Ok(())
    })
}

#[repr(C)]
#[derive(Debug)]
struct ModelInfo {
    next_index: i16,
    prev_index: i16,
    next_index_on_cd: i16,
    flags: u8,
    img_id: u8,
    cd_pos: u32,
    cd_size: u32,
    load_state: u8,
    _pad: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StreamSource(u32);

impl StreamSource {
    fn new(image_index: u8, sector_offset: u32) -> StreamSource {
        StreamSource(((image_index as u32) << 24) | sector_offset)
    }

    fn image_index(&self) -> u8 {
        // The top 8 bits encode the index of the image that the resource is from in the global
        //  image handle array.
        (self.0 >> 24) as u8
    }

    fn sector_offset(&self) -> u32 {
        // The bottom 24 bits encode the number of sectors the resource is from the beginning of
        //  the file.
        self.0 & 0xffffff
    }
}

#[repr(C)]
#[derive(Debug)]
struct Stream {
    sector_offset: u32,
    sectors_to_read: u32,
    buffer: *mut u8,

    // CLEO addition.
    // hack: We shouldn't really need to add another field to Stream.
    _do_not_use_img_index: u8,

    in_use: bool,
    processing: bool,
    _pad: u8,
    status: u32,
    semaphore: &'static mut Semaphore,
    request_mutex: &'static mut GameMutex,
    image_file: Option<&'static mut ImageHandle>,
}

#[repr(C)]
#[derive(Debug)]
struct Queue {
    data: *mut i32,
    head: u32,
    tail: u32,
    capacity: u32,
}

impl Queue {
    fn with_capacity(capacity: u32) -> Queue {
        Queue {
            data: unsafe { libc::malloc(capacity as usize * 4).cast() },
            head: 0,
            tail: 0,
            capacity,
        }
    }

    fn add(&mut self, value: i32) {
        unsafe {
            self.data.offset(self.tail as isize).write(value);
        }

        self.tail = (self.tail + 1) % self.capacity;
    }

    fn first(&self) -> i32 {
        if self.head == self.tail {
            -1
        } else {
            unsafe { self.data.offset(self.head as isize).read() }
        }
    }

    fn remove_first(&mut self) {
        if self.head != self.tail {
            self.head = (self.head + 1) % self.capacity;
        }
    }
}
