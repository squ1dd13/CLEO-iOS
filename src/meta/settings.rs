use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use eyre::Result;

use super::{
    gui::colours::Colour,
    language::{self, Language, Message, MessageKey},
    menu::{self, RowData, RowDetail},
};
use serde::{Deserialize, Serialize};

/// Trait for option values.
trait Setting {
    /// The title of the setting. This is shown in the menu.
    fn title(&self) -> Message;

    /// The description of the setting. This is shown in the menu.
    fn description(&self) -> Message;

    /// Sets this setting's value in `options`.
    fn apply(&self, options: &mut Options);

    /// Changes to the next value in the cycle. This method is called when the user taps the option
    /// cell to change the value.
    fn cycle_value(&mut self);

    /// Returns the colour that should be used to represent this setting's value in the menu. If
    /// there is no meaningful use for a colour given the current value, this should return `None`.
    fn status_colour(&self) -> Option<Colour>;

    /// Returns a string describing the value.
    fn to_str(&self) -> Message;
}

impl<Opt> RowData for Opt
where
    Opt: Setting,
{
    fn title(&self) -> Message {
        Setting::title(self)
    }

    fn detail(&self) -> RowDetail {
        RowDetail::Info(self.description())
    }

    fn value(&self) -> Message {
        self.to_str()
    }

    fn tint(&self) -> Option<(u8, u8, u8)> {
        self.status_colour().map(|colour| colour.rgb())
    }

    fn handle_tap(&mut self) -> bool {
        // Switch to the next value.
        self.cycle_value();

        {
            let mut options = Options::global_mut();
            self.apply(options.as_mut().unwrap());

            // Mutex guard is dropped here.
        }

        Options::save();

        // Reload the cells every time. This covers any cases where a setting modifies other
        // settings, and there isn't much overhead because there are so few settings cells.
        true
    }
}

/// FPS lock modes.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum FpsLock {
    /// 30 FPS. This is the default mode.
    Thirty,

    /// 60 FPS.
    Sixty,
}

impl FpsLock {
    /// Returns the FPS lock as a number.
    pub fn fps(self) -> u32 {
        match self {
            FpsLock::Thirty => 30,
            FpsLock::Sixty => 60,
        }
    }
}

impl Default for FpsLock {
    fn default() -> Self {
        FpsLock::Sixty
    }
}

impl Setting for FpsLock {
    fn title(&self) -> Message {
        MessageKey::FpsLockOptTitle.to_message()
    }

    fn description(&self) -> Message {
        MessageKey::FpsLockOptDesc.to_message()
    }

    fn apply(&self, options: &mut Options) {
        options.fps_lock = *self;
    }

    fn cycle_value(&mut self) {
        *self = match self {
            FpsLock::Thirty => FpsLock::Sixty,
            FpsLock::Sixty => FpsLock::Thirty,
        };
    }

    fn status_colour(&self) -> Option<Colour> {
        None
    }

    fn to_str(&self) -> Message {
        match self {
            FpsLock::Thirty => MessageKey::FpsLockOpt30,
            FpsLock::Sixty => MessageKey::FpsLockOpt60,
        }
        .to_message()
    }
}

/// The visibility of the FPS counter.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum FpsVisibility {
    /// Not visible. This is the default.
    Hidden,

    /// Visible.
    Visible,
}

impl Default for FpsVisibility {
    fn default() -> Self {
        FpsVisibility::Hidden
    }
}

impl Setting for FpsVisibility {
    fn title(&self) -> Message {
        MessageKey::FpsCounterOptTitle.to_message()
    }

    fn description(&self) -> Message {
        MessageKey::FpsCounterOptDesc.to_message()
    }

    fn apply(&self, options: &mut Options) {
        options.fps_visibility = *self;
    }

    fn cycle_value(&mut self) {
        *self = match self {
            FpsVisibility::Hidden => FpsVisibility::Visible,
            FpsVisibility::Visible => FpsVisibility::Hidden,
        };
    }

    fn status_colour(&self) -> Option<Colour> {
        None
    }

    fn to_str(&self) -> Message {
        match self {
            FpsVisibility::Hidden => MessageKey::FpsCounterOptHidden,
            FpsVisibility::Visible => MessageKey::FpsCounterOptEnabled,
        }
        .to_message()
    }
}

/// The behaviour of the cheat system across game restarts.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum CheatTransience {
    /// Cheats will not be saved across game restarts. This is the default.
    Transient,

    /// Cheats will be saved across game restarts.
    Persistent,
}

impl Default for CheatTransience {
    fn default() -> Self {
        CheatTransience::Persistent
    }
}

impl Setting for CheatTransience {
    fn title(&self) -> Message {
        MessageKey::CheatTransienceOptTitle.to_message()
    }

    fn description(&self) -> Message {
        MessageKey::CheatTransienceOptDesc.to_message()
    }

    fn apply(&self, options: &mut Options) {
        options.cheat_transience = *self;
    }

    fn cycle_value(&mut self) {
        *self = match self {
            CheatTransience::Transient => CheatTransience::Persistent,
            CheatTransience::Persistent => CheatTransience::Transient,
        };
    }

    fn status_colour(&self) -> Option<Colour> {
        None
    }

    fn to_str(&self) -> Message {
        match self {
            CheatTransience::Transient => MessageKey::CheatTransienceOptTransient,
            CheatTransience::Persistent => MessageKey::CheatTransienceOptPersistent,
        }
        .to_message()
    }
}

/// Modes for handling long script loops.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum BreakMode {
    /// Loops will be allowed to run, even if it causes the game to lag.
    DontBreak,

    /// Loops will be broken and continued later. This is the default.
    Break,
}

impl Default for BreakMode {
    fn default() -> Self {
        BreakMode::Break
    }
}

impl Setting for BreakMode {
    fn title(&self) -> Message {
        MessageKey::ScriptModeOptTitle.to_message()
    }

    fn description(&self) -> Message {
        MessageKey::ScriptModeOptDesc.to_message()
    }

    fn apply(&self, options: &mut Options) {
        options.loop_break = *self;
    }

    fn cycle_value(&mut self) {
        *self = match self {
            BreakMode::DontBreak => BreakMode::Break,
            BreakMode::Break => BreakMode::DontBreak,
        };
    }

    fn status_colour(&self) -> Option<Colour> {
        None
    }

    fn to_str(&self) -> Message {
        match self {
            BreakMode::DontBreak => MessageKey::ScriptModeOptDontBreak,
            BreakMode::Break => MessageKey::ScriptModeOptBreak,
        }
        .to_message()
    }
}

/// Which set of updates the users receives.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum ReleaseChannel {
    /// No release channel. The user won't receive any updates.
    None,

    /// Stable release channel. Most users should be on this.
    Stable,

    /// Alpha release channel. Alpha releases are less stable but come with new features. Stable
    /// releases are also included here, but the user will always be prompted to get the latest
    /// alpha if it's newer than the latest stable release.
    Alpha,
}

impl Default for ReleaseChannel {
    fn default() -> Self {
        ReleaseChannel::Stable
    }
}

impl Setting for ReleaseChannel {
    fn title(&self) -> Message {
        MessageKey::UpdateReleaseChannelOptTitle.to_message()
    }

    fn description(&self) -> Message {
        MessageKey::UpdateReleaseChannelOptDesc.to_message()
    }

    fn apply(&self, options: &mut Options) {
        options.release_channel = *self;
    }

    fn cycle_value(&mut self) {
        *self = match self {
            ReleaseChannel::None => ReleaseChannel::Stable,
            ReleaseChannel::Stable => ReleaseChannel::Alpha,
            ReleaseChannel::Alpha => ReleaseChannel::None,
        }
    }

    fn status_colour(&self) -> Option<Colour> {
        match self {
            ReleaseChannel::None => Some(Colour::Red),
            ReleaseChannel::Stable => Some(Colour::Green),
            ReleaseChannel::Alpha => Some(Colour::Orange),
        }
    }

    fn to_str(&self) -> Message {
        match self {
            ReleaseChannel::None => MessageKey::UpdateReleaseChannelOptDisabled,
            ReleaseChannel::Stable => MessageKey::UpdateReleaseChannelOptStable,
            ReleaseChannel::Alpha => MessageKey::UpdateReleaseChannelOptAlpha,
        }
        .to_message()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum LanguageMode {
    /// Matches the CLEO language to the device/game settings.
    Automatic,

    /// Forces CLEO to use a specific language.
    Explicit(Language),
}

impl LanguageMode {
    /// Returns the language that this mode represents, or `None` if the language should be chosen
    /// automatically.
    pub fn language(self) -> Option<Language> {
        match self {
            LanguageMode::Automatic => None,
            LanguageMode::Explicit(language) => Some(language),
        }
    }
}

impl Default for LanguageMode {
    fn default() -> Self {
        LanguageMode::Automatic
    }
}

impl Setting for LanguageMode {
    fn title(&self) -> Message {
        MessageKey::LanguageOptTitle.to_message()
    }

    fn description(&self) -> Message {
        MessageKey::LanguageOptDesc.to_message()
    }

    fn apply(&self, options: &mut Options) {
        options.language_mode = *self;

        language::set(match self {
            LanguageMode::Automatic => None,
            LanguageMode::Explicit(language) => Some(*language),
        });
    }

    fn cycle_value(&mut self) {
        *self = match self {
            // If we're currently set to automatic mode, choose English first, because it has the
            // largest number of speakers.
            LanguageMode::Automatic => LanguageMode::Explicit(Language::English),

            LanguageMode::Explicit(current) => {
                // If we're on an explicit choice which isn't the last language, move to the next
                // language.
                if let Some(next_most_spoken) = current.next_most_spoken() {
                    LanguageMode::Explicit(next_most_spoken)
                } else {
                    // If we've reached the end of the supported languages, go back to automatic
                    // mode.
                    LanguageMode::Automatic
                }
            }
        }
    }

    fn status_colour(&self) -> Option<Colour> {
        None
    }

    fn to_str(&self) -> Message {
        match self {
            LanguageMode::Automatic => MessageKey::LanguageAutoName,
            LanguageMode::Explicit(_) => MessageKey::LanguageName,
        }
        .to_message()
    }
}

/// The user's CLEO settings.
#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug)]
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

    /// Determines which language CLEO will use.
    #[serde(default)]
    pub language_mode: LanguageMode,
}

impl Options {
    /// Attempts to parse the contents of `reader` to get an `Options` value.
    fn parse_json(reader: impl Read) -> Result<Options> {
        // Coerce with `?`.
        Ok(serde_json::from_reader(reader)?)
    }

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

    /// Looks for an old-style settings file and loads it. If an old file does exist, it will be
    /// deleted after loading.
    fn load_from_old_file() -> Result<Option<Options>> {
        let old_path = crate::meta::resources::get_documents_path("cleo_settings.json");

        if !old_path.exists() {
            return Ok(None);
        }

        let json: HashMap<String, bool> = serde_json::from_reader(File::open(&old_path)?)?;

        // We've got everything we need from the old file, so delete it.
        if let Err(err) = std::fs::remove_file(old_path) {
            log::error!("Error removing old file: {err:?}.");
        }

        /// Converts the Boolean value found at `key` in `json` to an `Opt` value, yielding
        /// `true_val` if the value is `true`, `false_val` if not, and `Opt::default()` if the key
        /// wasn't found in `json`.
        fn convert_bool<Opt: Default>(
            json: &HashMap<String, bool>,
            key: &'static str,
            true_val: Opt,
            false_val: Opt,
        ) -> Opt {
            json.get(key)
                .map(|v| if *v { true_val } else { false_val })
                .unwrap_or_default()
        }

        Ok(Some(Options {
            fps_lock: convert_bool(&json, "sixty_fps", FpsLock::Sixty, FpsLock::Thirty),

            fps_visibility: convert_bool(
                &json,
                "show_fps",
                FpsVisibility::Visible,
                FpsVisibility::Hidden,
            ),

            cheat_transience: convert_bool(
                &json,
                "save_cheats",
                CheatTransience::Persistent,
                CheatTransience::Transient,
            ),

            loop_break: convert_bool(
                &json,
                "interrupt_loops",
                BreakMode::Break,
                BreakMode::DontBreak,
            ),

            // These options didn't exist.
            release_channel: ReleaseChannel::default(),
            language_mode: LanguageMode::default(),
        }))
    }

    /// Returns the path of the file that options are saved to.
    fn path() -> PathBuf {
        crate::meta::resources::get_documents_path("settings.cleo.json")
    }

    /// Looks for a settings file and loads it.
    fn load_from_file() -> Result<Option<Options>> {
        let path = Options::path();

        if !path.exists() {
            // This isn't an error, but we didn't find any settings.
            return Ok(None);
        }

        Ok(Some(Options::parse_json(File::open(path)?)?))
    }

    /// Loads whatever settings files the user has.
    fn load_files() -> Result<Option<Options>> {
        // Try to load an old file first so that we can preserve the user's settings.
        let old_settings = Options::load_from_old_file();

        if let Ok(Some(_)) = old_settings {
            log::info!("Loaded from old settings file.");
            return old_settings;
        } else {
            log::info!("No old settings found.");
        }

        Options::load_from_file()
    }

    /// Either loads the settings from disk or generates default values for them.
    fn load() -> Options {
        match Options::load_files() {
            Ok(Some(options)) => return options,

            Ok(None) => log::info!("No settings files found. Defaults will be used."),

            Err(err) => {
                log::error!("Error loading settings files: {err:?}. Defaults will be used.")
            }
        };

        Options::default()
    }

    /// Saves the settings to a file, returning any errors encountered.
    fn try_save() -> Result<()> {
        std::fs::write(
            Options::path(),
            serde_json::to_string_pretty(&Options::get())?,
        )?;

        Ok(())
    }

    /// Saves the settings to a file. Errors will be logged.
    fn save() {
        if let Err(err) = Options::try_save() {
            log::error!("Error saving options to file: {err:?}.");
        } else {
            log::info!("Settings saved.");
        }
    }

    /// Loads the settings and stores them globally.
    fn init_global() {
        *Options::global_mut() = Some(Options::load());
    }
}

/// Returns the tab data for the options menu.
pub fn tab_data() -> menu::TabData {
    let options = Options::get();

    let options = vec![
        Box::new(options.fps_lock) as Box<dyn RowData>,
        Box::new(options.fps_visibility) as Box<dyn RowData>,
        Box::new(options.cheat_transience) as Box<dyn RowData>,
        Box::new(options.loop_break) as Box<dyn RowData>,
        Box::new(options.language_mode) as Box<dyn RowData>,
        Box::new(options.release_channel) as Box<dyn RowData>,
    ];

    menu::TabData {
        name: MessageKey::MenuOptionsTabTitle.to_message(),
        warning: None,
        row_data: options,
    }
}

pub fn init() {
    Options::init_global();

    log::info!("Options: {:#?}", Options::get());
}
