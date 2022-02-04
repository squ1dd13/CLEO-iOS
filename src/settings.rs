//! Manages the saving and loading of settings, as well as providing menu data and a thread-safe API.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use once_cell::sync::OnceCell;

use crate::{
    resources,
    ui::{self, RowData, RowDetail},
};

static SETTINGS: OnceCell<Settings> = OnceCell::new();

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct StoredSettings {
    sixty_fps: bool,
    show_fps: bool,
    save_cheats: bool,
    no_ceiling: bool,
    interrupt_loops: bool,
}

impl StoredSettings {
    fn into_settings(self) -> Settings {
        Settings {
            sixty_fps: Arc::new(AtomicBool::new(self.sixty_fps)),
            show_fps: Arc::new(AtomicBool::new(self.show_fps)),
            save_cheats: Arc::new(AtomicBool::new(self.save_cheats)),
            no_ceiling: Arc::new(AtomicBool::new(self.no_ceiling)),
            interrupt_loops: Arc::new(AtomicBool::new(self.interrupt_loops)),
            dirty: AtomicBool::new(true),
        }
    }

    fn from_settings(settings: &Settings) -> StoredSettings {
        StoredSettings {
            sixty_fps: settings.sixty_fps.load(Ordering::SeqCst),
            show_fps: settings.show_fps.load(Ordering::SeqCst),
            save_cheats: settings.save_cheats.load(Ordering::SeqCst),
            no_ceiling: settings.no_ceiling.load(Ordering::SeqCst),
            interrupt_loops: settings.interrupt_loops.load(Ordering::SeqCst),
        }
    }
}

impl Default for StoredSettings {
    fn default() -> Self {
        StoredSettings {
            sixty_fps: true,
            show_fps: false,
            save_cheats: false,
            no_ceiling: true,
            interrupt_loops: true,
        }
    }
}

pub struct Settings {
    pub sixty_fps: Arc<AtomicBool>,
    pub show_fps: Arc<AtomicBool>,
    pub save_cheats: Arc<AtomicBool>,
    pub no_ceiling: Arc<AtomicBool>,
    pub interrupt_loops: Arc<AtomicBool>,
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
            log::warn!("Settings structure already exists.");
        }
    }

    fn save(&self) -> eyre::Result<()> {
        // Only save if the settings have changed.
        if !self.dirty.load(Ordering::SeqCst) {
            log::info!("Settings have not changed since last save.");
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

impl ui::RowData for OptionInfo {
    fn title(&self) -> String {
        self.title.into()
    }

    fn detail(&self) -> ui::RowDetail {
        RowDetail::Info(self.desc.into())
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
            Some(ui::colours::GREEN)
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
            log::info!("Error saving settings in OptionInfo::drop: {}", err);
        }
    }
}

pub fn tab_data() -> ui::TabData {
    let settings = Settings::shared();

    let option_info = vec![
        OptionInfo::new(
            "60 FPS",
            "Increase the framerate limit from 30 FPS to 60. Default is On.",
            settings.sixty_fps.clone(),
        ),
        OptionInfo::new(
            "Save Cheat States",
            "Preserve the states of toggleable cheats between game loads/launches. Default is Off.",
            settings.save_cheats.clone(),
        ),
        // OptionInfo::new(
        //     "Remove Height Limit",
        //     "Remove the limit on how high you can fly. Default is On.",
        //     settings.no_ceiling.clone(),
        // ),
        OptionInfo::new(
            "Show FPS",
            "Display the current framerate at the top of the screen. Default is Off.",
            settings.show_fps.clone(),
        ),
        OptionInfo::new(
            "Interrupt Script Loops",
            "Reduce lag by detecting and interrupting long loops in scripts. Default is On.",
            settings.interrupt_loops.clone(),
        ),
    ];

    ui::TabData {
        name: "Options".to_string(),
        warning: None,
        row_data: option_info
            .into_iter()
            .map(|info| Box::new(info) as Box<dyn RowData>)
            .collect(),
    }
}

fn load_settings(menu_manager: u64) {
    log::info!("Loading CLEO settings");
    Settings::load_shared();

    // Save the current state of the settings so we create a settings file if it didn't exist.
    if let Err(err) = Settings::shared().save() {
        log::error!("Unable to save settings after load: {:?}", err);
    }

    log::info!("Loading game settings");
    crate::call_original!(crate::targets::load_settings, menu_manager);
}

pub fn init() {
    crate::targets::load_settings::install(load_settings);
}
