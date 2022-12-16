use std::{
    collections::HashMap,
    ffi::CStr,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::Mutex,
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{call_original, hook};

use super::game::StreamSource;

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

fn load_directory(path_c: *const i8, archive_id: i32) {
    let path = unsafe { CStr::from_ptr(path_c) }.to_str().unwrap();

    let (path, archive_name) = get_archive_path(path).expect("Unable to resolve path name.");

    log::info!("Registering contents of archive '{}'.", archive_name);

    if let Err(err) = load_archive_into_database(&path, archive_id) {
        log::error!("Failed to load archive: {}", err);
        call_original!(crate::targets::load_cd_directory, path_c, archive_id);
        return;
    } else {
        log::info!("Registered archive contents successfully.");
    }

    call_original!(crate::targets::load_cd_directory, path_c, archive_id);

    let streaming_info_arr: *mut StreamingInfo = hook::slide(0x1006ac8f4);

    with_model_names(|model_names| {
        with_replacements(&mut |replacements| {
            let empty = HashMap::new();

            let replacement_map = if let Some(map) = replacements.get(&archive_name) {
                map
            } else {
                &empty
            };

            // 26316 is the total number of entries in the streaming info array.
            for i in 0..26316 {
                let info = unsafe { streaming_info_arr.offset(i as isize).as_mut().unwrap() };
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

/// Hooks the loading system for CD images.
pub fn hook() {
    crate::targets::load_cd_directory::install(load_directory);
}
