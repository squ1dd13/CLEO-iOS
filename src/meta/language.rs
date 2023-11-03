use eyre::{eyre, Result};
use fluent::{concurrent::FluentBundle, FluentArgs, FluentResource};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    ffi::{CStr, CString},
    sync::Mutex,
};
use strum::{EnumIter, EnumString, EnumVariantNames, IntoEnumIterator, IntoStaticStr};

use super::gui::{Font, FontSet};
pub use fluent::fluent_args as msg_args;
use objc::runtime::Object;

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
        self.auto_language = Language::system_language().unwrap_or(Language::English);
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
        panic!("failed to load all languages: {err:?}");
    }

    // Set the language override based on the langauge chosen in the settings.
    loader.language_override = crate::meta::settings::Options::get()
        .language_mode
        .language();

    loader.find_auto_language();
}

/// Returns the current language.
pub fn current() -> Language {
    Loader::lock().current_language()
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
    Galactic,
    Khmer,
    Russian,
    Slovak,
    Spanish,
    Thai,
    Turkish,
    Vietnamese,
}

impl Language {
    /// Returns the `Language` variant matching the given identifier, or `None` if no such language
    /// exists for CLEO.
    fn from_id(id: impl AsRef<str>) -> Option<Language> {
        Some(match id.as_ref() {
            "ar" => Language::Arabic,
            "zh" => Language::Chinese,
            "cz" => Language::Czech,
            "nl" => Language::Dutch,
            "en" => Language::English,
            "mc" => Language::Galactic,
            "km" => Language::Khmer,
            "sk" => Language::Slovak,
            "es" => Language::Spanish,
            "th" => Language::Thai,
            "tr" => Language::Turkish,
            "vi" => Language::Vietnamese,
            _ => return None,
        })
    }

    /// Returns the Unicode language ID for this language.
    fn lang_id(self) -> unic_langid::LanguageIdentifier {
        match self {
            Language::Arabic => "ar",
            Language::Chinese => "zh",
            Language::Czech => "cz",
            Language::Dutch => "nl",
            Language::English => "en",
            Language::Galactic => "mc",
            Language::Khmer => "km",
            Language::Russian => "ru",
            Language::Slovak => "sk",
            Language::Spanish => "es",
            Language::Thai => "th",
            Language::Turkish => "tr",
            Language::Vietnamese => "vi",
        }
        .parse()
        .unwrap()
    }

    /// Returns the FTL translation for this language.
    const fn ftl_str(self) -> &'static str {
        match self {
            Language::Arabic => include_str!("../../loc/ar.ftl"),
            Language::Chinese => include_str!("../../loc/zh.ftl"),
            Language::Czech => include_str!("../../loc/cz.ftl"),
            Language::Dutch => include_str!("../../loc/nl.ftl"),
            Language::English => include_str!("../../loc/en.ftl"),
            Language::Galactic => include_str!("../../loc/mc.ftl"),
            Language::Khmer => include_str!("../../loc/km.ftl"),
            Language::Russian => include_str!("../../loc/ru.ftl"),
            Language::Slovak => include_str!("../../loc/sk.ftl"),
            Language::Spanish => include_str!("../../loc/es.ftl"),
            Language::Thai => include_str!("../../loc/th.ftl"),
            Language::Turkish => include_str!("../../loc/tr.ftl"),
            Language::Vietnamese => include_str!("../../loc/vi.ftl"),
        }
    }

    /// Returns the set of fonts that should be used for this language.
    pub fn font_set(self) -> FontSet {
        // Define some standard sizes to work with.
        const STD_TITLE: f64 = 25.0;
        const STD_SMALL: f64 = 10.0;
        const STD_TEXT: f64 = 15.0;
        const STD_SUBTITLE: f64 = 17.0;

        match self {
            // Czech and Slovak need a font that covers all of the accented characters. Arabic
            // always makes use of Geeza Pro (I think) so we could just leave it with Chalet
            // Comprime, but Avenir Next has easier-to-read Latin letters.
            Language::Arabic | Language::Czech | Language::Slovak => FontSet {
                title_font: Font::AvenirNextHeavy,
                title_size: STD_TITLE,
                small_font: Font::AvenirNextMedium,
                small_size: STD_SMALL,
                text_font: Font::AvenirNextCondensed,
                text_size: STD_TEXT,
                subtitle_font: Font::AvenirNextCondensed,
                subtitle_size: STD_SUBTITLE,
            },

            // PingFang SC is the variant of PingFang for Simplified Chinese. It has large and
            // obvious Latin characters.
            Language::Chinese => FontSet {
                title_font: Font::PingFangSemibold,
                title_size: STD_TITLE,
                small_font: Font::PingFangMedium,
                small_size: STD_SMALL,
                text_font: Font::PingFangLight,
                text_size: STD_TEXT,
                subtitle_font: Font::PingFangMedium,
                subtitle_size: STD_SUBTITLE,
            },

            // Chalet Comprime has support for rich Latin alphabets, so English, Spanish and Dutch
            // are fine. Our Turkish translation uses only ASCII, so it's fine too. Russian also
            // seems to work OK.
            Language::Dutch
            | Language::English
            | Language::Galactic
            | Language::Russian
            | Language::Spanish
            | Language::Turkish => FontSet {
                title_font: Font::Pricedown,
                title_size: STD_TITLE,
                small_font: Font::AvenirNextMedium,
                small_size: STD_SMALL,
                text_font: Font::ChaletComprime,
                text_size: STD_TEXT,
                subtitle_font: Font::ChaletComprime,
                subtitle_size: STD_SUBTITLE + 2.0,
            },

            // Khmer characters always fall back to Khmer Sangam MN, because it's the only font
            // that has them (I think), so the only reason to use it explicitly is for the Latin
            // characters.
            Language::Khmer => FontSet {
                title_font: Font::KhmerSangam,
                title_size: STD_TITLE,
                small_font: Font::KhmerSangam,
                small_size: STD_SMALL,
                text_font: Font::KhmerSangam,
                text_size: STD_TEXT,
                subtitle_font: Font::KhmerSangam,
                subtitle_size: STD_SUBTITLE,
            },

            Language::Thai | Language::Vietnamese => FontSet {
                title_font: Font::AvenirNextMedium,
                title_size: STD_TITLE,
                small_font: Font::AvenirNextMedium,
                small_size: STD_SMALL,
                text_font: Font::AvenirNextCondensed,
                text_size: STD_TEXT,
                subtitle_font: Font::AvenirNextCondensed,
                subtitle_size: STD_SUBTITLE,
            },
        }
    }

    /// Returns `true` if this language is read right-to-left.
    pub fn is_rtl(self) -> bool {
        matches!(self, Language::Arabic)
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

    /// Returns the system's language, or `None` if the system language isn't available for CLEO.
    fn system_language() -> Option<Language> {
        // Normally we'd use `[[NSLocale currentLocale] languageCode]` to get the language code for
        // the app, but GTA only offers the system the languages that it supports, so iOS will only
        // ever set the current locale for the app to one of them. If we ask for the user's
        // preferred languages instead, we can find out what they actually want.

        let preferred_languages: *const Object = unsafe {
            let class = objc::class!(NSLocale);

            // Class methods are just instance methods of the metaclass, so to check for the
            // existence of the `_globalPreferredLanguages` class method on `NSLocale` we need to
            // find `NSLocale`'s metaclass first.
            let metaclass = {
                let name = CString::new("NSLocale").unwrap();

                objc::runtime::objc_getMetaClass(name.as_ptr())
                    .as_ref()
                    .expect("couldn't find NSLocale metaclass")
            };

            // We're supposed to use `preferredLanguages` here, but it doesn't always return the
            // same value on different game launches, even less than a minute apart and without
            // changing any settings. `_globalPreferredLanguages` seems to be (more) stable, but I
            // can't verify how far back it exists. I've only found headers with it from iOS 15,
            // but it exists on my iOS 13 iP8. We'll use `preferredLanguages` as a fallback only.
            if metaclass
                .instance_method(objc::sel!(_globalPreferredLanguages))
                .is_some()
            {
                log::info!("_globalPreferredLanguages exists");
                objc::msg_send![class, _globalPreferredLanguages]
            } else {
                log::warn!("_globalPreferredLanguages does not exist");
                objc::msg_send![class, preferredLanguages]
            }
        };

        let language_count: i32 = unsafe { objc::msg_send![preferred_languages, count] };

        let mut preferred_languages = (0..language_count).into_iter().map(|index| {
            let language_code = &unsafe {
                let nsstring: *const Object =
                    objc::msg_send![preferred_languages, objectAtIndex: index];

                CStr::from_ptr(objc::msg_send![nsstring, UTF8String])
            }
            .to_str()
            // Take only the first two characters, because we don't want the region identifier.
            // Also, iOS does some pretty weird things, like invent `nl-GB`.
            .expect("invalid language identifier string")[..2];

            log::info!("Language {index} is {language_code}");

            language_code
        });

        // Find the first language in the array that we have in CLEO.
        preferred_languages.find_map(Language::from_id)
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
            Language::Chinese => Some(Language::Spanish),

            // 475 million
            Language::Spanish => Some(Language::Arabic),

            // 371 million
            Language::Arabic => Some(Language::Russian),

            // 260 million
            Language::Russian => Some(Language::Turkish),

            // 88 million
            Language::Turkish => Some(Language::Vietnamese),

            // 85 million
            Language::Vietnamese => Some(Language::Thai),

            // 40 million
            Language::Thai => Some(Language::Dutch),

            // 30 million
            Language::Dutch => Some(Language::Khmer),

            // 18 million
            Language::Khmer => Some(Language::Czech),

            // 11 million
            Language::Czech => Some(Language::Slovak),

            // 5 million
            Language::Slovak => Some(Language::Galactic),

            // No speakers
            Language::Galactic => None,
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

    MenuGestureOptTitle,
    MenuGestureOptDesc,

    MenuGestureOptOneFingerSwipe,
    MenuGestureOptTwoFingerSwipe,
    MenuGestureOptTwoFingerTap,
    MenuGestureOptThreeFingerTap,

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
    #[strum(serialize = "cheat-weapons-4")]
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
