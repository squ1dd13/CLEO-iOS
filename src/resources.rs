use crate::*;
use cached::proc_macro::cached;
use std::path::{Path, PathBuf};

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
            log::warn!("Only folders may have no extension.");
            return None;
        }

        let extension = extension.unwrap().to_str()?.to_lowercase();
        let relative_to_cleo = path.strip_prefix(find_cleo_dir_path()).ok()?;

        if relative_to_cleo.starts_with("Replace") || relative_to_cleo.starts_with("replace") {
            return Some(ModResource::FileReplacement(path.to_path_buf()));
        }

        let first_component = relative_to_cleo.iter().next().map(|first| Path::new(first));

        let is_in_archive = first_component
            .and_then(std::path::Path::extension)
            .and_then(std::ffi::OsStr::to_str)
            .map(|ext| ext.to_lowercase() == "img")
            .unwrap_or(false);

        if is_in_archive {
            let archive_name = first_component?.file_name()?.to_str()?.to_lowercase();

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
                log::warn!("Unrecognised extension '{}'.", extension);
                None
            }
        }
    }
}

#[cached]
fn find_cleo_dir_path() -> PathBuf {
    // Since iOS 13.5, we haven't been able to access the /var/mobile/Documents folder, so CLEO resources
    //  moved to the game's data folder. This is harder to find for users, but allows compatibility with
    //  basically any version of iOS. However, some users still use the /var/mobile/Documents/CS folder
    //  that was used exclusively in earlier C++ versions, so we still support that folder.
    let path = get_documents_path("CLEO");

    if !path.exists() {
        // Try the old path.
        let path = Path::new("/var/mobile/Documents/CS");

        if path.exists() {
            log::error!(
                "Using old pre-official path. Please consider switching to the newer path."
            );

            return path.to_path_buf();
        } else {
            log::error!("Unable to find the CLEO folder!");
        }
    }

    if !path.exists() {
        // Create the folder.
        if let Err(err) = std::fs::create_dir(&path) {
            log::error!("Unable to create CLEO folder! Error: {}", err);
        }
    }

    path
}

fn create_archive_dirs() {
    // The layout of file replacements can be difficult to explain, especially with the added
    //  complication of replacing files within IMG archives being a different process. In order
    //  to make things easier, we create all the folders for replacement IMG contents for the user.
    let game_dir = loader::get_game_path().expect("Unable to get game path.");

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

pub fn initialise() {
    let cleo_path = find_cleo_dir_path();

    log::info!("Creating archive folders...");
    create_archive_dirs();

    log::info!("Finding and loading resources...");
    let all_resources = ModResource::flatten_dir(&cleo_path).unwrap();

    for resource in all_resources.iter() {
        log::trace!("{:#?}", resource);

        let load_error = match resource {
            ModResource::StartupScript(path) => new_scripts::load_running_script(path).err(),
            ModResource::InvokedScript(path) => new_scripts::load_invoked_script(path).err(),
            ModResource::LanguageFile(path) => text::load_fxt(path).err(),
            ModResource::StreamReplacement(archive_name, path) => {
                stream::load_replacement(&archive_name, &path).err()
            }
            ModResource::FileReplacement(path) => loader::load_replacement(&path).err(),
        };

        if let Some(err) = load_error {
            log::warn!("Failed to load resource: {}", err);
        }
    }
}
