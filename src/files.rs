use std::{
    io::{Error, ErrorKind, Result},
    path::Path,
};

use log::{debug, info, warn};

use crate::scripts;

fn load_language_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let comment_pattern: regex::Regex = regex::Regex::new(r"//|#").unwrap();

    for line in std::fs::read_to_string(path)?.lines() {
        let line = comment_pattern
            .split(line)
            .next()
            .and_then(|s| Some(s.trim()));

        if let Some(line) = line {
            if line.is_empty() {
                continue;
            }

            // split_once isn't stable yet, so we have to do this.
            let mut split = line.splitn(2, ' ');
            let (key, value) = (split.next(), split.next());

            if key.is_none() || value.is_none() {
                warn!("Unable to find key and value in line '{}'", line);
                continue;
            }

            crate::text::set_kv(key.unwrap(), value.unwrap());
        }
    }

    Ok(())
}

fn load_csa_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let result = scripts::Script::new(&path);

    debug!(
        "scripts::Script::new({:#?}) ==> {:#?}",
        path.as_ref().to_str(),
        result
    );

    scripts::loaded_scripts().push(result?);
    Ok(())
}

fn load_csi_file<P: AsRef<Path>>(path: P) -> Result<()> {
    warn!("CSI loading not yet available.");
    Ok(())
}

fn load_path<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if path.is_dir() {
        return load_all(path);
    }

    match path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or(Error::new(ErrorKind::InvalidInput, "Extension required"))?
        .to_lowercase()
        .as_str()
    {
        "fxt" => Ok(load_language_file(path)?),
        "csa" => Ok(load_csa_file(path)?),
        "csi" => Ok(load_csi_file(path)?),

        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            "Unrecognised extension",
        )),
    }
}

pub fn load_all<P: AsRef<Path>>(dir_path: P) -> Result<()> {
    info!("Loading files from {:?}", dir_path.as_ref().to_path_buf());

    let directory = std::fs::read_dir(dir_path)?;

    for item in directory {
        if let Ok(entry) = item {
            let result = load_path(entry.path());

            if let Err(err) = result {
                warn!("Unable to load {:#?}: {}", entry.path(), err);
            } else {
                info!("Loaded resource {:#?}", entry.path());
            }
        }
    }

    Ok(())
}
