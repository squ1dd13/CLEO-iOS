use std::ffi::CStr;

use eyre::Result;

use crate::hook;

/// Opaque semaphore type. Use only as a pointer or reference.
#[derive(Debug)]
pub struct Semaphore;

impl Semaphore {
    /// Creates a new semaphore using game code and returns a mutable reference to it.
    pub fn new_mut() -> &'static mut Semaphore {
        hook::slide::<fn() -> &'static mut Semaphore>(0x1004e8b18)()
    }

    /// Blocks until the semaphore count becomes greater than zero, then decrements it and returns.
    pub fn wait(&mut self) {
        hook::slide::<fn(&mut Semaphore)>(0x1004e8b84)(self);
    }

    /// Increments the semaphore count.
    pub fn post(&mut self) {
        hook::slide::<fn(&mut Semaphore)>(0x1004e8b5c)(self);
    }
}

/// Opaque mutex type. Use only as a pointer or reference.
#[derive(Debug)]
pub struct GameMutex;

impl GameMutex {
    /// Creates a new mutex using game code and returns a mutable reference to it.
    pub fn new_mut() -> &'static mut GameMutex {
        hook::slide::<fn(usize) -> &'static mut GameMutex>(0x1004e8a5c)(0x0)
    }

    /// Locks the mutex, blocking if necessary.
    pub fn lock(&mut self) {
        unsafe {
            libc::pthread_mutex_lock((self as *mut GameMutex).cast());
        }
    }

    /// Unlocks the mutex.
    pub fn unlock(&mut self) {
        unsafe {
            libc::pthread_mutex_unlock((self as *mut GameMutex).cast());
        }
    }
}

/// Pointer to a game file structure.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePointer(*mut u8);

impl FilePointer {
    /// Returns a null file pointer.
    pub fn null() -> FilePointer {
        FilePointer(std::ptr::null_mut())
    }

    /// Opens the file at `path` (in `data_area`) with `mode`.
    pub fn open(path: &CStr, data_area: u64, mode: u64) -> Result<FilePointer> {
        let func: fn(u64, &mut FilePointer, *const i8, u64) = hook::slide(0x1004e4f94);

        // Create a null pointer for the function to overwrite with the new file pointer.
        let mut handle = FilePointer(std::ptr::null_mut());

        // Call the function.
        func(data_area, &mut handle, path.as_ptr(), 0);

        match std::ptr::NonNull::new(handle.0) {
            Some(ptr) => Ok(FilePointer(ptr.as_ptr())),

            // If the handle is null, there was an error opening the file.
            None => Err(eyre::format_err!(
                "Unable to open file '{}' in data area {} with mode {:#x}",
                path.to_str().unwrap_or("<unrepresentable>"),
                data_area,
                mode
            )),
        }
    }

    /// Reads `count` bytes from the file and stores them at `output`.
    pub fn read(self, output: *mut u8, count: usize) -> u32 {
        hook::slide::<fn(FilePointer, *mut u8, usize) -> u32>(0x1004e5300)(self, output, count)
    }

    /// Seeks `position` bytes from the start of the file.
    pub fn seek_to(self, position: usize) -> u32 {
        hook::slide::<fn(FilePointer, usize) -> u32>(0x1004e51dc)(self, position)
    }
}

/// The game's queue type.
#[repr(C)]
#[derive(Debug)]
pub struct Queue {
    data: *mut i32,
    head: u32,
    tail: u32,
    capacity: u32,
}

impl Queue {
    pub fn with_capacity(capacity: u32) -> Queue {
        Queue {
            data: unsafe { libc::malloc(capacity as usize * 4).cast() },
            head: 0,
            tail: 0,
            capacity,
        }
    }

    pub fn add(&mut self, value: i32) {
        unsafe {
            self.data.offset(self.tail as isize).write(value);
        }

        self.tail = (self.tail + 1) % self.capacity;
    }

    pub fn first(&self) -> i32 {
        if self.head == self.tail {
            -1
        } else {
            unsafe { self.data.offset(self.head as isize).read() }
        }
    }

    pub fn remove_first(&mut self) {
        if self.head != self.tail {
            self.head = (self.head + 1) % self.capacity;
        }
    }
}

/// A queue of streams.
///
/// When data is requested from a stream, the stream is added to the queue and the stream queue
/// semaphore is incremented.
pub(super) struct GlobalStreamQueue {
    /// The global stream queue semaphore.
    semaphore_ref: &'static mut Semaphore,

    /// The global queue of stream indices that represent the streams in this queue.
    index_queue_ref: &'static mut Queue,
}

impl GlobalStreamQueue {
    /// Initialises the global queue.
    pub fn init() {
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
    pub fn global() -> GlobalStreamQueue {
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
    pub fn pop_blocking(&mut self) -> &'static mut Stream {
        self.semaphore_ref.wait();

        // Get the stream index from the queue and remove it.
        let stream_index = self.index_queue_ref.first() as usize;
        self.index_queue_ref.remove_first();

        // Find the stream corresponding to the index.
        &mut Stream::streams()[stream_index]
    }

    /// Adds the given stream index to the queue.
    pub fn push_index(&mut self, stream_index: usize) {
        self.index_queue_ref.add(stream_index as i32);
        self.semaphore_ref.post();
    }
}

/// Describes the position and size of a resource in an image file.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageRegion {
    /// The offset of the resource from the start of its parent image file.
    pub offset_sectors: usize,

    /// The size of the resource, in sectors.
    pub size_sectors: usize,
}

impl ImageRegion {
    /// Returns the offset of the region in bytes.
    pub fn offset_bytes(self) -> usize {
        self.offset_sectors * 2048
    }

    /// Returns the size of the region in bytes.
    pub fn size_bytes(self) -> usize {
        self.size_sectors * 2048
    }
}

impl std::fmt::Debug for ImageRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ImageRegion {{ offset = {} segments / {} bytes, size = {} segments / {} bytes }}",
            self.offset_sectors,
            self.offset_bytes(),
            self.size_sectors,
            self.size_bytes()
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct StreamSource(u32);

impl StreamSource {
    pub fn new(image_index: u8, sector_offset: u32) -> StreamSource {
        StreamSource(((image_index as u32) << 24) | sector_offset)
    }

    pub fn image_index(self) -> u8 {
        // The top 8 bits encode the index of the image that the resource is from in the global
        // image handle array.
        (self.0 >> 24) as u8
    }

    pub fn sector_offset(self) -> u32 {
        // The bottom 24 bits encode the number of sectors the resource is from the beginning of
        // the file.
        self.0 & 0xffffff
    }

    /// Returns the 32-bit representation of the position.
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Stream {
    sector_offset: u32,
    sectors_to_read: u32,
    pub(super) buffer: *mut u8,

    // CLEO addition.
    // hack: We shouldn't really need to add another field to Stream.
    _do_not_use_img_index: u8,

    pub(super) in_use: bool,
    processing: bool,
    _pad: u8,
    pub(super) status: u32,
    semaphore: &'static mut Semaphore,
    request_mutex: &'static mut GameMutex,
    pub(super) image_file: FilePointer,
}

impl Stream {
    /// Creates a new stream with no image file.
    pub fn new() -> Stream {
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
            image_file: FilePointer::null(),
        }
    }

    /// Returns a mutable reference to the global stream count.
    fn count_mut() -> &'static mut u32 {
        unsafe { &mut *hook::slide::<*mut u32>(0x100939110) }
    }

    /// Returns the global stream count.
    pub fn global_count() -> u32 {
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
    pub fn create_streams(count: usize) {
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
    pub fn streams() -> &'static mut [Stream] {
        let stream_array: *mut Stream = hook::deref_global(0x100939118);
        let stream_count: usize = unsafe { *hook::slide::<*mut i32>(0x100939110) } as usize;

        unsafe { std::slice::from_raw_parts_mut(stream_array, stream_count) }
    }

    /// Stores the end of the given resource region as the current stream request position.
    pub fn set_global_position(image_index: usize, region: ImageRegion) {
        let source = StreamSource::new(image_index as u8, region.offset_sectors as u32);

        unsafe {
            hook::slide::<*mut u32>(0x100939240).write(source.0 + region.size_sectors as u32);
        }
    }

    /// Returns the location in the image file of the requested resource.
    pub fn file_region(&self) -> ImageRegion {
        ImageRegion {
            offset_sectors: self.sector_offset as usize,
            size_sectors: self.sectors_to_read as usize,
        }
    }

    /// Returns `true` if the stream did not encounter an error on the most recent read.
    pub fn is_ok(&self) -> bool {
        self.status == 0
    }

    /// Returns `true` if this stream is currently processing a request or has unloaded data.
    pub fn is_busy(&self) -> bool {
        self.sectors_to_read != 0 || self.processing
    }

    /// Sets the stream to the processing state.
    pub fn enter_processing_state(&mut self) {
        self.processing = true;
    }

    /// Sets the stream back to the idle state and clears request information.
    pub fn exit_processing_state(&mut self) {
        self.request_mutex.lock();

        self.sectors_to_read = 0;

        if self.in_use {
            self.semaphore.post();
        }

        self.processing = false;

        self.request_mutex.unlock();
    }

    /// Clears the error code if there is one set.
    pub fn clear_error(&mut self) {
        self.status = 0;
    }

    /// Sets the location of the next file read within the current image to `region`.
    pub fn set_region(&mut self, region: ImageRegion) {
        self.sector_offset = region.offset_sectors as u32;
        self.sectors_to_read = region.size_sectors as u32;
    }
}
