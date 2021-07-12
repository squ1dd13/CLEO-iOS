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
        let stream: &mut Stream = unsafe { &mut *streams.add(i) };

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

    let mut image_names: Vec<Option<String>> = std::iter::repeat(None).take(32).collect();

    loop {
        let stream_semaphore = hook::get_global(0x1006ac8e0);

        // eq: OS_SemaphoreWait(...)
        hook::slide::<fn(*mut u8)>(0x1004e8b84)(stream_semaphore);

        let queue = streaming_queue();
        let streams = streams_array();

        // Get the first stream index from the queue and then get a reference to the stream.
        let stream_index = queue.first() as isize;
        let mut stream = unsafe { &mut *streams.offset(stream_index) };

        // Mark the stream as in use.
        stream.processing = true;

        // A status of 0 means that the last read was successful.
        if stream.status == 0 {
            let stream_source = StreamSource::new(stream.img_index, stream.sector_offset);

            let stream_index_usize = stream.img_index as usize;

            // We cache image names because obtaining them is quite expensive.
            let image_name = match &image_names[stream_index_usize] {
                Some(name) => name,
                None => {
                    let image_name = unsafe {
                        let name_ptr = hook::slide::<*mut i8>(0x1006ac0e0)
                            .offset(stream.img_index as isize * 64);

                        let path_string = std::ffi::CStr::from_ptr(name_ptr)
                            .to_str()
                            .unwrap()
                            .to_lowercase()
                            .replace('\\', "/");

                        // The game actually uses paths from the PC version, but we only want the file names.
                        Path::new(&path_string)
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string()
                    };

                    let slot = &mut image_names[stream_index_usize];
                    *slot = Some(image_name);
                    slot.as_ref().unwrap()
                }
            };

            let read_custom = with_replacements(&mut |replacements| {
                let replacements = replacements.get_mut(image_name)?;

                let model_name = with_model_names(|models| models.get(&stream_source).cloned())?;

                let folder_child = replacements.get_mut(&model_name)?;

                // Reset the file to offset 0 so we are definitely reading from the start.
                folder_child.reset();

                let file = &mut folder_child.file;

                let mut buffer = vec![0u8; stream.sectors_to_read as usize * 2048];

                // read_exact here would cause a crash for models that don't have aligned sizes, since
                //  we can't read enough to fill the whole buffer.
                if let Err(err) = file.read(&mut buffer) {
                    log::error!("Failed to read model data: {}", err);
                    stream.status = 0xfe;
                } else {
                    unsafe {
                        // todo: Read directly into streaming buffer rather than reading and copying.
                        std::ptr::copy(buffer.as_ptr(), stream.buffer, buffer.len());
                    }

                    stream.status = 0;
                }

                Some(())
            });

            if read_custom.is_none() {
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

fn stream_read(
    stream_index: u32,
    buffer: *mut u8,
    source: StreamSource,
    sector_count: u32,
) -> bool {
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

    // Set up the stream for getting the resource we want.
    stream.status = 0;
    stream.sector_offset = source.sector_offset();
    stream.sectors_to_read = sector_count;
    stream.buffer = buffer;
    stream.locked = false;
    stream.img_index = source.image_index();

    streaming_queue().add(stream_index as i32);

    let stream_semaphore = hook::get_global(0x1006ac8e0);

    // eq: OS_SemaphorePost(...)
    hook::slide::<fn(*mut u8)>(0x1004e8b5c)(stream_semaphore);

    true
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

fn load_archive_into_database(path: &str, img_id: i32) -> std::io::Result<()> {
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
                    hook::get_global::<u32>(0x10072d320).max(info.cd_size as u32);

                unsafe {
                    *hook::slide::<*mut u32>(0x10072d320) = streaming_buffer_size;
                }
            }
        });
    });
}

pub fn hook() {
    const CD_STREAM_INIT: hook::Target<fn(i32)> = hook::Target::Address(0x100177eb8);
    CD_STREAM_INIT.hook_hard(stream_init);

    const CD_STREAM_READ: hook::Target<fn(u32, *mut u8, StreamSource, u32) -> bool> =
        hook::Target::Address(0x100178048);
    CD_STREAM_READ.hook_hard(stream_read);

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

pub fn load_replacement(image_name: &str, path: &impl AsRef<Path>) -> std::io::Result<()> {
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
    img_index: u8,

    locked: bool,
    processing: bool,
    _pad: u8,
    status: u32,
    semaphore: *mut u8,
    mutex: *mut u8,
    file: *mut u8,
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
