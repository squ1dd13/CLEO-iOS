//! Creates a flattened version of the CLEO directory and delegates to other modules for
//! handling specific types of resources.

use cached::proc_macro::cached;
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

        x.img       A folder containing replacements for the files inside the x.img archive in the game's folder.
          example   A file that replaces the file named 'example' that is normally loaded from inside x.img.

        Replace     A folder containing replacements for game files.
          example   A file that replaces the file named 'example' in the game's folder.

    The job of this module is to flatten this structure to make it easier for other modules to find the files they want.
*/

#[derive(Debug)]
pub enum ModRes {
    // CSA script.
    RunningScript(PathBuf),

    // CSI script.
    LazyScript(PathBuf),

    // JS script.
    JsScript(PathBuf),

    // FXT language file.
    KeyValFile(PathBuf),

    // Anything inside a top-level folder with the extension "img".
    // First value is the image name.
    ArchSwap(String, PathBuf),

    // A file from the "Replace" folder.
    Swap(PathBuf),
}

impl Display for ModRes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModRes::RunningScript(path) => {
                write!(f, "running script {:?}", path.file_name().unwrap())
            }
            ModRes::LazyScript(path) => {
                write!(f, "lazy script {:?}", path.file_name().unwrap())
            }
            ModRes::JsScript(path) => {
                write!(f, "JavaScript script {:?}", path.file_name().unwrap())
            }
            ModRes::KeyValFile(path) => {
                write!(f, "language file {:?}", path.file_name().unwrap())
            }
            ModRes::ArchSwap(img_name, path) => write!(
                f,
                "replacement file {:?} for archive \"{}\"",
                path.file_name().unwrap(),
                img_name
            ),
            ModRes::Swap(path) => write!(
                f,
                "general file replacement {:?}",
                path.file_name().unwrap()
            ),
        }
    }
}

impl ModRes {
    fn from_path(path: &Path) -> Option<ModRes> {
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

        let path = path.to_path_buf();

        let extension = extension.unwrap().to_str()?.to_lowercase();
        let relative_to_cleo = path.strip_prefix(find_cleo_dir_path()).ok()?;

        if relative_to_cleo.starts_with("Replace") || relative_to_cleo.starts_with("replace") {
            return Some(ModRes::Swap(path));
        }

        let first_component = relative_to_cleo.iter().next().map(Path::new);

        let is_in_archive = first_component
            .and_then(std::path::Path::extension)
            .and_then(std::ffi::OsStr::to_str)
            .map(|ext| ext.to_lowercase() == "img")
            .unwrap_or(false);

        if is_in_archive {
            let archive_name = first_component?.file_name()?.to_str()?.to_lowercase();

            let instruction_file_name = format!("put files to go inside {} here", archive_name);

            if path.file_name()?.to_str()? == instruction_file_name {
                // Ignore the instruction file.
                return None;
            }

            return Some(ModRes::ArchSwap(archive_name, path));
        }

        match extension.as_str() {
            "csa" => Some(ModRes::RunningScript(path)),
            "csi" => Some(ModRes::LazyScript(path)),
            "js" => Some(ModRes::JsScript(path)),
            "fxt" => Some(ModRes::KeyValFile(path)),

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
    let game_dir = super::get_game_path().expect("Unable to get game path.");

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

/// Returns an iterator over the resources in the CLEO directory. Modules should filter through the
/// resources in the iterator to find the ones they need to use.
pub fn res_iter() -> impl Iterator<Item = ModRes> {
    walkdir::WalkDir::new(find_cleo_dir_path())
        .into_iter()
        .filter_map(|entry| ModRes::from_path(entry.ok()?.path()))
}

pub fn init() {
    create_replace_dir();
    create_archive_dirs();
}
