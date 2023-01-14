use eyre::{eyre, Result};
use fluent::{concurrent::FluentBundle, FluentArgs, FluentResource};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, sync::Mutex};
use strum::{EnumIter, EnumString, EnumVariantNames, IntoEnumIterator, IntoStaticStr};

pub use fluent::fluent_args as msg_args;

lazy_static::lazy_static! {
    static ref LOADER: Mutex<Loader> = Mutex::new(Loader::new_empty());
}

/// Structure for managing language bundles.
struct Loader {
    /// The language set by the user. If this is `None`, `auto_language` will be used.
    language_override: Option<Language>,

    /// The language to use if the user doesn't set one explicitly.
    auto_language: Language,

    /// The bundles that have been loaded.
    bundles: HashMap<Language, LanguageBundle>,
}

impl Loader {
    /// Locks the shared loader and returns the guard.
    fn lock() -> std::sync::MutexGuard<'static, Loader> {
        LOADER.lock().unwrap()
    }

    /// Creates an empty language loader.
    fn new_empty() -> Loader {
        Loader {
            language_override: None,
            auto_language: Language::English,
            bundles: HashMap::new(),
        }
    }

    /// Returns the language currently in use.
    fn current_language(&self) -> Language {
        if let Some(language) = self.language_override {
            language
        } else {
            self.auto_language
        }
    }

    /// Sets `auto_language` to the most sensible language available.
    fn find_auto_language(&mut self) {
        log::warn!("No automatic languages yet");
        self.auto_language = Language::English;
    }

    /// Loads all of the language bundles.
    fn load_all(&mut self) -> Result<()> {
        for language in Language::iter() {
            self.bundles.insert(language, language.load_bundle()?);
        }

        Ok(())
    }

    /// Returns the bundle for the current language.
    fn current_bundle(&self) -> &LanguageBundle {
        self.bundles.get(&self.current_language()).unwrap()
    }
}

/// Loads CLEO's language system.
pub fn init() {
    let mut loader = Loader::lock();

    if let Err(err) = loader.load_all() {
        log::error!("{:?}", err);
        panic!();
    }

    // Set the language override based on the langauge chosen in the settings.
    loader.language_override = crate::meta::settings::Options::get()
        .language_mode
        .language();

    loader.find_auto_language();
}

/// Sets the current translation to the given language, or automatically select a language if
/// `language` is `None`.
pub fn set(language: Option<Language>) {
    Loader::lock().language_override = language;
}

/// Translation information for a single language.
struct LanguageBundle {
    /// The language that this bundle is for.
    language: Language,

    /// The Fluent bundle containing the localisation messages for this language.
    bundle: FluentBundle<FluentResource>,
}

impl LanguageBundle {
    /// Try to format the message for `key` with `args`.
    fn try_format<'me>(
        &'me self,
        key: impl AsRef<str>,
        args: Option<&'me FluentArgs>,
    ) -> Result<Cow<'me, str>> {
        let message = self.bundle.get_message(key.as_ref()).ok_or_else(|| {
            eyre!(
                "message '{}' not found for '{}'",
                key.as_ref(),
                self.language.lang_id()
            )
        })?;

        let mut errors = vec![];

        let formatted = self.bundle.format_pattern(
            message.value().ok_or_else(|| {
                eyre!(
                    "couldn't get value from message {:?} (key {})",
                    message,
                    key.as_ref(),
                )
            })?,
            args,
            &mut errors,
        );

        if !errors.is_empty() {
            return Err(eyre!("formatting error(s): {:?}", errors));
        }

        Ok(formatted)
    }

    /// Format the message for `key` with optional `args`.
    fn format_maybe<'me>(
        &'me self,
        key: impl AsRef<str>,
        args: Option<&'me FluentArgs>,
    ) -> Cow<'me, str> {
        // `as_ref` here so we don't move the key.
        match self.try_format(key.as_ref(), args) {
            Ok(s) => s,
            Err(err) => {
                log::error!(
                    "unable to format {:?} with {:?}: {:?}",
                    key.as_ref(),
                    args,
                    err
                );

                Cow::Owned(key.as_ref().to_string())
            }
        }
    }

    /// Format the message for `key` with `args`, panicking on error.
    fn format<'me>(&'me self, key: impl AsRef<str>, args: &'me FluentArgs) -> Cow<'me, str> {
        self.format_maybe(key, Some(args))
    }

    /// Get the message for `key` directly without any formatting.
    fn get(&self, key: impl AsRef<str>) -> Cow<str> {
        self.format_maybe(key, None)
    }
}

/// Languages that CLEO supports.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Debug, EnumIter,
)]
pub enum Language {
    Arabic,
    Chinese,
    Czech,
    Dutch,
    English,
    Khmer,
    Slovak,
    Turkish,
    Vietnamese,
}

impl Language {
    /// Returns the Unicode language ID for this language.
    fn lang_id(self) -> unic_langid::LanguageIdentifier {
        match self {
            Language::Arabic => "ar",
            Language::Chinese => "cn",
            Language::Czech => "cz",
            Language::Dutch => "nl",
            Language::English => "en",
            Language::Khmer => "km",
            Language::Slovak => "sk",
            Language::Turkish => "tr",
            Language::Vietnamese => "vi",
        }
        .parse()
        .unwrap()
    }

    /// Returns the FTL translation for this language.
    fn ftl_str(self) -> &'static str {
        match self {
            Language::Arabic => include_str!("../../loc/ar.ftl"),
            Language::Chinese => include_str!("../../loc/cn.ftl"),
            Language::Czech => include_str!("../../loc/cz.ftl"),
            Language::Dutch => include_str!("../../loc/nl.ftl"),
            Language::English => include_str!("../../loc/en.ftl"),
            Language::Khmer => include_str!("../../loc/kh.ftl"),
            Language::Slovak => include_str!("../../loc/sk.ftl"),
            Language::Turkish => include_str!("../../loc/tr.ftl"),
            Language::Vietnamese => include_str!("../../loc/vi.ftl"),
        }
    }

    /// Creates and loads a new `LanguageBundle` with resources for this language.
    fn load_bundle(self) -> Result<LanguageBundle> {
        let mut bundle = FluentBundle::new_concurrent(vec![self.lang_id()]);

        let ftl_result = FluentResource::try_new(self.ftl_str().to_owned());
        let ftl = ftl_result.map_err(|(_res, errors)| {
            eyre!(
                "encountered error(s) loading '{}': {:?}",
                self.lang_id(),
                errors
            )
        })?;

        bundle.add_resource(ftl).map_err(|errors| {
            eyre!(
                "encountered error(s) adding FTL for '{}' to bundle: {:?}",
                self.lang_id(),
                errors
            )
        })?;

        Ok(LanguageBundle {
            language: self,
            bundle,
        })
    }

    /// Returns the next most-spoken language after this one. Returns `None` if this is the
    /// least-spoken language that we support.
    pub fn next_most_spoken(self) -> Option<Language> {
        // The number of speakers is only approximate, but should be fine for ordering the
        // languages.
        match self {
            // 1.5 billion speakers
            Language::English => Some(Language::Chinese),

            // 1.1 billion
            Language::Chinese => Some(Language::Arabic),

            // 371 million
            Language::Arabic => Some(Language::Turkish),

            // 88 million
            Language::Turkish => Some(Language::Vietnamese),

            // 85 million
            Language::Vietnamese => Some(Language::Dutch),

            // 30 million
            Language::Dutch => Some(Language::Khmer),

            // 18 million
            Language::Khmer => Some(Language::Czech),

            // 11 million
            Language::Czech => Some(Language::Slovak),

            // 5 million
            Language::Slovak => None,
        }
    }
}

/// Identifies a translated message.
#[derive(Clone)]
pub enum Message {
    Message(MessageKey),
    Formatted(MessageKey, std::rc::Rc<FluentArgs<'static>>),
}

impl Message {
    /// Translates the message into the user's selected language.
    pub fn translate(&self) -> Cow<'static, str> {
        log::warn!(
            "cloning all messages at the moment: {}",
            match self {
                Message::Message(key) => key.key_str(),
                Message::Formatted(key, _) => key.key_str(),
            }
        );

        match self {
            Message::Message(key) => Cow::Owned(
                Loader::lock()
                    .current_bundle()
                    .get(key.key_str())
                    .into_owned(),
            ),

            Message::Formatted(key, args) => Cow::Owned(
                Loader::lock()
                    .current_bundle()
                    .format(key.key_str(), args.as_ref())
                    .into_owned(),
            ),
        }
    }

    pub fn key(&self) -> MessageKey {
        match self {
            Message::Message(key) | Message::Formatted(key, _) => *key,
        }
    }
}

// Implementation before definition because the definition is long.
impl MessageKey {
    pub fn to_message(self) -> Message {
        Message::Message(self)
    }

    pub fn format(self, args: FluentArgs<'static>) -> Message {
        Message::Formatted(self, std::rc::Rc::new(args))
    }

    /// Returns the Fluent key for this message.
    fn key_str(self) -> &'static str {
        self.into()
    }
}

#[derive(Clone, Copy, Debug, EnumString, EnumVariantNames, IntoStaticStr, PartialEq, Eq, Hash)]
#[strum(serialize_all = "kebab-case")]
pub enum MessageKey {
    LanguageOptTitle,
    LanguageOptDesc,

    LanguageName,
    LanguageAutoName,

    SplashLegal,
    SplashFun,

    UpdatePromptTitle,
    UpdatePromptMessage,

    UpdateReleaseChannelOptTitle,
    UpdateReleaseChannelOptDesc,

    UpdateReleaseChannelOptDisabled,
    UpdateReleaseChannelOptStable,
    UpdateReleaseChannelOptAlpha,

    MenuClose,
    MenuOptionsTabTitle,

    MenuScriptWarningOverview,
    MenuScriptSeeBelow,

    MenuScriptCsaTabTitle,
    MenuScriptCsiTabTitle,

    ScriptUnimplementedInCleo,
    ScriptImpossibleOnIos,
    ScriptDuplicate,
    ScriptCheckFailed,
    ScriptNoProblems,

    ScriptCsaRowTitle,
    ScriptCsiRowTitle,

    ScriptRunning,
    ScriptNotRunning,
    ScriptCsaForcedRunning,

    ScriptModeOptTitle,
    ScriptModeOptDesc,

    ScriptModeOptDontBreak,
    ScriptModeOptBreak,

    FpsLockOptTitle,
    FpsLockOptDesc,

    #[strum(serialize = "fps-lock-opt-30")]
    FpsLockOpt30,
    #[strum(serialize = "fps-lock-opt-60")]
    FpsLockOpt60,

    FpsCounterOptTitle,
    FpsCounterOptDesc,

    FpsCounterOptHidden,
    FpsCounterOptEnabled,

    CheatTabTitle,

    CheatMenuWarning,

    CheatOn,
    CheatOff,
    CheatQueuedOn,
    CheatQueuedOff,

    CheatCodeRowTitle,
    CheatNoCodeTitle,

    CheatTransienceOptTitle,
    CheatTransienceOptDesc,

    CheatTransienceOptTransient,
    CheatTransienceOptPersistent,

    CheatThugsArmoury,
    CheatProfessionalsKit,
    CheatNuttersToys,
    CheatWeapons4,

    CheatDebugMappings,
    CheatDebugTapToTarget,
    CheatDebugTargeting,

    CheatINeedSomeHelp,
    CheatSkipMission,

    CheatFullInvincibility,
    CheatStingLikeABee,
    CheatIAmNeverHungry,
    CheatKangaroo,
    CheatNooneCanHurtMe,
    CheatManFromAtlantis,

    CheatWorshipMe,
    CheatHelloLadies,

    CheatWhoAteAllThePies,
    CheatBuffMeUp,
    CheatMaxGambling,
    CheatLeanAndMean,
    CheatICanGoAllNight,

    CheatProfessionalKiller,
    CheatNaturalTalent,

    CheatTurnUpTheHeat,
    CheatTurnDownTheHeat,
    CheatIDoAsIPlease,
    CheatBringItOn,

    CheatPleasantlyWarm,
    CheatTooDamnHot,
    CheatDullDullDay,
    CheatStayInAndWatchTv,
    CheatCantSeeWhereImGoing,
    CheatScottishSummer,
    CheatSandInMyEars,

    CheatClockForward,
    CheatTimeJustFliesBy,
    CheatSpeedItUp,
    CheatSlowItDown,
    CheatNightProwler,
    CheatDontBringOnTheNight,

    CheatLetsGoBaseJumping,
    CheatRocketman,

    CheatTimeToKickAss,
    CheatOldSpeedDemon,
    CheatTintedRancher,
    CheatNotForPublicRoads,
    CheatJustTryAndStopMe,
    CheatWheresTheFuneral,
    CheatCelebrityStatus,
    CheatTrueGrime,
    #[strum(serialize = "cheat-18-holes")]
    Cheat18Holes,
    CheatJumpJet,
    CheatIWantToHover,
    CheatOhDude,
    CheatFourWheelFun,
    CheatHitTheRoadJack,
    CheatItsAllBull,
    CheatFlyingToStunt,
    CheatMonsterMash,

    CheatWannaBeInMyGang,
    CheatNooneCanStopUs,
    CheatRocketMayhem,

    CheatAllDriversAreCriminals,
    CheatPinkIsTheNewCool,
    CheatSoLongAsItsBlack,
    CheatEveryoneIsPoor,
    CheatEveryoneIsRich,

    CheatRoughNeighbourhood,
    CheatStopPickingOnMe,
    CheatSurroundedByNutters,
    CheatBlueSuedeShoes,
    CheatAttackOfTheVillagePeople,
    CheatOnlyHomiesAllowed,
    CheatBetterStayIndoors,
    CheatStateOfEmergency,
    CheatGhostTown,

    CheatNinjaTown,
    CheatLoveConquersAll,
    CheatLifesABeach,
    CheatHicksville,
    CheatCrazyTown,

    CheatAllCarsGoBoom,
    CheatWheelsOnlyPlease,
    CheatSidewaysWheels,
    CheatSpeedFreak,
    CheatCoolTaxis,

    CheatChittyChittyBangBang,
    CheatCjPhoneHome,
    CheatTouchMyCarYouDie,
    CheatBubbleCars,
    CheatStickLikeGlue,
    CheatDontTryAndStopMe,
    CheatFlyingFish,

    CheatFullClip,
    CheatIWannaDriveby,

    CheatGoodbyeCruelWorld,
    CheatTakeAChillPill,
    CheatProstitutesPay,

    CheatXboxHelper,

    CheatSlotMelee,
    CheatSlotHandgun,
    CheatSlotSmg,
    CheatSlotShotgun,
    CheatSlotAssaultRifle,
    CheatSlotLongRifle,
    CheatSlotThrown,
    CheatSlotHeavy,
    CheatSlotEquipment,
    CheatSlotOther,

    CheatPredator,
}
