use std::{
    collections::HashMap,
    ffi::CStr,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Mutex,
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{call_original, hook};

use super::game::{ImageRegion, StreamSource};

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
                            "{} at ({}, {}) will be replaced with new size {}",
                            name,
                            info.img_id,
                            info.cd_pos,
                            child.size_in_segments()
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

fn load_archive_file(path: &str, _image_id: i32) -> eyre::Result<()> {
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

    let _entries = DirectoryEntry::read_entries(entry_count, &mut reader)?;

    Ok(())
}

/// Returns a map of regions and their respective replacement file paths for the image called
/// `name`.
pub fn region_swaps_for_image_name(name: &str) -> Option<&'static HashMap<ImageRegion, PathBuf>> {
    lazy_static::lazy_static! {
        /// Maps image names to region replacements.
        ///
        /// Image names should be uppercase because the game uses uppercase. If we used lowercase,
        /// we would have to turn every `&str` from the game into a `String` using `to_lowercase`,
        /// which would be bad because that would happen very frequently.
        static ref MAPS: HashMap<String, HashMap<ImageRegion, PathBuf>> = HashMap::from([(
            "GTA3.IMG".to_string(),
            HashMap::from([(
                ImageRegion {
                    offset_sectors: 88827,
                    size_sectors: 2974,
                },
                crate::resources::get_documents_path("CLEO/gta3.img/clover.dff"),
            )]),
        )]);
    };

    // The image names are actually Windows-style path segments, but since we only want the file
    // name ("x.img"), we only consider the part after the final backslash.
    let name = name.rsplit('\\').next().expect("Empty image name");

    MAPS.get(name)
}

/// Hooks the loading system for CD images.
pub fn hook() {
    crate::targets::load_cd_directory::install(load_directory);
}
