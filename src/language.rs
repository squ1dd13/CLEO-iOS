use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use strum::{EnumString, EnumVariantNames, IntoStaticStr};

/// Returns the message associated with `key` in the current language.
pub fn message(key: MessageKey) -> Cow<'static, str> {
    todo!()
}

/// Sets the current translation to the given language, or automatically select a language if
/// `language` is `None`.
pub fn set(language: Option<Language>) {
    todo!()
}

/// Languages that CLEO supports.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Language {
    Arabic,
    Chinese,
    Czech,
    English,
    Khmer,
    Dutch,
    Slovak,
    Turkish,
}

impl Language {
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

            // 80 million
            Language::Turkish => Some(Language::Dutch),

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
    Formatted(MessageKey, ()),
}

impl Message {
    /// Translates the message into the user's selected language.
    pub fn translate(self) -> Cow<'static, str> {
        match self {
            Message::Message(key) => message(key),
            Message::Formatted(_, _) => todo!("formatting"),
        }
    }
}

// Implementation before definition because the definition is long.
impl MessageKey {
    pub fn to_message(self) -> Message {
        Message::Message(self)
    }

    pub fn format(self, args: ()) -> Message {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, EnumString, EnumVariantNames, IntoStaticStr)]
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

    FpsLockOpt30,
    FpsLockOpt60,

    FpsCounterOptTitle,
    FpsCounterOptDesc,

    FpsCounterOptHidden,
    FpsCounterOptEnabled,

    CheatTabTitle,

    CheatMenuWarning,
    CheatMenuAdvice,

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
