//! Manages the saving and loading of settings, as well as providing menu data and a thread-safe API.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use once_cell::sync::OnceCell;

use crate::{
    menu::{self, RowData, RowDetail},
    resources,
};

static SETTINGS: OnceCell<Settings> = OnceCell::new();

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct StoredSettings {
    sixty_fps: bool,
    show_fps: bool,
}

impl StoredSettings {
    fn into_settings(self) -> Settings {
        Settings {
            sixty_fps: Arc::new(AtomicBool::new(self.sixty_fps)),
            show_fps: Arc::new(AtomicBool::new(self.show_fps)),
            dirty: AtomicBool::new(true),
        }
    }

    fn from_settings(settings: &Settings) -> StoredSettings {
        StoredSettings {
            sixty_fps: settings.sixty_fps.load(Ordering::SeqCst),
            show_fps: settings.show_fps.load(Ordering::SeqCst),
        }
    }
}

impl Default for StoredSettings {
    fn default() -> Self {
        StoredSettings {
            sixty_fps: true,
            show_fps: false,
        }
    }
}

pub struct Settings {
    pub sixty_fps: Arc<AtomicBool>,
    pub show_fps: Arc<AtomicBool>,
    dirty: AtomicBool,
}

impl Settings {
    fn load_path(path: std::path::PathBuf) -> eyre::Result<Settings> {
        let stored: StoredSettings = serde_json::from_reader(std::fs::File::open(path)?)?;
        Ok(stored.into_settings())
    }

    fn load_shared() {
        let path = resources::get_documents_path("cleo_settings.json");

        let settings = Self::load_path(path).unwrap_or_else(|err| {
            log::error!("Failed to load settings from JSON: {:?}", err);
            log::info!("Using default values instead.");
            StoredSettings::default().into_settings()
        });

        if SETTINGS.set(settings).is_err() {
            log::warn!("Settings structure already exists");
        }
    }

    fn save(&self) -> eyre::Result<()> {
        // Only save if the settings have changed.
        if !self.dirty.load(Ordering::SeqCst) {
            log::info!("settings have not changed since last save");
            return Ok(());
        }

        self.dirty.store(false, Ordering::SeqCst);

        // fixme: Settings::save should be non-blocking.
        Ok(serde_json::to_writer_pretty(
            std::fs::File::create(resources::get_documents_path("cleo_settings.json"))?,
            &StoredSettings::from_settings(self),
        )?)
    }

    fn set_dirty(&self) {
        self.dirty.store(true, Ordering::SeqCst);
    }

    pub fn shared() -> &'static Settings {
        SETTINGS.get().unwrap()
    }
}

#[derive(Debug)]
struct OptionInfo {
    title: &'static str,
    desc: &'static str,
    value: Arc<AtomicBool>,
}

impl OptionInfo {
    fn new(title: &'static str, desc: &'static str, value: Arc<AtomicBool>) -> OptionInfo {
        OptionInfo { title, desc, value }
    }
}

impl crate::menu::RowData for OptionInfo {
    fn title(&self) -> &str {
        self.title
    }

    fn detail(&self) -> crate::menu::RowDetail<'_> {
        RowDetail::Info(self.desc)
    }

    fn value(&self) -> &str {
        if self.value.load(Ordering::SeqCst) {
            "On"
        } else {
            "Off"
        }
    }

    fn tint(&self) -> Option<(u8, u8, u8)> {
        if self.value.load(Ordering::SeqCst) {
            Some(crate::gui::colours::GREEN)
        } else {
            None
        }
    }

    fn handle_tap(&mut self) -> bool {
        self.value
            .store(!self.value.load(Ordering::SeqCst), Ordering::SeqCst);

        Settings::shared().set_dirty();

        true
    }
}

impl Drop for OptionInfo {
    fn drop(&mut self) {
        if let Err(err) = Settings::shared().save() {
            log::info!("error saving settings in OptionInfo::drop: {}", err);
        }
    }
}

pub fn tab_data() -> menu::TabData {
    let settings = Settings::shared();

    let option_info = vec![
        OptionInfo::new(
            "60 FPS",
            "Increase the framerate limit from 30 FPS to 60. Default is On.",
            settings.sixty_fps.clone(),
        ),
        OptionInfo::new(
            "Show FPS",
            "Display the current framerate at the top of the screen. Default is Off.",
            settings.show_fps.clone(),
        ),
    ];

    menu::TabData {
        name: "Settings".to_string(),
        warning: None,
        row_data: option_info
            .into_iter()
            .map(|info| Box::new(info) as Box<dyn RowData>)
            .collect(),
    }
}

fn load_settings(menu_manager: u64) {
    log::info!("loading CLEO settings");
    Settings::load_shared();

    // Save the current state of the settings so we create a settings file if it didn't exist.
    if let Err(err) = Settings::shared().save() {
        log::error!("unable to save settings after load: {:?}", err);
    }

    log::info!("loading game settings");
    crate::call_original!(crate::targets::load_settings, menu_manager);
}

pub fn hook() {
    crate::targets::load_settings::install(load_settings);
}
