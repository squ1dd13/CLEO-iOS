//! High-level abstractions for working with OSW sound effect packages.

use byteorder::{ReadBytesExt, LE};
use case_insensitive_hashmap::CaseInsensitiveHashMap as UnicaseHashMap;
use eyre::{eyre, Context, Result};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

/// An area within an OSW file.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OswRegion {
    /// The offset of this region from the beginning of the file in bytes.
    offset: u32,

    /// The size of this region in bytes.
    size: u32,
}

/// Refers to an entry in an OSW file.
struct OswEntry {
    /// The name of this entry.
    name: String,

    /// The area of the OSW file that contains this entry's data.
    region: OswRegion,
}

impl OswEntry {
    /// Reads a single entry from `reader`.
    fn read(reader: &mut impl Read) -> Result<OswEntry> {
        let offset = reader.read_u32::<LE>().wrap_err("offset")?;
        let size = reader.read_u32::<LE>().wrap_err("size")?;

        // The name is a variable-length string, represented as a `u16` length value followed by as
        // many characters as indicated by the length value.
        let name = String::from_utf8({
            let name_len = reader.read_u16::<LE>().wrap_err("name length")?;

            let mut name_bytes = vec![0u8; name_len as usize];
            reader.read_exact(&mut name_bytes)?;

            name_bytes
        })?;

        Ok(OswEntry {
            name,
            region: OswRegion { offset, size },
        })
    }

    /// Destructures the entry into its name and data region.
    fn into_pair(self) -> (String, OswRegion) {
        (self.name, self.region)
    }
}

/// Loads the entries from an OSW index file (`.osw.idx`).
fn read_osw_index(path: impl AsRef<Path>) -> Result<Vec<OswEntry>> {
    // Use a buffered reader because we're going to do a lot of small reads.
    let mut reader = BufReader::new(File::open(path).wrap_err("failed to open index file")?);

    let entry_count = reader
        .read_u32::<LE>()
        .wrap_err("error reading entry count")?;

    // Allocate a vector large enough to hold all of the entries, and then read all of the entry
    // data into it.
    let mut entries = Vec::with_capacity(entry_count as usize);

    for _ in 0..entry_count {
        entries.push(OswEntry::read(&mut reader).wrap_err("error reading entry")?);
    }

    Ok(entries)
}

/// Wrapper around an open OSW file providing a high-level interface for reading the contained
/// files.
pub struct OswFile {
    /// The entries in this OSW file, keyed by name.
    entries: UnicaseHashMap<OswRegion>,

    /// The OSW file.
    file: File,
}

impl OswFile {
    /// Opens the OSW file at `data_path`, reading the entry data from `index_path`.
    pub fn open(data_path: impl AsRef<Path>, index_path: impl AsRef<Path>) -> Result<OswFile> {
        let entry_vec = read_osw_index(index_path).wrap_err("error reading index")?;

        // Create a map from the vector of entries so we can search through them more efficiently.
        let entry_map = UnicaseHashMap::from_iter(entry_vec.into_iter().map(OswEntry::into_pair));

        Ok(OswFile {
            entries: entry_map,
            file: File::open(data_path).wrap_err("failed to open data file")?,
        })
    }

    /// Returns an iterator over the file names and regions in this OSW file.
    pub fn contents(&self) -> impl Iterator<Item = (&str, OswRegion)> {
        self.entries
            .iter()
            .map(|(name, region)| (name.as_str(), *region))
    }

    /// Returns the region that holds the data of the file called `name` within this OSW file.
    pub fn region(&self, name: impl AsRef<str>) -> Option<OswRegion> {
        self.entries.get(name.as_ref()).copied()
    }

    /// Reads the data from `region` in this OSW file into `buf`. `buf` should be at least as large
    /// as `region.size`.
    pub fn read_region(&mut self, region: OswRegion, buf: &mut [u8]) -> Result<()> {
        // Try to obtain a subslice of `buf` that is exactly the size we need for `region`. This
        // allows us to use `read_exact` later.
        let buf = buf
            .get_mut(..region.size as usize)
            .ok_or_else(|| eyre!("buffer length is smaller than region size {}", region.size))?;

        self.file
            .seek(SeekFrom::Start(region.offset as u64))
            .wrap_err("error while seeking")?;

        self.file.read_exact(buf).wrap_err("error while reading")
    }

    /// Extracts all of the files from `self` into the directory at `dir_path`.
    pub fn dump(&mut self, dir_path: impl AsRef<Path>) -> Result<()> {
        if self.entries.is_empty() {
            return Ok(());
        }

        // Unwrap here because there will always be a maximum value when the map isn't empty.
        let max_size = self
            .entries
            .values()
            .map(|region| region.size)
            .max()
            .unwrap();

        // Create a single buffer to use with all of the files.
        let mut buf = vec![0u8; max_size as usize];

        for (name, region) in self.entries.iter() {
            let buf = buf.get_mut(..region.size as usize).unwrap();

            self.file
                .seek(SeekFrom::Start(region.offset as u64))
                .wrap_err("error while seeking")?;

            self.file.read_exact(buf).wrap_err("error while reading")?;

            let path = dir_path.as_ref().join(name.as_str());

            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir(parent).wrap_err("creating subdirectory")?;
                }
            }

            std::fs::write(
                dir_path.as_ref().join(name.as_str()),
                &buf[..region.size as usize],
            )
            .wrap_err("writing file")?;
        }

        Ok(())
    }
}
