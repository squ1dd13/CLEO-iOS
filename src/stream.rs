use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use libc::c_char;

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

        // Mark the stream as in use.
        stream.processing = true;

        // A status of 0 means that the last read was successful.
        if stream.status == 0 {
            // Multiply the sector values by 2048 (the sector size) in order to get byte values.
            let (byte_offset, over) = stream.sector_offset.overflowing_mul(2048);

            if over {
                log::error!("Offset calculation caused overflow.");
                log::trace!("{:#?}", stream);
                stream.status = 0xfe;
                panic!();
            }

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

    if source.sector_offset().overflowing_mul(2048).1 {
        let image_names: *mut i8 = hook::slide(0x1006ac0e0);

        let image_name = unsafe {
            let dest = image_names.offset(source.image_index() as isize * 64);
            std::ffi::CStr::from_ptr(dest).to_str().unwrap()
        };

        log::error!(
            "Bad read: {} in image {}.",
            source.sector_offset(),
            image_name
        );
    }

    streaming_queue().add(stream_index);

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
    let path = path.replace('\\', "/");

    // Up to now, we have the path as it would be relative to the .exe in the PC's game directory.
    // We aren't using the game's file management, so we need to resolve this ourselves.
    let current_exe = std::env::current_exe().ok()?;
    let dir = current_exe.parent()?;

    let archive_name = std::path::Path::new(path.as_str())
        .file_name()?
        .to_str()
        .unwrap();
    let mut absolute_path = dir.to_path_buf();
    absolute_path.push(archive_name.to_lowercase());

    let path = absolute_path.to_str().unwrap();

    if !absolute_path.exists() {
        log::warn!("Absolute path '{}' does not exist.", path);
    }

    Some((
        crate::loader::process_path(path).unwrap_or(path.to_string()),
        archive_name.to_string(),
    ))
}

fn load_directory(path: &str, archive_id: i32) {
    let (path, archive_name) = get_archive_path(path).expect("Unable to resolve path name.");
    let mut file =
        std::io::BufReader::new(std::fs::File::open(&path).expect("Failed to open file."));

    // 0x32524556 is VER2 as an unsigned integer.
    let identifier = file
        .read_u32::<LittleEndian>()
        .expect("Failed to read identifier.");
    if identifier != 0x32524556 {
        log::error!(
            "Archive '{}' does not have VER2 identifier! Processing will continue anyway.",
            archive_name
        );
    }

    let entry_count = file
        .read_u32::<LittleEndian>()
        .expect("Failed to read entry count.");

    log::info!("{} has {} entries.", archive_name, entry_count);

    let mut last_time_check = std::time::Instant::now();

    let mut previous_model_id = -1i32;

    #[repr(C)]
    struct Entry {
        offset: u32,
        streaming_size: u16,
        size_in_archive: u16,
        name: [u8; 24],
    }

    for _ in 0..entry_count {
        let time_since_check = std::time::Instant::now() - last_time_check;

        if time_since_check.as_millis() >= 34 {
            // eq: CLoadingScreen::m_bActive
            if !hook::get_global::<bool>(0x10081f460) {
                // eq: bLoadingScene
                if hook::get_global::<bool>(0x10072d530) {
                    // eq: Pump_SwapBuffers()
                    hook::slide::<fn()>(0x100243070)();
                }
            } else {
                // eq: CLoadingScreen::DisplayPCScreen()
                hook::slide::<fn()>(0x1002b54c4)();
            }

            last_time_check = std::time::Instant::now();
        }

        let mut offset = file
            .read_u32::<LittleEndian>()
            .expect("Failed to read entry offset.");
        let mut size_sectors = file
            .read_u16::<LittleEndian>()
            .expect("Failed to read entry size.");
        let archive_size = file
            .read_u16::<LittleEndian>()
            .expect("Failed to read entry archive size.");

        if archive_size != 0 {
            log::warn!("Archive size should be zero, but is {}.", archive_size);
        }

        let mut name_bytes = [0u8; 24];
        file.read_exact(&mut name_bytes)
            .expect("Failed to read entry name.");

        name_bytes[23] = 0;

        let entry_name = unsafe { std::ffi::CStr::from_ptr(name_bytes.as_ptr().cast()) }
            .to_str()
            .unwrap();

        let streaming_buffer_size: u32 =
            hook::get_global::<u32>(0x10072d320).max(size_sectors as u32);

        unsafe {
            *hook::slide::<*mut u32>(0x10072d320) = streaming_buffer_size;
        }

        let entry_path = std::path::Path::new(entry_name);

        let file_extension = entry_path.extension().and_then(|os_str| os_str.to_str());

        let file_extension = if let Some(ext) = file_extension {
            ext.to_lowercase()
        } else {
            log::warn!(
                "Skipping {}/{} because it has no extension.",
                archive_name,
                entry_name
            );

            continue;
        };

        let file_extension = file_extension.as_str();

        let name = entry_path
            .file_stem()
            .expect("Need a name!")
            .to_str()
            .unwrap();

        let name_c = std::ffi::CString::new(name).unwrap();
        let name_c = name_c.as_ptr();

        let mut model_id = 0i32;

        match file_extension {
            "dff" => {
                let mut model_info_ptr: *mut u8 = std::ptr::null_mut();

                let get_entry: extern "C" fn(*mut u8, *mut *mut u8, *mut i32, *const i8) =
                    hook::slide(0x1002d0258);

                // eq: CModelInfoAccelerator::GetEntry(...)
                get_entry(
                    hook::slide(0x1008572e8),
                    &mut model_info_ptr,
                    &mut model_id,
                    name_c,
                );

                // eq: CModelInfo::GetModelInfo(...)
                // let model_info_ptr = hook::slide::<fn(*const i8, *mut i32) -> *mut u8>(0x1002cf864)(
                // name_c,
                // &mut model_id,
                // );

                if model_info_ptr.is_null() {
                    offset = offset | (archive_id as u32) << 24;

                    let extra_objects_dir: *mut u8 = hook::get_global(0x10072d4c0);

                    let entry = Entry {
                        offset,
                        streaming_size: size_sectors,
                        size_in_archive: archive_size,
                        name: name_bytes,
                    };

                    unsafe {
                        let capacity = extra_objects_dir.offset(0x8).cast::<u32>().read();
                        let num_entries = extra_objects_dir.offset(0xc).cast::<u32>().read();

                        // log::trace!("extra objects: {}/{}", num_entries, capacity);

                        if num_entries >= capacity {
                            panic!("too many things");
                        }
                    }

                    // eq: CDirectory::AddItem(...)
                    hook::slide::<fn(*mut u8, *const Entry)>(0x10023712c)(
                        extra_objects_dir,
                        &entry,
                    );

                    previous_model_id = -1;
                    continue;
                } else {
                    // log::trace!("Not extra");
                }
            }

            "txd" => {
                // eq: CTxdStore::FindTxdSlot(...)
                model_id = hook::slide::<fn(*const i8) -> i32>(0x1003a20d8)(name_c);

                if model_id == -1 {
                    // eq: CTxdStore::AddTxdSlot(...)
                    model_id = hook::slide::<fn(*const i8) -> i32>(0x1003a1cdc)(name_c);
                }

                model_id += 20000;
            }

            "col" => {
                // fixme: This is always -1, so we don't need it.
                // eq: CColStore::FindColSlot(...)
                model_id = hook::slide::<fn(*const i8) -> i32>(0x10018aab4)(name_c);

                if model_id == -1 {
                    // eq: CColStore::AddColSlot(...)
                    model_id = hook::slide::<fn(*const i8) -> i32>(0x10018a548)(name_c);
                }

                model_id += 25000;
            }

            "ipl" => {
                // eq: CIplStore::FindIplSlot(...)
                model_id = hook::slide::<fn(*const i8) -> i32>(0x1001762c4)(name_c);

                if model_id == -1 {
                    // eq: CIplStore::AddIplSlot(...)
                    model_id = hook::slide::<fn(*const i8) -> i32>(0x100175dd8)(name_c);
                }

                model_id += 25255;
            }

            "dat" => {
                // Should be a nodes*.dat file, so we need to find what the '*' is.
                if name.len() < 6 {
                    log::error!("Cannot parse .dat name '{}' because it is too short.", name);
                    continue;
                }

                model_id = i32::from_str_radix(&name[5..], 10)
                    .expect("Could not parse DAT number.")
                    + 25511;
            }

            "ifp" => {
                // eq: CAnimManager::RegisterAnimBlock(...)
                model_id = hook::slide::<fn(*const i8) -> i32>(0x100141034)(name_c) + 25575;
            }

            "rrr" => {
                // eq: CVehicleRecording::RegisterRecordingFile(...)
                model_id = hook::slide::<fn(*const i8) -> i32>(0x1001c9c34)(name_c) + 25755;
            }

            "scm" => {
                let the_scripts: *mut u8 = hook::slide(0x1007a42a0);

                // eq:: CStreamedScripts::RegisterScript(...)
                model_id =
                    hook::slide::<fn(*mut u8, *const i8) -> i32>(0x1001fd3b4)(the_scripts, name_c)
                        + 26230;
            }

            _ => {
                log::warn!(
                    "{}/{} has unknown extension '{}'.",
                    name,
                    entry_name,
                    file_extension
                );

                continue;
            }
        }

        let model_info_arr: *mut StreamingInfo = hook::slide(0x1006ac8f4);
        let info = unsafe { model_info_arr.offset(model_id as isize).as_mut().unwrap() };

        // Fill in details if we don't already have them.
        if info.cd_size != 0 {
            log::trace!("Already have size and position info.");
            previous_model_id = -1;
            panic!();
        } else {
            info.img_id = archive_id as u8;

            if archive_size != 0 {
                size_sectors = archive_size;
            }

            info.cd_pos = offset;
            info.cd_size = size_sectors as u32;
            info.flags = 0;

            if (offset as u32).overflowing_mul(2048).1 {
                log::error!("end {}/{}", archive_name, entry_name);
                panic!();
            }

            if previous_model_id != -1 {
                unsafe {
                    let previous = model_info_arr
                        .offset(previous_model_id as isize)
                        .as_mut()
                        .unwrap();

                    previous.next_index_on_cd = model_id as i16;
                }
            }

            previous_model_id = model_id;

            // log::trace!("{:#?}", info);
        }
    }
}

fn log_all_models() {
    let model_info_arr: *mut StreamingInfo = hook::slide(0x1006ac8f4);

    for i in 0..26316 {
        let info = unsafe { model_info_arr.offset(i as isize).as_ref().unwrap() };
        log::trace!("{:#?}", info);
    }
}

fn load_cd_directory(path: *const i8, archive_id: i32) {
    let path = unsafe { std::ffi::CStr::from_ptr(path) }
        .to_str()
        .expect("Unable to convert path C string to Rust.");

    load_directory(path, archive_id);
    log_all_models();
}

pub fn hook() {
    const CD_STREAM_INIT: crate::hook::Target<fn(i32)> = crate::hook::Target::Address(0x100177eb8);
    CD_STREAM_INIT.hook_hard(stream_init);

    const CD_STREAM_READ: crate::hook::Target<fn(u32, *mut u8, StreamSource, u32) -> bool> =
        crate::hook::Target::Address(0x100178048);
    CD_STREAM_READ.hook_hard(stream_read);

    const CD_STREAM_OPEN: crate::hook::Target<fn(*const c_char, bool) -> i32> =
        crate::hook::Target::Address(0x1001782b0);
    CD_STREAM_OPEN.hook_hard(stream_open);

    const LOAD_ARCHIVE: crate::hook::Target<fn(*const c_char, i32)> =
        crate::hook::Target::Address(0x1002f0e18);
    LOAD_ARCHIVE.hook_hard(load_cd_directory);
}

#[repr(C)]
#[derive(Debug)]
struct StreamingInfo {
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
struct StreamFileInfo {
    name: [i8; 40],
    not_player_img: bool,
    _pad: [u8; 3],
    stream_handle: i32,
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

    #[allow(dead_code)]
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
