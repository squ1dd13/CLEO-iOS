//! Creates a flattened version of the CLEO directory and delegates to other modules for
//! handling specific types of resources.

use crate::game::{loader, scripts, streaming, text};
use cached::proc_macro::cached;
use itertools::Itertools;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

/*
    Documents       The game's documents directory.
      CLEO
        x.csa       A script that is loaded when the game loads and should only exit when the game does.
        x.csi       A script that is launched by the user from the CLEO menu.
        x.fxt       A file containing text definitions that are added to the game's localisation system for use by scripts.

        x.img       A older containing replacements for the files inside the x.img archive in the game's folder.
          example   A file that replaces the file named 'example' that is normally loaded from inside x.img.

        Replace     A folder containing replacements for game files.
          example   A file that replaces the file named 'example' in the game's folder.

    The job of this module is to flatten this structure to make it easier for other modules to find the files they want.
*/

#[derive(Debug)]
enum ModResource {
    // CSA script.
    StartupScript(PathBuf),

    // CSI script.
    InvokedScript(PathBuf),

    // FXT language file.
    LanguageFile(PathBuf),

    // Anything inside a top-level folder with the extension "img".
    // First value is the image name.
    StreamReplacement(String, PathBuf),

    // A file from the "Replace" folder.
    FileReplacement(PathBuf),
}

impl Display for ModResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModResource::StartupScript(path) => {
                write!(f, "startup script {:?}", path.file_name().unwrap())
            }
            ModResource::InvokedScript(path) => {
                write!(f, "invoked script {:?}", path.file_name().unwrap())
            }
            ModResource::LanguageFile(path) => {
                write!(f, "language file {:?}", path.file_name().unwrap())
            }
            ModResource::StreamReplacement(img_name, path) => write!(
                f,
                "replacement file {:?} for archive \"{}\"",
                path.file_name().unwrap(),
                img_name
            ),
            ModResource::FileReplacement(path) => write!(
                f,
                "general file replacement {:?}",
                path.file_name().unwrap()
            ),
        }
    }
}

impl ModResource {
    fn flatten_dir(path: &impl AsRef<Path>) -> Option<Vec<ModResource>> {
        let path = path.as_ref();
        let mut resources = vec![];

        for entry in path.read_dir().ok()? {
            let entry = if let Err(err) = entry {
                log::warn!("Error while reading resources from directory: {}", err);
                continue;
            } else {
                entry.unwrap()
            };

            let entry_path = entry.path();

            if let Some(resource) = Self::from_path(&entry_path) {
                resources.push(resource);
            } else if entry_path.is_dir() {
                if let Some(mut found) = Self::flatten_dir(&entry_path) {
                    resources.append(&mut found);
                }
            }
        }

        Some(resources)
    }

    fn from_path(path: &impl AsRef<Path>) -> Option<ModResource> {
        let path = path.as_ref();

        if path.is_dir() {
            // We don't have any reason to use directories as resources at the moment, although
            //  this may change in the future.
            return None;
        }

        let extension = path.extension();

        if extension.is_none() && path.is_file() {
            log::warn!("Only folders may have no extension");
            return None;
        }

        let extension = extension.unwrap().to_str()?.to_lowercase();
        let relative_to_cleo = path.strip_prefix(find_cleo_dir_path()).ok()?;

        if relative_to_cleo.starts_with("Replace") || relative_to_cleo.starts_with("replace") {
            return Some(ModResource::FileReplacement(path.to_path_buf()));
        }

        let first_component = relative_to_cleo.iter().next().map(Path::new);

        let is_in_archive = first_component
            .and_then(std::path::Path::extension)
            .and_then(std::ffi::OsStr::to_str)
            .map(|ext| ext.to_lowercase() == "img")
            .unwrap_or(false);

        if is_in_archive {
            let archive_name = first_component?.file_name()?.to_str()?.to_lowercase();

            let instruction_file_name = format!("put files to go inside {archive_name} here");

            if path.file_name()?.to_str()? == instruction_file_name {
                // Ignore the instruction file.
                return None;
            }

            return Some(ModResource::StreamReplacement(
                archive_name,
                path.to_path_buf(),
            ));
        }

        match extension.as_str() {
            "csa" => Some(ModResource::StartupScript(path.to_path_buf())),
            "csi" => Some(ModResource::InvokedScript(path.to_path_buf())),
            "fxt" => Some(ModResource::LanguageFile(path.to_path_buf())),

            _ => {
                log::warn!("Unrecognised extension '{}'", extension);
                None
            }
        }
    }
}

#[cached]
fn find_cleo_dir_path() -> PathBuf {
    // Since iOS 13.5, we haven't been able to access the /var/mobile/Documents folder, so CLEO resources
    //  moved to the game's data folder. This is harder to find for users, but allows compatibility with
    //  basically any version of iOS.
    let path = get_documents_path("CLEO");

    if !path.exists() {
        log::warn!("CLEO folder was not found. It will be created.");

        // Create the folder.
        if let Err(err) = std::fs::create_dir(&path) {
            log::error!("Unable to create CLEO folder! Error: {}", err);
        }
    }

    path
}

fn create_replace_dir() {
    let path_lower = {
        let mut replace_path = find_cleo_dir_path();
        replace_path.push("replace");
        replace_path
    };

    let path_upper = {
        let mut replace_path = find_cleo_dir_path();
        replace_path.push("Replace");
        replace_path
    };

    if path_lower.exists() || path_upper.exists() {
        log::info!("'Replace' folder already exists.");
        return;
    }

    // We use the uppercase path if we're making the folder.
    if let Err(err) = std::fs::create_dir(&path_upper) {
        log::error!("Error creating dir {:?}: {}", path_upper, err);
    }
}

fn create_archive_dirs() {
    // The layout of file replacements can be difficult to explain, especially with the added
    //  complication of replacing files within IMG archives being a different process. In order
    //  to make things easier, we create all the folders for replacement IMG contents for the user.
    let game_dir = crate::game::loader::get_game_path().expect("Unable to get game path.");

    for entry in game_dir.read_dir().expect("Unable to read game directory.") {
        let entry = if let Err(err) = entry {
            log::error!("Error reading entry from game directory: {}", err);
            continue;
        } else {
            entry.unwrap()
        };

        let entry_path = entry.path();

        let extension = if let Some(ext) = entry_path.extension().and_then(|os| os.to_str()) {
            ext
        } else {
            continue;
        };

        if extension == "img" {
            let name_path = entry_path.strip_prefix(&game_dir).unwrap();
            let mut new_folder_path = find_cleo_dir_path();
            new_folder_path.push(name_path);

            let create_instruction_file = |mut path: PathBuf| {
                path.push(format!(
                    "put files to go inside {} here",
                    name_path.display()
                ));

                if path.exists() {
                    return;
                }

                if let Err(err) = std::fs::File::create(&path) {
                    log::error!("Failed to create instruction file {:?}: {}", path, err);
                }
            };

            if new_folder_path.exists() {
                if !new_folder_path.is_dir() {
                    log::error!("Top-level items with an 'img' extension must be directories!");
                    continue;
                }

                create_instruction_file(new_folder_path);

                continue;
            }

            if let Err(err) = std::fs::create_dir(&new_folder_path) {
                log::error!("Error creating dir {:?}: {}", new_folder_path, err);
            }

            create_instruction_file(new_folder_path);
        }
    }
}

/// Returns the path to the directory where we store game shaders.
pub fn shaders_path() -> PathBuf {
    get_documents_path("shaders")
}

pub fn get_log_path() -> PathBuf {
    let mut dir_path = find_cleo_dir_path();
    dir_path.push("cleo.log");

    dir_path
}

pub fn get_documents_path(resource_name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.set_file_name("Documents");
    path.push(resource_name);

    path
}

pub fn init() {
    log::info!("initialising resources...");

    let cleo_path = find_cleo_dir_path();

    log::info!("Creating 'Replace' folder...");
    create_replace_dir();

    log::info!("Creating archive folders...");
    create_archive_dirs();

    log::info!("Creating shader directory...");

    if let Err(err) = std::fs::create_dir_all(shaders_path()) {
        log::error!("failed to create shader dir: {:?}", err);
    }

    log::info!("Finding and loading resources...");
    let all_resources = ModResource::flatten_dir(&cleo_path).unwrap();

    let image_replacements = all_resources
        .iter()
        .filter_map(|resource| {
            if let ModResource::StreamReplacement(image_name, path) = resource {
                Some((image_name, path.clone()))
            } else {
                None
            }
        })
        .group_by(|&(image_name, _)| image_name);

    for (image_name, paths) in image_replacements.into_iter() {
        streaming::load_replacements(image_name, paths.map(|(_, path)| path));
    }

    for resource in &all_resources {
        log::info!("Attempting to load {}.", resource);

        let load_error = match resource {
            ModResource::StartupScript(path) => scripts::runtime::load_running_script(path).err(),
            ModResource::InvokedScript(path) => scripts::runtime::load_invoked_script(path).err(),
            ModResource::LanguageFile(path) => text::load_fxt(path).err(),
            ModResource::StreamReplacement(_, _) => None,
            ModResource::FileReplacement(path) => loader::load_replacement(&path).err(),
        };

        if let Some(err) = load_error {
            log::warn!("Failed to load resource: {}", err);
        }
    }

    log::info!("Finished loading resources.");
}
