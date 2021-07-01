use crate::hook;

fn zero_memory(ptr: *mut u8, bytes: usize) {
    for i in 0..bytes {
        unsafe {
            ptr.offset(i as isize).write(0);
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

fn streams_array() -> *mut Stream {
    hook::get_global(0x100939118)
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
        let stream: &mut Stream = unsafe { &mut *streams.offset(i as isize) };

        // eq: OS_SemaphoreCreate()
        stream.semaphore = hook::slide::<fn() -> *mut u8>(0x1004e8b18)();

        // eq: OS_MutexCreate(...)
        stream.mutex = hook::slide::<fn(usize) -> *mut u8>(0x1004e8a5c)(0x0);

        if stream.semaphore.is_null() {
            panic!("Stream {} semaphore is null!", i);
        }

        if stream.mutex.is_null() {
            panic!("Stream {} mutex is null!", i);
        }
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
        let thread = launch(stream_thread, 0x0, 3, cd_stream_name, 0, 3);

        if thread.is_null() {
            panic!("Failed to start streaming thread!");
        }

        global_stream_thread.write(thread);
    }
}

fn stream_thread(_: usize) {
    log::trace!("Streaming thread started!");

    loop {
        let stream_semaphore = hook::get_global(0x1006ac8e0);

        // eq: OS_SemaphoreWait(...)
        hook::slide::<fn(*mut u8)>(0x1004e8b84)(stream_semaphore);

        let queue = streaming_queue();
        let streams = streams_array();

        // Get the first stream index from the queue and then get a reference to the stream.
        let stream_index = queue.first() as isize;
        let mut stream = unsafe { &mut *streams.offset(stream_index) };

        log::trace!("stream = {:#?}", stream);

        // Mark the stream as in use.
        stream.processing = true;

        // A status of 0 means that the last read was successful.
        if stream.status == 0 {
            // Multiply the sector values by 2048 (the sector size) in order to get byte values.
            let byte_offset = stream.sector_offset * 2048;
            let bytes_to_read = stream.sectors_to_read * 2048;

            // eq: OS_FileSetPosition(...)
            hook::slide::<fn(*mut u8, u32) -> u32>(0x1004e51dc)(stream.file, byte_offset);

            // eq: OS_FileRead(...)
            let read_result = hook::slide::<fn(*mut u8, *mut u8, u32) -> u32>(0x1004e5300)(
                stream.file,
                stream.buffer,
                bytes_to_read,
            );

            stream.status = if read_result != 0 { 0xfe } else { 0 };
        }

        // Remove the queue entry we just processed so the next iteration processes the item after.
        queue.remove_first();

        // eq: pthread_mutex_lock(...)
        hook::slide::<fn(*mut u8)>(0x1004fbd34)(stream.mutex);

        stream.sectors_to_read = 0;

        if stream.locked {
            // eq: OS_SemaphorePost(...)
            hook::slide::<fn(*mut u8)>(0x1004e8b5c)(stream.semaphore);
        }

        stream.processing = false;

        // eq: pthread_mutex_unlock(...)
        hook::slide::<fn(*mut u8)>(0x1004fbd40)(stream.mutex);
    }
}

/*

    Plan:
      - Before archives are loaded for streaming, open them and find information for the models we're overriding.
      - Let the streaming system load, but before streaming begins, change the model info so our models can be loaded (e.g. increase load size).
      - When a request is made for an overridden model, load the custom model from the file instead of from the archive.
         * Keep all custom models files open so we can read straight away.
      - Custom files go in x.img folders, with ".img" being associated with an archive component which registers the new files with the
         streaming system.

*/

fn stream_read(
    stream_index: u32,
    buffer: *mut u8,
    source: StreamSource,
    sector_count: u32,
) -> bool {
    log::trace!(
        "stream_read({}, {:#?}, {:#x}, {})",
        stream_index,
        buffer,
        source.0,
        sector_count
    );

    unsafe {
        hook::slide::<*mut u32>(0x100939240).write(source.0 + sector_count);
    }

    let stream = unsafe { &mut *streams_array().offset(stream_index as isize) };

    unsafe {
        let handle_arr_base: *mut *mut u8 = hook::slide(0x100939140);
        let handle_ptr: *mut u8 = *handle_arr_base.offset(source.image_index() as isize);

        stream.file = handle_ptr;
    }

    if stream.sectors_to_read != 0 || stream.processing {
        return false;
    }

    // Set up the stream for getting the resource we want
    stream.status = 0;
    stream.sector_offset = source.sector_offset();
    stream.sectors_to_read = sector_count;
    stream.buffer = buffer;
    stream.locked = false;

    streaming_queue().add(stream_index);

    let stream_semaphore = hook::get_global(0x1006ac8e0);

    // eq: OS_SemaphorePost(...)
    hook::slide::<fn(*mut u8)>(0x1004e8b5c)(stream_semaphore);

    true
}

pub fn hook() {
    const CD_STREAM_INIT: crate::hook::Target<fn(i32)> = crate::hook::Target::Address(0x100177eb8);
    CD_STREAM_INIT.hook_hard(stream_init);

    const CD_STREAM_READ: crate::hook::Target<fn(u32, *mut u8, StreamSource, u32) -> bool> =
        crate::hook::Target::Address(0x100178048);
    CD_STREAM_READ.hook_hard(stream_read);
}

#[repr(C)]
struct StreamSource(u32);

impl StreamSource {
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
    _pad: u8,
    locked: bool,
    processing: bool,
    _pad0: u8,
    status: u32,
    semaphore: *mut u8,
    mutex: *mut u8,
    file: *mut u8,
}

#[repr(C)]
#[derive(Debug)]
struct Queue {
    data: *mut u32,
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

    fn finalise(&mut self) {
        unsafe {
            libc::free(self.data.cast());
        }

        self.data = std::ptr::null_mut();
        self.head = 0;
        self.tail = 0;
        self.capacity = 0;
    }

    fn add(&mut self, value: u32) {
        log::trace!("{:#?}", self);

        unsafe {
            self.data.offset(self.tail as isize).write(value);
        }

        self.tail = (self.tail + 1) % self.capacity;
    }

    fn first(&self) -> i32 {
        if self.head == self.tail {
            -1
        } else {
            unsafe { self.data.offset(self.head as isize).cast::<i32>().read() }
        }
    }

    fn remove_first(&mut self) {
        if self.head != self.tail {
            self.head = (self.head + 1) % self.capacity;
        }
    }
}
