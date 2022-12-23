//! Manages the saving and loading of settings, as well as providing menu data and a thread-safe API.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard,
};

use eyre::Result;
use once_cell::sync::OnceCell;

use crate::{
    menu::{self, RowData, RowDetail},
    resources,
};

static SETTINGS: OnceCell<Settings> = OnceCell::new();

/// FPS lock modes.
#[derive(Clone, Copy)]
pub enum FpsLock {
    /// 30 FPS. This is the default mode.
    Thirty,

    /// 60 FPS.
    Sixty,
}

/// The visibility of the FPS counter.
#[derive(Clone, Copy)]
pub enum FpsVisibility {
    /// Not visible. This is the default.
    Hidden,

    /// Visible.
    Visible,
}

/// The behaviour of the cheat system across game restarts.
#[derive(Clone, Copy)]
pub enum CheatTransience {
    /// Cheats will not be saved across game restarts. This is the default.
    Transient,

    /// Cheats will be saved across game restarts.
    Persistent,
}

/// Modes for handling long script loops.
#[derive(Clone, Copy)]
pub enum BreakMode {
    /// Loops will be allowed to run, even if it causes the game to lag.
    DontBreak,

    /// Loops will be broken and continued later. This is the default.
    Break,
}

/// Which set of updates the users receives.
#[derive(Clone, Copy)]
pub enum ReleaseChannel {
    /// Stable release channel. Most users should be on this.
    Stable,

    /// Alpha release channel. Alpha releases are less stable but come with new features. Stable
    /// releases are also included here, but the user will always be prompted to get the latest
    /// alpha if it's newer than the latest stable release.
    Alpha,
}

/// The user's CLEO settings.
#[derive(Clone, Copy)]
pub struct Options {
    /// The FPS value that the game locks to.
    pub fps_lock: FpsLock,

    /// Whether or not the current FPS will be shown on the screen.
    pub fps_visibility: FpsVisibility,

    /// How cheats persist across game restarts.
    pub cheat_transience: CheatTransience,

    /// How long loops are handled in order to reduce lag.
    pub loop_break: BreakMode,

    /// Controls when the user is prompted to update their game.
    pub release_channel: ReleaseChannel,
}

// todo: Manually write JSON and add comments. Use ".jsonc".

impl Options {
    /// Returns a mutex guard around the global options value.
    fn global_mut() -> MutexGuard<'static, Option<Options>> {
        lazy_static::lazy_static! {
            static ref OPTIONS: Mutex<Option<Options>> = Mutex::new(None);
        }

        OPTIONS.lock().expect("Failed to lock options")
    }

    /// Returns the user's current CLEO settings.
    pub fn get() -> Options {
        Options::global_mut().expect("Settings haven't been loaded yet")
    }

    /// Loads the global settings from disk.
    fn load_global() -> Result<()> {
        // todo: If we find some old settings JSON, replace it with the modern equivalent JSON.

        todo!()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct StoredSettings {
    sixty_fps: bool,
    show_fps: bool,
    save_cheats: bool,
    no_ceiling: bool,
    interrupt_loops: bool,
    alpha_updates: bool,
}

impl StoredSettings {
    fn into_settings(self) -> Settings {
        Settings {
            sixty_fps: Arc::new(AtomicBool::new(self.sixty_fps)),
            show_fps: Arc::new(AtomicBool::new(self.show_fps)),
            save_cheats: Arc::new(AtomicBool::new(self.save_cheats)),
            no_ceiling: Arc::new(AtomicBool::new(self.no_ceiling)),
            interrupt_loops: Arc::new(AtomicBool::new(self.interrupt_loops)),
            alpha_updates: Arc::new(AtomicBool::new(self.alpha_updates)),
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
            alpha_updates: settings.alpha_updates.load(Ordering::SeqCst),
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
            alpha_updates: false,
        }
    }
}

#[derive(Debug)]
pub struct Settings {
    pub sixty_fps: Arc<AtomicBool>,
    pub show_fps: Arc<AtomicBool>,
    pub save_cheats: Arc<AtomicBool>,
    pub no_ceiling: Arc<AtomicBool>,
    pub interrupt_loops: Arc<AtomicBool>,
    pub alpha_updates: Arc<AtomicBool>,
    dirty: AtomicBool,
}

impl Settings {
    fn load_path(path: std::path::PathBuf) -> Result<Settings> {
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

    fn save(&self) -> Result<()> {
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

impl crate::menu::RowData for OptionInfo {
    fn title(&self) -> String {
        self.title.into()
    }

    fn detail(&self) -> crate::menu::RowDetail {
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
            log::info!("Error saving settings in OptionInfo::drop: {}", err);
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
            "Save Cheat States",
            "Preserve the states of toggleable cheats between game loads/launches. Default is Off.",
            settings.save_cheats.clone(),
        ),
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
        OptionInfo::new(
            "Receive Alpha Updates",
            "Receive an update prompt when a new alpha version of CLEO is available. Default is Off.",
            settings.alpha_updates.clone(),
        ),
    ];

    menu::TabData {
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
    Settings::load_shared();

    log::info!("Settings: {:?}", Settings::shared());

    log::info!("installing settings hook...");
    crate::targets::load_settings::install(load_settings);
}
