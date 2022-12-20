//! Hooks into the game's CD image loading system in order to let us load custom resources.
//!
//! This module is responsible for finding out which resources are to be replaced with custom
//! files, exposing this data for the `stream` module to reference while the game is running, and
//! modifying the resource metadata to match the replacements.

use std::{
    collections::{hash_map::OccupiedError, HashMap},
    ffi::CStr,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Mutex,
};

use byteorder::{LittleEndian, ReadBytesExt};
use eyre::{format_err, Context, Result};
use itertools::Itertools;

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
fn load_archive_into_database(path: &str, img_id: i32) -> Result<()> {
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

pub fn load_replacement(image_name: &str, path: &impl AsRef<Path>) -> Result<()> {
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
    fn read(reader: &mut impl std::io::BufRead, name_buf: &mut [u8; 24]) -> Result<DirectoryEntry> {
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

        let name = name.to_lowercase();

        Ok(DirectoryEntry { name, region })
    }

    /// Attempts to read `count` directory entries from `reader`.
    fn read_entries(
        count: usize,
        reader: &mut impl std::io::BufRead,
    ) -> Result<impl Iterator<Item = eyre::Result<DirectoryEntry>> + '_> {
        // Create a single buffer for temporarily holding the bytes of each name entry during
        // processing.
        let mut name_buf = [0u8; 24];

        let read_entry =
            move |_| -> Result<DirectoryEntry> { DirectoryEntry::read(reader, &mut name_buf) };

        Ok((0..count).map(read_entry))
    }
}

// stream knows region and image only
// load knows names and regions

// mapper creates a map of theoretical resource names to paths from all of the image replacement
// resources it finds, entry e.g. ["clover.dff", "/path/to/replacement/clover.dff"]

// when an image is loaded in, the mapper looks through all of the entries and finds the ones that
// have replacements in the name-path map

// any matches are added to a map of image regions and names to replacement paths: e.g.
// [("gta3.img", { clover region }), "/path/to/replacement/clover.dff"]

// after the image is loaded, the mapper is consulted again to find out which models need their
// sizes changing in the streaming info array (so that the game allocates enough space)

// stream can then create a key in the form (image name, requested region) for each resource load,
// and use the mapper to find if there's a replacement file for that region without even knowing
// what the file name is

/// An error containing a reference to a path which didn't match any real entries in an image.
struct UnmappedPathErr<'path>(&'path PathBuf);

/// Stores information about the resource replacements for an individual image.
struct ImageMapper {
    /// Maps theoretical resource paths to their replacement file paths.
    ///
    /// The resource paths are "theoretical" because this map is generated from the paths with no
    /// reference to which resources actually exist in the game. Those which do not actually exist
    /// will simply go unused.
    ///
    /// Names in this map should be lowercase.
    replacements_by_name: HashMap<String, PathBuf>,

    /// Maps real image regions to the paths to the files that replace them.
    replacements_by_region: HashMap<ImageRegion, PathBuf>,
}

impl ImageMapper {
    /// Attempts to convert `path` to a tuple containing the file name and original `path`.
    fn path_to_replacement_pair(path: PathBuf) -> Result<(String, PathBuf)> {
        let no_name = || {
            format_err!(
                "Cannot use '{:?}' as an archive replacement as it doesn't have a file name",
                path
            )
        };

        let not_utf8 = || {
            format_err!(
                "Cannot use '{:?}' as an archive replacement as it isn't valid UTF-8",
                path
            )
        };

        let file_name = path
            .file_name()
            .ok_or_else(no_name)?
            .to_ascii_lowercase()
            .to_str()
            .ok_or_else(not_utf8)?
            // Our `&str` refers to a temporary value, so create an owned `String`.
            .to_string();

        Ok((file_name, path))
    }

    /// Registers the contents of `paths` as replacement file paths for this image.
    ///
    /// None of the errors encountered in this method are fatal, so the method will not return
    /// early upon generating one. Instead, errors are collected in a `Vec`. If no errors are
    /// encountered, this method will return `Ok(())`, but if any errors have been collected, the
    /// vector will be returned inside an `Err`.
    fn register_paths(
        &mut self,
        paths: impl Iterator<Item = PathBuf>,
    ) -> Result<(), Vec<eyre::Report>> {
        let (replacements, mut errors): (Vec<_>, Vec<_>) = paths
            .map(ImageMapper::path_to_replacement_pair)
            .partition_result();

        for (file_name, path) in replacements {
            let duplicate_err = |occupied_err: OccupiedError<_, _>| {
                format_err!(
                    "Duplicate replacements for resource '{}'",
                    occupied_err.entry.key()
                )
            };

            // Insert the replacement, or generate an error if there is already a replacement for
            // the resource name.
            if let Err(err) = self
                .replacements_by_name
                .try_insert(file_name, path)
                .map_err(duplicate_err)
            {
                errors.push(err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Uses `entries` to add region information to the file replacements.
    ///
    /// The error type is a boxed iterator of errors. One error is generated for each unmapped file
    /// path that remains after searching through all of the entries.
    fn register_entries(
        &mut self,
        entries: impl Iterator<Item = DirectoryEntry>,
    ) -> Result<(), Box<impl Iterator<Item = UnmappedPathErr>>> {
        // If we don't have any replacements, we don't need to look through the entries.
        if self.replacements_by_name.is_empty() {
            return Ok(());
        }

        // Keep a record of the number of replacements we've got left to map to regions. When this
        // reaches zero, we can stop looking through the entries.
        let mut unmapped_regions = self.replacements_by_name.len();

        for entry in entries {
            // Check if we've got a replacement registered with the same name as this entry. If we
            // have, we can map the entry's region to the replacement's file path.
            if let Some(replacement_path) = self.replacements_by_name.get(&entry.name) {
                self.replacements_by_region
                    .insert(entry.region, replacement_path.clone());

                unmapped_regions -= 1;

                if unmapped_regions == 0 {
                    // Successfully mapped all the regions, so we're done.
                    return Ok(());
                }
            }
        }

        // If there are unmapped regions remaining, not all of the files with replacements
        // registered actually have entries. We should warn the user about this, so they don't get
        // confused if one of their replacements isn't working due to a typo in the name.
        if unmapped_regions > 0 {
            // Find which replacement paths weren't mapped by creating an iterator from the mapped
            // paths and the registered paths. Every path is registered, so unmapped paths will
            // only appear once in the resulting iterator.
            let unmapped_path_errors = self
                .replacements_by_name
                .values()
                .chain(self.replacements_by_region.values())
                .unique()
                .map(UnmappedPathErr);

            // Box the iterator to reduce the memory footprint of the `Err` case.
            return Err(Box::new(unmapped_path_errors));
        }

        Ok(())
    }
}

/// Stores information about the resource replacements for all images.
struct ReplacementMapper {
    /// Mappers for the images, keyed by image name.
    image_mappers: HashMap<String, ImageMapper>,
}

impl ReplacementMapper {
    /// Registers the contents of `paths` as replacement file paths for the image with name
    /// `image_name`.
    ///
    /// Returns nothing on success, or a vector of errors if any were encountered.
    fn register_paths_for_image(
        &mut self,
        image_name: impl AsRef<str>,
        paths: impl Iterator<Item = PathBuf>,
    ) -> Result<(), Vec<eyre::Report>> {
        // Get a mutable reference to the mapper for this image, creating a new mapper if there
        // isn't one already.
        let mapper = self
            .image_mappers
            .entry(image_name.as_ref().to_lowercase())
            .or_insert_with(|| ImageMapper {
                replacements_by_name: HashMap::new(),
                replacements_by_region: HashMap::new(),
            });

        let registering_result = mapper.register_paths(paths);

        // Wrap any errors with a message making their source clearer.
        registering_result.map_err(|errors| {
            errors
                .into_iter()
                .map(|err| {
                    err.wrap_err(format!(
                        "While adding replacements to '{}'",
                        image_name.as_ref()
                    ))
                })
                .collect()
        })
    }

    /// Uses `entries` to add region information to the file paths registered as replacements for
    /// the specified image.
    ///
    /// This method will return an error for every unmapped file path that remains after processing
    /// all of the entries.
    fn register_entries_for_image(
        &mut self,
        image_name: impl AsRef<str>,
        entries: impl Iterator<Item = DirectoryEntry>,
    ) -> Result<(), Vec<eyre::Report>> {
        let mapper = if let Some(mapper) = self
            .image_mappers
            .get_mut(&image_name.as_ref().to_lowercase())
        {
            mapper
        } else {
            // If we don't have a mapper for this image, there's no point looking through the
            // entries, since we don't have any replacements to worry about.
            return Ok(());
        };

        if let Err(errors) = mapper.register_entries(entries) {
            let image_name_owned = image_name.as_ref().to_string();

            // Create a closure that turns an unmapped path error into an error report with an
            // explanation.
            let explain = move |error: UnmappedPathErr| {
                eyre::format_err!(
                    "'{:?}' does not replace anything in '{}'. \
Is the name correct? \
If this file is not intended to be a replacement, \
consider removing it from the replacement directory to speed up loading times.",
                    error.0,
                    image_name_owned.clone()
                )
            };

            Err(errors.map(explain).collect_vec())
        } else {
            Ok(())
        }
    }
}

fn load_archive_file(path: &str, _image_id: i32) -> Result<()> {
    // Use a buffered reader because we do a lot of small sequential reads.
    let mut reader = std::io::BufReader::new(std::fs::File::open(path)?);

    let mut identifier = [0u8; 4];
    reader.read_exact(&mut identifier)?;

    if &identifier != b"VER2" {
        return Err(format_err!(
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
