//! Provides the backend for the settings displayed in the menu, along with interfaces for fetching
//! option values and saving/loading.
// todo: Use JSON for storing settings instead of the crap we have going on here.
// fixme: Settings module is not compatible with the new menu because of its design.

use std::{
    io::{Read, Write},
    sync::Mutex,
};

use crate::{
    call_original,
    menu::{RowData, TabData},
};

pub struct OptionInfo {
    pub title: &'static str,
    pub description: &'static str,
    pub value: bool,
}

impl OptionInfo {
    const fn new(title: &'static str, description: &'static str, value: bool) -> OptionInfo {
        OptionInfo {
            title,
            description,
            value,
        }
    }
}

#[repr(usize)]
pub enum Key {
    SixtyFPS,
    ShowFPS,
}

pub struct Settings(pub Vec<OptionInfo>);

impl Settings {
    pub fn get(&mut self, key: Key) -> &mut OptionInfo {
        &mut self.0[key as usize]
    }

    fn save(&self) {
        let path = crate::resources::get_documents_path("cleo.settings");

        if let Ok(mut opened) = std::fs::File::create(path) {
            let bytes: Vec<u8> = self.0.iter().map(|opt| opt.value as u8).collect();

            if let Err(err) = opened.write(&bytes[..]) {
                log::error!("Unable to write settings file. Error: {:?}", err);
            } else {
                log::info!("Wrote settings file.");
            }
        }
    }

    pub fn load(&mut self) {
        let path = crate::resources::get_documents_path("cleo.settings");

        if let Ok(mut opened) = std::fs::File::open(path) {
            let mut bytes = vec![0u8; self.0.len()];

            match opened.read(&mut bytes[..]) {
                Err(err) => {
                    log::error!("Error while reading bytes from settings file: {:?}", err);
                    return;
                }

                Ok(num) => {
                    log::info!("Loaded {} bytes from cleo.settings file.", num);

                    if num != bytes.len() {
                        log::warn!("Did not fill settings buffer entirely! Default option values will be used.");
                        return;
                    }
                }
            };

            // Apply the settings values to the options we have.
            for (i, value) in bytes.iter().enumerate() {
                self.0[i].value = *value != 0;
            }
        }
    }
}

pub fn with_shared<T>(with: &mut impl FnMut(&mut Settings) -> T) -> T {
    let mut locked = SETTINGS.lock();
    with(locked.as_mut().unwrap())
}

pub fn save() {
    std::thread::spawn(|| {
        with_shared(&mut |options| {
            options.save();
        });
    });
}

fn load_settings(menu_manager: u64) {
    log::info!("Loading CLEO settings.");
    with_shared(&mut Settings::load);
    log::info!("Finished loading CLEO settings. Game will now load its own settings.");

    call_original!(crate::targets::load_settings, menu_manager);
}

pub fn hook() {
    crate::targets::load_settings::install(load_settings);
}

lazy_static::lazy_static! {
    static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings(vec![
        OptionInfo::new(
            "60 FPS",
            "Increase the framerate limit from 30 to 60 FPS.",
            true,
        ),
        OptionInfo::new(
            "Show FPS",
            "Enable the game's built-in FPS visualisation.",
            false,
        ),
    ]));
}
