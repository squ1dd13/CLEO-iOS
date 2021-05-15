use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;
use log::{error, info, warn};
use std::sync::Mutex;

pub struct LanguageFile;

impl LanguageFile {
    pub fn new(path: &Path) -> Result<Box<dyn Component>> {
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

        Ok(Box::new(LanguageFile {}))
    }
}

impl Component for LanguageFile {}

pub fn get_cleo_dir_path() -> PathBuf {
    // As of iOS 13.5, we need extra entitlements to access /var/mobile/Documents/*, so
    //  we need to use the app's own data directory instead. env::temp_dir() returns the
    //  'tmp' subdirectory of that data directory, and then we can just replace the 'tmp'
    //  with 'Documents/CLEO' to get our own directory.
    let mut path = std::env::temp_dir();
    path.set_file_name("Documents");
    path.push("CLEO");
    path
}

pub fn get_log_path() -> PathBuf {
    let mut path = get_cleo_dir_path();
    path.push("cleo.log");
    path
}

pub fn setup_cleo_fs() -> Result<()> {
    let cleo_path = get_cleo_dir_path();

    if !cleo_path.exists() {
        std::fs::create_dir(&cleo_path)?;
    }

    Ok(())
}

pub trait Component {
    /// Unload the component.
    fn unload(&mut self) {}

    /// Reset the component when the game is reloaded.
    fn reset(&mut self) {}
}

pub type ExtensionHandler = fn(&Path) -> std::io::Result<Box<dyn Component>>;

lazy_static! {
    static ref EXTENSION_HANDLERS: Mutex<HashMap<String, ExtensionHandler>> =
        Mutex::new(HashMap::new());
}

pub struct ComponentSystem {
    components: Vec<Box<dyn Component>>,
}

impl ComponentSystem {
    pub fn new(dir_path: impl AsRef<Path>) -> std::io::Result<ComponentSystem> {
        info!("Loading component system from {:?}", dir_path.as_ref());

        let mut component_system = ComponentSystem { components: vec![] };

        if let Err(err) = component_system.load_dir(dir_path) {
            error!("Error loading component system directory: {}", err);
        }

        Ok(component_system)
    }

    fn load_dir(&mut self, dir_path: impl AsRef<Path>) -> std::io::Result<()> {
        let directory = std::fs::read_dir(dir_path)?;

        for item in directory {
            if let Ok(entry) = item {
                if let Ok(file_type) = entry.file_type() {
                    let path = entry.path();

                    if file_type.is_dir() {
                        if let Err(err) = self.load_dir(entry.path()) {
                            error!("Unable to load dir: {}", err);
                        }
                    } else {
                        if let Err(err) = self.load_path(&path) {
                            error!("Error loading {:?}: {}", path, err);
                        }
                    }
                } else {
                    error!("Unable to obtain file type for entry {:?}", entry);
                }
            } else {
                error!("Bad directory entry: {:?}", item);
            }
        }

        Ok(())
    }

    fn load_path(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let extension = path
            .as_ref()
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .ok_or(Error::new(ErrorKind::InvalidInput, "Extension required"))?
            .to_lowercase()
            .to_string();

        let handlers = EXTENSION_HANDLERS.lock();
        let handler = handlers.as_ref().ok().and_then(|map| map.get(&extension));

        if let Some(handler) = handler {
            let result = handler(path.as_ref());

            match result {
                Ok(component) => {
                    self.components.push(component);
                }

                Err(err) => error!(
                    "Error running component handler for '{:?}': {}",
                    path.as_ref().to_path_buf(),
                    err
                ),
            }
        } else {
            warn!("No handler set for extension '{}'.", extension);
        }

        Ok(())
    }

    pub fn reset_all(&mut self) {
        self.components.iter_mut().for_each(|boxed| boxed.reset());
    }

    pub fn unload_all(&mut self) {
        self.components.iter_mut().for_each(|boxed| boxed.unload());
    }

    /// Register a function to handle the construction of components from files with a certain extension.
    pub fn register_extension(extension: impl ToString, function: ExtensionHandler) {
        let mut handler_map = EXTENSION_HANDLERS.lock().unwrap();

        let extension: String = extension.to_string().clone();

        if handler_map.contains_key(&extension) {
            warn!(
                "Overwriting previous handler for extension '{}'.",
                extension
            );
        }

        handler_map.insert(extension, function);
    }
}

impl Drop for ComponentSystem {
    fn drop(&mut self) {
        self.unload_all();
    }
}
