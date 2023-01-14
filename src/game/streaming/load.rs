//! Hooks into the game's CD image loading system in order to let us load custom resources.
//!
//! This module is responsible for finding out which resources are to be replaced with custom
//! files, exposing this data for the `stream` module to reference while the game is running, and
//! modifying the resource metadata to match the replacements.

use std::{
    collections::{hash_map::OccupiedError, HashMap},
    ffi::CStr,
    io::Read,
    path::{Path, PathBuf},
    sync::Mutex,
};

use byteorder::{LittleEndian, ReadBytesExt};
use case_insensitive_hashmap::CaseInsensitiveHashMap;
use eyre::{format_err, Context, Result};
use itertools::Itertools;

use crate::{call_original, hook};

use super::game::ImageRegion;

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

impl StreamingInfo {
    /// Returns the global streaming info slice.
    fn global() -> &'static mut [StreamingInfo] {
        let array: *mut StreamingInfo = hook::slide(0x1006ac8f4);

        // The length is always 26316.
        unsafe { std::slice::from_raw_parts_mut(array, 26316) }
    }

    /// Returns an iterator over mutable references to the streaming info structures with the given
    /// image ID.
    fn for_image_id(image_id: u8) -> impl Iterator<Item = &'static mut StreamingInfo> {
        StreamingInfo::global()
            .iter_mut()
            .filter(move |info| info.image_id() == image_id)
    }

    /// Modifies the size that this streaming info structure reports.
    fn set_size_segments(&mut self, segments: usize) {
        self.cd_size = segments as u32;
    }

    /// Returns the ID of the image that this streaming info structure is for.
    fn image_id(&self) -> u8 {
        self.img_id
    }

    /// Returns the region of its parent image that this streaming info refers to.
    fn region(&self) -> ImageRegion {
        ImageRegion {
            offset_sectors: self.cd_pos as usize,
            size_sectors: self.cd_size as usize,
        }
    }
}

pub fn load_replacements(image_name: impl AsRef<str>, paths: impl Iterator<Item = PathBuf>) {
    let mut mapper = ReplacementMapper::shared();

    if let Err(errors) = mapper.register_paths_for_image(image_name.as_ref(), paths) {
        for error in errors {
            log::warn!(
                "Error loading replacement for '{}': {:?}",
                image_name.as_ref(),
                error
            );
        }
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

fn path_name(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();

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

    Ok(path
        .file_name()
        .ok_or_else(no_name)?
        .to_str()
        .ok_or_else(not_utf8)?
        // Our `&str` refers to a temporary value, so create an owned `String`.
        .to_string())
}

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

    /// Maps real image offsets to the paths to the files that replace them.
    replacements_by_region: HashMap<usize, PathBuf>,
}

impl ImageMapper {
    /// Attempts to convert `path` to a tuple containing the file name and original `path`.
    fn path_to_replacement_pair(path: PathBuf) -> Result<(String, PathBuf)> {
        Ok((path_name(&path)?, path))
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
            log::info!("Installing replacement '{:?}' for '{}'.", path, file_name);

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

        log::info!(
            "{} entr{} to map",
            unmapped_regions,
            if unmapped_regions == 1 { "y" } else { "ies" }
        );

        for entry in entries {
            // Check if we've got a replacement registered with the same name as this entry. If we
            // have, we can map the entry's offset to the replacement's file path.
            if let Some(replacement_path) = self.replacements_by_name.get(&entry.name) {
                self.replacements_by_region
                    .insert(entry.region.offset_sectors, replacement_path.clone());

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

    /// Returns the path to the file that contains the replacement data for `region`, or `None` if
    /// the region has no replacement.
    fn replacement_path(&self, region: ImageRegion) -> Option<&PathBuf> {
        self.replacements_by_region.get(&region.offset_sectors)
    }

    /// Returns the number of segments occupied by the data that will be used for `region`. For
    /// unswapped regions, this will be the same as the region's size. For regions that have been
    /// replaced, this may differ, as it will be the size of the file's data instead.
    fn real_segment_count(&self, region: ImageRegion) -> Result<usize> {
        let path = self.replacement_path(region);

        match path {
            Some(path) => {
                let metadata = path.metadata().wrap_err_with(|| {
                    eyre::format_err!(
                        "Unable to get size of replacement file '{:?}' for region {:?}",
                        path,
                        region
                    )
                })?;

                let file_size_bytes = metadata.len() as usize;

                // Round up to the next segment boundary. If the byte count is already
                // segment-aligned, this will leave it as-is.
                let file_size_segments = (file_size_bytes + 2047) / 2048;

                Ok(file_size_segments)
            }

            None => Ok(region.size_sectors),
        }
    }

    /// Removes the replacement mapped for `region`, if such a mapping exists.
    fn remove_replacement(&mut self, region: ImageRegion) {
        self.replacements_by_region.remove(&region.offset_sectors);
    }
}

/// Stores information about the resource replacements for all images.
pub struct ReplacementMapper {
    /// Mappers for the images, keyed by image name.
    image_mappers: CaseInsensitiveHashMap<ImageMapper>,
}

impl ReplacementMapper {
    /// Returns the shared replacement mapper.
    pub fn shared() -> std::sync::MutexGuard<'static, ReplacementMapper> {
        lazy_static::lazy_static! {
            static ref MAPPER: Mutex<ReplacementMapper> = Mutex::new(ReplacementMapper {
                image_mappers: CaseInsensitiveHashMap::new(),
            });
        }

        MAPPER.lock().unwrap()
    }

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
        log::info!("Registering entries: {}", image_name.as_ref());

        let mapper = if let Some(mapper) = self.image_mappers.get_mut(image_name.as_ref()) {
            mapper
        } else {
            log::info!("No replacements, skipping");

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
                    Is the name correct? Is the file in the correct folder? \
                    If this file is not intended to be a replacement, \
                    consider removing it from the replacement directory \
                    to speed up loading times.",
                    error.0,
                    image_name_owned.clone()
                )
            };

            Err(errors.map(explain).collect_vec())
        } else {
            Ok(())
        }
    }

    /// Returns the path to the replacement file used for the data in `region` of the specified
    /// image.
    pub fn replacement_path(
        &self,
        image_name: impl AsRef<str>,
        region: ImageRegion,
    ) -> Option<&PathBuf> {
        self.image_mappers
            .get(image_name.as_ref())?
            .replacement_path(region)
    }

    /// Returns the number of segments that the actual data for `region` in the specified image
    /// will take up. This may differ from the region's size if the region has been replaced.
    fn real_segment_count(
        &self,
        image_name: impl AsRef<str>,
        region: ImageRegion,
    ) -> Result<usize> {
        match self.image_mappers.get(image_name.as_ref()) {
            Some(mapper) => mapper.real_segment_count(region),
            None => Ok(region.size_sectors),
        }
    }

    /// Removes the replacement mapped to `region` in the specified image, if one exists.
    fn remove_replacement(&mut self, image_name: impl AsRef<str>, region: ImageRegion) {
        if let Some(mapper) = self.image_mappers.get_mut(image_name.as_ref()) {
            mapper.remove_replacement(region);
        }
    }
}

/// Ensures that the global streaming buffer is at least as many as `segment_count` segments in
/// size.
fn reserve_streaming_buffer(segment_count: usize) {
    let buf_size_segments = unsafe { &mut *hook::slide::<*mut u32>(0x10072d320) };
    *buf_size_segments = (*buf_size_segments).max(segment_count as u32);
}

/// Adjusts the recorded sizes of any resources within an image that have been replaced with files.
/// This is necessary to ensure that the game always allocates enough memory for a resource before
/// it is loaded.
fn update_resource_sizes(
    mapper: &mut ReplacementMapper,
    image_name: impl AsRef<str>,
    image_id: u32,
) {
    // The game needs to know the largest amount of data it could possibly load so that it can
    // allocate enough space for it, so we keep a record of the largest size we find.
    let mut max_entry_size_segments = 0usize;

    // Go through the streaming info and correct the size values.
    for streaming_info in StreamingInfo::for_image_id(image_id as u8) {
        let region = streaming_info.region();

        match mapper.real_segment_count(&image_name, region) {
            Ok(segment_count) => {
                // This won't change the value for most entries, since most entries won't have
                // replacements mapped.
                streaming_info.set_size_segments(segment_count);

                max_entry_size_segments = max_entry_size_segments.max(segment_count);
            }

            Err(count_err) => {
                log::error!(
                    "Size calculation failed: {:?}.\
                    Replacement for region {:?} in image '{}' will be removed.",
                    count_err,
                    region,
                    image_name.as_ref()
                );

                // Since we couldn't get the file size, we can't update the value that the game
                // holds for the size of this resource. That means we could end up trying to load
                // more data than the game is expecting, which would be Very Bad. To prevent that,
                // we simply remove the replacement.
                mapper.remove_replacement(&image_name, region);
            }
        }
    }

    // Ensure that the streaming buffer is at least as large as the maximum size we found here.
    reserve_streaming_buffer(max_entry_size_segments);
}

/// Opens the image file at `path` and maps its contents into `mapper`.
fn map_replacements(mapper: &mut ReplacementMapper, path: &str, image_id: u32) -> Result<()> {
    let file = std::fs::File::open(path).wrap_err("Failed to open image file")?;

    // Use a buffered reader because we do a lot of small sequential reads.
    let mut reader = std::io::BufReader::new(file);

    let mut identifier = [0u8; 4];
    reader.read_exact(&mut identifier)?;

    eyre::ensure!(
        &identifier == b"VER2",
        "Bad identifier in archive '{}': {:?} (expected 'VER2')",
        path,
        identifier
    );

    let image_name = path_name(path)?;

    let entry_count = reader.read_u32::<LittleEndian>()? as usize;
    let entries = DirectoryEntry::read_entries(entry_count, &mut reader)?;

    let log_error = |result| match result {
        Ok(entry) => Some(entry),
        Err(err) => {
            log::error!(
                "Error reading entry from directory of '{}': {:?}",
                path,
                err
            );

            None
        }
    };

    let filtered_entries = entries.filter_map(log_error);
    let registration_result = mapper.register_entries_for_image(&image_name, filtered_entries);

    if let Err(reg_errors) = registration_result {
        // We only log the errors as warnings because they're more likely to be a result of user
        // error than of an issue with CLEO or the game.
        for error in reg_errors {
            log::warn!("Registration error: {}", error);
        }
    }

    update_resource_sizes(mapper, &image_name, image_id);

    Ok(())
}

/// Hook for `CStreaming::LoadCdDirectory`, which the game uses to open image files and register
/// their contents with various game systems before streaming actually begins. We hook it so that
/// we can modify the data from the image immediately after registration, allowing us to load
/// custom resource data later on.
fn load_cd_directory_hook(path: *const i8, image_id: u32) {
    call_original!(crate::targets::load_cd_directory, path, image_id);

    let mut mapper = ReplacementMapper::shared();

    let windows_path = unsafe { CStr::from_ptr(path) }
        .to_str()
        .expect("Unable to convert image path to string slice");

    // Convert the Windows path to a real file path.
    let path = match crate::loader::find_absolute_path(&windows_path.to_lowercase()) {
        Some(path) => path,
        None => {
            log::error!("Unable to resolve image path '{}'", windows_path);
            return;
        }
    };

    if let Err(err) = map_replacements(&mut mapper, &path, image_id) {
        log::error!(
            "Encountered error when trying to map replacements for '{}': {:?}",
            path,
            err
        );
    }
}

/// Hooks the loading system for CD images.
pub fn hook() {
    crate::targets::load_cd_directory::install(load_cd_directory_hook);
}
