//! Replaces parts of the game's streaming system to allow the loading of replacement files inside IMGs,
//! and also manages the loaded replacements.

// hack: The `stream` module is messy, poorly documented and full of hacky code.
// bug: Opcode 0x04ee seems to break when animations have been swapped.

use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::Mutex,
};

use byteorder::{LittleEndian, ReadBytesExt};
use libc::c_char;

use crate::{call_original, hook, targets};

fn zero_memory(ptr: *mut u8, bytes: usize) {
    for i in 0..bytes {
        unsafe {
            ptr.add(i).write(0);
        }
    }
}

fn streaming_queue() -> &'static mut Queue {
    unsafe {
        hook::slide::<*mut Queue>(0x100939120)
            .as_mut()
            .expect("how tf how did we manage to slide and get zero?")
    }
}

fn stream_init(stream_count: i32) {
    // Zero the image handles.
    zero_memory(hook::slide(0x100939140), 0x100);

    // Zero the image names.
    zero_memory(hook::slide(0x1006ac0e0), 2048);

    // Write the stream count to the global count variable.
    unsafe {
        hook::slide::<*mut i32>(0x100939110).write(stream_count);
    }

    let streams = {
        let streams_double_ptr: *mut *mut Stream = hook::slide(0x100939118);

        unsafe {
            // Allocate the stream array. Each stream structure is 48 (0x30) bytes.
            let byte_count = stream_count as usize * 0x30;
            let allocated = libc::malloc(byte_count).cast();
            zero_memory(allocated, byte_count);

            streams_double_ptr.write(allocated.cast());
            *streams_double_ptr
        }
    };

    let stream_struct_size = std::mem::size_of::<Stream>();
    if stream_struct_size != 0x30 {
        panic!(
            "Incorrect size for Stream structure: expected 0x30, got {:#?}.",
            stream_struct_size
        );
    }

    for i in 0..stream_count as usize {
        let stream: &mut Stream = unsafe { &mut *streams.add(i) };

        // eq: OS_SemaphoreCreate()
        stream.semaphore = hook::slide::<fn() -> &'static mut Semaphore>(0x1004e8b18)();

        // eq: OS_MutexCreate(...)
        stream.request_mutex = hook::slide::<fn(usize) -> &'static mut GameMutex>(0x1004e8a5c)(0x0);
    }

    *streaming_queue() = Queue::with_capacity(stream_count as u32 + 1);

    unsafe {
        // Create the global stream semaphore.
        // eq: OS_SemaphoreCreate()
        let semaphore = hook::slide::<fn() -> *mut u8>(0x1004e8b18)();

        if semaphore.is_null() {
            panic!("Failed to create global stream semaphore!");
        }

        // Write to the variable.
        hook::slide::<*mut *mut u8>(0x1006ac8e0).write(semaphore);

        // "CdStream"
        let cd_stream_name: *const u8 = hook::slide(0x10058a2eb);
        let global_stream_thread: *mut *mut u8 = hook::slide(0x100939138);

        // Launch the thread.
        let launch =
            hook::slide::<fn(fn(usize), usize, u32, *const u8, i32, u32) -> *mut u8>(0x1004e8888);

        // eq: OS_ThreadLaunch(...)
        let thread = launch(streaming_thread_hook, 0x0, 3, cd_stream_name, 0, 3);

        if thread.is_null() {
            panic!("Failed to start streaming thread!");
        }

        global_stream_thread.write(thread);
    }
}

/// Opaque type used to refer to semaphores used by the game.
#[derive(Debug)]
struct Semaphore;

impl Semaphore {
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
    semaphore: &'static mut Semaphore,

    /// The global queue of stream indices that represent the streams in this queue.
    index_queue: &'static mut Queue,
}

impl GlobalStreamQueue {
    /// Obtains the global queue.
    fn global() -> GlobalStreamQueue {
        GlobalStreamQueue {
            semaphore: unsafe {
                hook::get_global::<*mut Semaphore>(0x1006ac8e0)
                    .as_mut()
                    .unwrap()
            },

            index_queue: unsafe { hook::slide::<*mut Queue>(0x100939120).as_mut().unwrap() },
        }
    }

    /// Returns a mutable reference to the next stream to be serviced, removing it from the queue.
    /// Blocks if there are no streams waiting.
    fn pop_blocking(&mut self) -> &'static mut Stream {
        self.semaphore.wait();

        // Get the stream index from the queue and remove it.
        let stream_index = self.index_queue.first() as usize;
        self.index_queue.remove_first();

        // Find the stream corresponding to the index.
        &mut Stream::streams()[stream_index]
    }

    /// Adds the given stream index to the queue.
    fn push_index(&mut self, stream_index: usize) {
        self.index_queue.add(stream_index as i32);
        self.semaphore.post();
    }
}

/// Opaque type used for referring to game mutexes.
#[derive(Debug)]
struct GameMutex;

impl GameMutex {
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

/// An image file handle.
#[derive(Debug)]
struct ImageHandle(FileHandle);

impl ImageHandle {
    /// Returns a slice of mutable references to the game's image handles.
    fn images() -> &'static mut [&'static mut ImageHandle] {
        let image_handle_ptrs: *mut &'static mut ImageHandle = hook::slide(0x100939140);

        unsafe { std::slice::from_raw_parts_mut(image_handle_ptrs, 8) }
    }

    /// Returns a mutable reference to the underlying file handle.
    fn file_handle_mut(&mut self) -> &mut FileHandle {
        &mut self.0
    }
}

impl Stream {
    /// Returns a mutable slice of the game's streams.
    fn streams() -> &'static mut [Stream] {
        let stream_array: *mut Stream = hook::get_global(0x100939118);
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
        // Seek to the start of the requested region in the image file.
        let seek_err = self
            .image_file
            .file_handle_mut()
            .seek_to(region.offset_bytes());

        if seek_err != 0 {
            return Err(eyre::format_err!(
                "Unable to seek to offset {}",
                region.offset_bytes()
            ));
        }

        // Fill the buffer with all of the data from the requested region.
        let read_err = self
            .image_file
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

        self.image_file = image;
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
    let image = &mut ImageHandle::images()[image_index];

    let request_result = stream.request_load(image, region, buffer);

    if request_result.is_err() {
        return false;
    }

    let mut queue = GlobalStreamQueue::global();
    queue.push_index(stream_index);

    true
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

fn stream_open(path: *const c_char, _: bool) -> i32 {
    let handles: *mut *mut u8 = hook::slide(0x100939140);

    // Find the first available place in the handles array.
    let mut index = 0;

    for i in 0..32isize {
        unsafe {
            if handles.offset(i).read().is_null() {
                break;
            }
        }

        index += 1;
    }

    if index == 32 {
        log::error!("No available slot for image.");
        return 0;
    }

    // eq: OS_FileOpen(...)
    let file_open: fn(u64, *mut *mut u8, *const c_char, u64) = hook::slide(0x1004e4f94);
    file_open(0, unsafe { handles.offset(index) }, path, 0);

    unsafe {
        if handles.offset(index).read().is_null() {
            return 0;
        }
    }

    let image_names: *mut i8 = hook::slide(0x1006ac0e0);

    unsafe {
        let dest = image_names.offset(index * 64);
        libc::strcpy(dest, path.cast());
    }

    (index as i32) << 24
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

            let name = unsafe { std::ffi::CStr::from_ptr(name_buf.as_ptr().cast()) }
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
    let path = unsafe { std::ffi::CStr::from_ptr(path_c) }
        .to_str()
        .unwrap();

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
                    hook::get_global::<u32>(0x10072d320).max(info.cd_size);

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
    CD_STREAM_INIT.hook_hard(stream_init);

    type StreamReadFn = fn(u32, *mut u8, StreamSource, u32) -> bool;
    const CD_STREAM_READ: hook::Target<StreamReadFn> = hook::Target::Address(0x100178048);
    CD_STREAM_READ.hook_hard(stream_read_hook);

    const CD_STREAM_OPEN: hook::Target<fn(*const c_char, bool) -> i32> =
        hook::Target::Address(0x1001782b0);
    CD_STREAM_OPEN.hook_hard(stream_open);

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
            file: std::fs::File::open(&path)?,
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
    image_file: &'static mut ImageHandle,
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
