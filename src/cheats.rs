//! Replaces the game's broken cheats system with our own system that integrates with the menu.

use std::{
    borrow::Cow,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    call_original, gui, hook,
    language::{Message, MessageKey},
    menu::{self, RowData, TabData},
    settings::{CheatTransience, Options},
};
use lazy_static::lazy_static;
use log::error;
use once_cell::sync::Lazy;

pub struct Cheat {
    index: usize,
    code: &'static str,
    description: MessageKey,
}

lazy_static! {
    static ref WAITING_CHEATS: std::sync::Mutex<Vec<usize>> = std::sync::Mutex::new(vec![]);
}

impl Cheat {
    const fn new(index: usize, code: &'static str, description: MessageKey) -> Cheat {
        Cheat {
            index,
            code,
            description,
        }
    }

    fn get_function(&self) -> Option<fn()> {
        let entry_address = 0x10065c358 + (self.index as usize * 8);
        let ptr = hook::slide::<*const *const u64>(entry_address);

        // The array pointer shouldn't be null, but we check it just in case.
        // The more important check is the second, which ensures that the function pointer is not 0.
        if ptr.is_null() || unsafe { *ptr }.is_null() {
            None
        } else {
            // Get the value again, but this time as a pointer to a function.
            // The reason we don't get it as a *const fn() the first time is that 'fn' is itself
            //  the function pointer, but we can't check if it is null. We use *const *const u64
            //  instead because we can check the inner pointer as well.
            let func_ptr = hook::slide::<*const fn()>(entry_address);
            Some(unsafe { *func_ptr })
        }
    }

    fn get_active_mut(&self) -> &'static mut bool {
        unsafe {
            hook::slide::<*mut bool>(0x10072dda8 + (self.index as usize))
                .as_mut()
                .unwrap()
        }
    }

    fn is_active(&self) -> bool {
        *self.get_active_mut()
    }

    fn queue(&self) {
        let mut waiting = WAITING_CHEATS.lock().unwrap();
        waiting.push(self.index);
    }

    fn cancel(&self) {
        let mut waiting = WAITING_CHEATS.lock().unwrap();
        waiting.retain(|cheat_index| *cheat_index != self.index);
    }

    fn is_in_queue(&self) -> bool {
        let waiting = WAITING_CHEATS.lock().unwrap();
        waiting.contains(&self.index)
    }

    fn run(&self) {
        if let Some(function) = self.get_function() {
            log::info!("Calling cheat function {:?}", function);
            function();
            return;
        }

        // If the cheat has no function pointer, then we need to toggle its active status.
        let active = self.get_active_mut();
        *active = !*active;
    }
}

// (a, b) where a = "check b" and b = "save the cheat states". a is false when a save is in progress.
// This is a weak system for avoiding two saves happening at the same time, but it works well enough.
// Besides, it's practically impossible for a save to be triggered when one is already in progress because
//  saving is very fast.
static SAVE_FLAGS: Lazy<(AtomicBool, AtomicBool)> =
    Lazy::new(|| (AtomicBool::new(true), AtomicBool::new(false)));

// CCheat::DoCheats is where cheat codes are checked and then cheats activated (indirectly),
//  so we need to do our cheat stuff here to ensure that the cheats don't fuck up the game by
//  doing stuff at weird times. The point in CGame::Process where DoCheats is called is where
//  every other system in the game expects cheats to be activated.
// Cheats that need textures to be loaded - such as weapon or vehicle cheats - can crash the
//  game if they are executed on the wrong thread or at the wrong time, so it is very important
//  that we get this right.
fn do_cheats() {
    if let Ok(waiting) = WAITING_CHEATS.lock().as_mut() {
        // Perform all queued cheat actions.
        for cheat_index in waiting.iter() {
            CHEATS[*cheat_index as usize].run();
        }

        // Clear the queue.
        waiting.clear();
    } else {
        error!("Unable to lock cheat queue for CCheat::DoCheats!");
    }

    if SAVE_FLAGS.0.load(Ordering::SeqCst) && SAVE_FLAGS.1.load(Ordering::SeqCst) {
        // Ignore further requests to save the cheat states.
        SAVE_FLAGS.0.store(false, Ordering::SeqCst);

        // We need to save the cheat states now. We only do this after the cheat functions have been called because
        //  some will clear their "enabled" status when they run.
        // todo: Check that saving cheats after execution is always a good idea.

        // We just save an array of bytes in the order of the cheats, with each being 1 or 0 depending on the status of that cheat.
        // We do this outside of our saving thread because we don't want the statuses to change while we're accessing them.
        let cheat_state_bytes: Vec<u8> = CHEATS
            .iter()
            .map(|cheat| {
                let is_active = cheat.is_active();
                if is_active {
                    log::info!("Saving status ON for cheat '{}'.", cheat.code);
                }
                is_active as u8
            })
            .collect();

        std::thread::spawn(|| {
            if let Err(err) = std::fs::write(
                crate::resources::get_documents_path("cleo_saved_cheats.u8"),
                cheat_state_bytes,
            ) {
                log::error!("Error while saving cheat states: {}", err);
            } else {
                log::info!("Cheat states saved successfully.");
            }

            // Clear save request flag (so we don't keep saving infinitely).
            SAVE_FLAGS.1.store(false, Ordering::SeqCst);

            // Allow new saving requests to be processed.
            SAVE_FLAGS.0.store(true, Ordering::SeqCst);
        });
    }
}

struct CheatData {
    cheat: &'static Cheat,
    queued_state: Option<bool>,
}

impl CheatData {
    fn new(cheat: &'static Cheat) -> CheatData {
        CheatData {
            cheat,
            queued_state: if cheat.is_in_queue() {
                Some(!cheat.is_active())
            } else {
                None
            },
        }
    }

    fn will_be_active(&self) -> bool {
        self.queued_state.unwrap_or_else(|| self.cheat.is_active())
    }
}

impl RowData for CheatData {
    fn title(&self) -> Message {
        if self.cheat.code.is_empty() {
            MessageKey::CheatNoCodeTitle.to_message()
        } else {
            MessageKey::CheatCodeRowTitle.format(todo!("cheat code title"))
        }
    }

    fn detail(&self) -> menu::RowDetail {
        menu::RowDetail::Info(self.cheat.description.to_message())
    }

    fn value(&self) -> Message {
        /*
            State                       Tint        Status

            In queue, turning on        Blue        "Queued On"
            In queue, turning off       Red         "Queued Off"
            Not in queue, on            Green       "On"
            Not in queue, off           None        "Off"
        */

        let will_be_active = self.will_be_active();

        let key = if self.cheat.is_in_queue() {
            if will_be_active {
                MessageKey::CheatQueuedOn
            } else {
                MessageKey::CheatQueuedOff
            }
        } else if will_be_active {
            MessageKey::CheatOn
        } else {
            MessageKey::CheatOff
        };

        key.to_message()
    }

    fn tint(&self) -> Option<(u8, u8, u8)> {
        let will_be_active = self.will_be_active();

        if self.cheat.is_in_queue() {
            if will_be_active {
                Some(gui::colours::BLUE)
            } else {
                Some(gui::colours::RED)
            }
        } else if will_be_active {
            Some(gui::colours::GREEN)
        } else {
            None
        }
    }

    fn handle_tap(&mut self) -> bool {
        let is_queued = self.queued_state.is_some();

        if is_queued {
            // Remove the cheat from the queue.
            self.cheat.cancel();
            self.queued_state = None;
        } else {
            self.queued_state = Some(!self.cheat.is_active());

            // Not queued yet.
            self.cheat.queue();
        }

        if let CheatTransience::Persistent = Options::get().cheat_transience {
            // Request that the cheats be saved because it is likely that a status will change.
            SAVE_FLAGS.1.store(true, Ordering::SeqCst);
        }

        true
    }
}

pub fn tab_data() -> TabData {
    let sorted_cheats: Lazy<Vec<&Cheat>> = Lazy::new(|| {
        let mut vec: Vec<&Cheat> = CHEATS.iter().by_ref().collect();

        vec.sort_by_key(|cheat| {
            if cheat.code.is_empty() {
                // Push cheats without codes to the end. If we don't do this, the cheat menu only shows "???" for the first few rows.
                "ZZZZZ"
            } else {
                cheat.code
            }
        });

        vec
    });

    TabData {
        name: "Cheats".to_string(),
        warning: Some(
            r#"Using cheats can lead to a crash and/or loss of progress.
If you don't want to risk breaking your save, back up your progress to a different slot first."#
                .to_string(),
        ),
        row_data: sorted_cheats
            .iter()
            .map(|cheat| Box::new(CheatData::new(cheat)) as Box<dyn RowData>)
            .collect(),
    }
}

fn reset_cheats() {
    log::info!("Resetting cheats");
    call_original!(crate::targets::reset_cheats);

    if !matches!(Options::get().cheat_transience, CheatTransience::Persistent) {
        log::info!("Cheat saving/loading is disabled.");
        return;
    }

    log::info!("Loading saved cheats.");

    let path = crate::resources::get_documents_path("cleo_saved_cheats.u8");

    if !path.exists() {
        log::info!("No saved cheats file found.");
        return;
    }

    match std::fs::read(path) {
        Ok(loaded_bytes) => {
            // Check that we have a file that matches our cheat count.
            if loaded_bytes.len() != CHEATS.len() {
                log::error!("Invalid cheat save: byte count must match cheat count.");
                return;
            }

            // Ensure that all the bytes are valid.
            for byte in &loaded_bytes {
                if byte != &0 && byte != &1 {
                    log::error!("Invalid cheat save: found non-Boolean byte.");
                    return;
                }
            }

            // Set all the cheat statuses according to the bytes in the file.
            for (i, b) in loaded_bytes.iter().enumerate() {
                *CHEATS[i].get_active_mut() = b == &1;
            }

            log::info!("Cheats loaded successfully.");
        }
        Err(err) => {
            log::error!("Error loading cheat file: {}", err);
        }
    }
}

pub fn init() {
    log::info!("installing cheat hooks...");
    crate::targets::do_cheats::install(do_cheats);
    crate::targets::reset_cheats::install(reset_cheats);
}

// We have to include the codes because the game doesn't have the array.
// Android does, though, so I copied the codes from there. The order has been preserved.
// The spreadsheet at
//   https://docs.google.com/spreadsheets/d/1-rmga12W9reALga7fct22tJ-1thxbbsfGiGltK2qgh0/edit?usp=sharing
//  was very helpful during research, and the page at https://gta.fandom.com/wiki/Cheats_in_GTA_San_Andreas
//  was really useful for writing cheat descriptions.
static CHEATS: [Cheat; 111] = [
    Cheat::new(0, "THUGSARMOURY", MessageKey::CheatThugsArmoury),
    Cheat::new(1, "PROFESSIONALSKIT", MessageKey::CheatProfessionalsKit),
    Cheat::new(2, "NUTTERSTOYS", MessageKey::CheatNuttersToys),
    Cheat::new(3, "", MessageKey::CheatWeapons4),
    Cheat::new(4, "", MessageKey::CheatClockForward),
    Cheat::new(5, "", MessageKey::CheatSkipMission),
    Cheat::new(6, "", MessageKey::CheatDebugMappings),
    Cheat::new(7, "", MessageKey::CheatFullInvincibility),
    Cheat::new(8, "", MessageKey::CheatDebugTapToTarget),
    Cheat::new(9, "", MessageKey::CheatDebugTargeting),
    Cheat::new(10, "INEEDSOMEHELP", MessageKey::CheatINeedSomeHelp),
    Cheat::new(11, "TURNUPTHEHEAT", MessageKey::CheatTurnUpTheHeat),
    Cheat::new(12, "TURNDOWNTHEHEAT", MessageKey::CheatTurnDownTheHeat),
    Cheat::new(13, "PLEASANTLYWARM", MessageKey::CheatPleasantlyWarm),
    Cheat::new(14, "TOODAMNHOT", MessageKey::CheatTooDamnHot),
    Cheat::new(15, "DULLDULLDAY", MessageKey::CheatDullDullDay),
    Cheat::new(16, "STAYINANDWATCHTV", MessageKey::CheatStayInAndWatchTv),
    Cheat::new(
        17,
        "CANTSEEWHEREIMGOING",
        MessageKey::CheatCantSeeWhereImGoing,
    ),
    Cheat::new(18, "TIMEJUSTFLIESBY", MessageKey::CheatTimeJustFliesBy),
    Cheat::new(19, "SPEEDITUP", MessageKey::CheatSpeedItUp),
    Cheat::new(20, "SLOWITDOWN", MessageKey::CheatSlowItDown),
    Cheat::new(
        21,
        "ROUGHNEIGHBOURHOOD",
        MessageKey::CheatRoughNeighbourhood,
    ),
    Cheat::new(22, "STOPPICKINGONME", MessageKey::CheatStopPickingOnMe),
    Cheat::new(
        23,
        "SURROUNDEDBYNUTTERS",
        MessageKey::CheatSurroundedByNutters,
    ),
    Cheat::new(24, "TIMETOKICKASS", MessageKey::CheatTimeToKickAss),
    Cheat::new(25, "OLDSPEEDDEMON", MessageKey::CheatOldSpeedDemon),
    Cheat::new(26, "", MessageKey::CheatTintedRancher),
    Cheat::new(27, "NOTFORPUBLICROADS", MessageKey::CheatNotForPublicRoads),
    Cheat::new(28, "JUSTTRYANDSTOPME", MessageKey::CheatJustTryAndStopMe),
    Cheat::new(29, "WHERESTHEFUNERAL", MessageKey::CheatWheresTheFuneral),
    Cheat::new(30, "CELEBRITYSTATUS", MessageKey::CheatCelebrityStatus),
    Cheat::new(31, "TRUEGRIME", MessageKey::CheatTrueGrime),
    Cheat::new(32, "18HOLES", MessageKey::Cheat18Holes),
    Cheat::new(33, "ALLCARSGOBOOM", MessageKey::CheatAllCarsGoBoom),
    Cheat::new(34, "WHEELSONLYPLEASE", MessageKey::CheatWheelsOnlyPlease),
    Cheat::new(35, "STICKLIKEGLUE", MessageKey::CheatStickLikeGlue),
    Cheat::new(36, "GOODBYECRUELWORLD", MessageKey::CheatGoodbyeCruelWorld),
    Cheat::new(37, "DONTTRYANDSTOPME", MessageKey::CheatDontTryAndStopMe),
    Cheat::new(
        38,
        "ALLDRIVERSARECRIMINALS",
        MessageKey::CheatAllDriversAreCriminals,
    ),
    Cheat::new(39, "PINKISTHENEWCOOL", MessageKey::CheatPinkIsTheNewCool),
    Cheat::new(40, "SOLONGASITSBLACK", MessageKey::CheatSoLongAsItsBlack),
    Cheat::new(41, "", MessageKey::CheatSidewaysWheels),
    Cheat::new(42, "FLYINGFISH", MessageKey::CheatFlyingFish),
    Cheat::new(43, "WHOATEALLTHEPIES", MessageKey::CheatWhoAteAllThePies),
    Cheat::new(44, "BUFFMEUP", MessageKey::CheatBuffMeUp),
    Cheat::new(45, "", MessageKey::CheatMaxGambling),
    Cheat::new(46, "LEANANDMEAN", MessageKey::CheatLeanAndMean),
    Cheat::new(47, "BLUESUEDESHOES", MessageKey::CheatBlueSuedeShoes),
    Cheat::new(
        48,
        "ATTACKOFTHEVILLAGEPEOPLE",
        MessageKey::CheatAttackOfTheVillagePeople,
    ),
    Cheat::new(49, "LIFESABEACH", MessageKey::CheatLifesABeach),
    Cheat::new(50, "ONLYHOMIESALLOWED", MessageKey::CheatOnlyHomiesAllowed),
    Cheat::new(51, "BETTERSTAYINDOORS", MessageKey::CheatBetterStayIndoors),
    Cheat::new(52, "NINJATOWN", MessageKey::CheatNinjaTown),
    Cheat::new(53, "LOVECONQUERSALL", MessageKey::CheatLoveConquersAll),
    Cheat::new(54, "EVERYONEISPOOR", MessageKey::CheatEveryoneIsPoor),
    Cheat::new(55, "EVERYONEISRICH", MessageKey::CheatEveryoneIsRich),
    Cheat::new(
        56,
        "CHITTYCHITTYBANGBANG",
        MessageKey::CheatChittyChittyBangBang,
    ),
    Cheat::new(57, "CJPHONEHOME", MessageKey::CheatCjPhoneHome),
    Cheat::new(58, "JUMPJET", MessageKey::CheatJumpJet),
    Cheat::new(59, "IWANTTOHOVER", MessageKey::CheatIWantToHover),
    Cheat::new(60, "TOUCHMYCARYOUDIE", MessageKey::CheatTouchMyCarYouDie),
    Cheat::new(61, "SPEEDFREAK", MessageKey::CheatSpeedFreak),
    Cheat::new(62, "BUBBLECARS", MessageKey::CheatBubbleCars),
    Cheat::new(63, "NIGHTPROWLER", MessageKey::CheatNightProwler),
    Cheat::new(
        64,
        "DONTBRINGONTHENIGHT",
        MessageKey::CheatDontBringOnTheNight,
    ),
    Cheat::new(65, "SCOTTISHSUMMER", MessageKey::CheatScottishSummer),
    Cheat::new(66, "SANDINMYEARS", MessageKey::CheatSandInMyEars),
    Cheat::new(67, "", MessageKey::CheatPredator),
    Cheat::new(68, "KANGAROO", MessageKey::CheatKangaroo),
    Cheat::new(69, "NOONECANHURTME", MessageKey::CheatNooneCanHurtMe),
    Cheat::new(70, "MANFROMATLANTIS", MessageKey::CheatManFromAtlantis),
    Cheat::new(71, "LETSGOBASEJUMPING", MessageKey::CheatLetsGoBaseJumping),
    Cheat::new(72, "ROCKETMAN", MessageKey::CheatRocketman),
    Cheat::new(73, "IDOASIPLEASE", MessageKey::CheatIDoAsIPlease),
    Cheat::new(74, "BRINGITON", MessageKey::CheatBringItOn),
    Cheat::new(75, "STINGLIKEABEE", MessageKey::CheatStingLikeABee),
    Cheat::new(76, "IAMNEVERHUNGRY", MessageKey::CheatIAmNeverHungry),
    Cheat::new(77, "STATEOFEMERGENCY", MessageKey::CheatStateOfEmergency),
    Cheat::new(78, "CRAZYTOWN", MessageKey::CheatCrazyTown),
    Cheat::new(79, "TAKEACHILLPILL", MessageKey::CheatTakeAChillPill),
    Cheat::new(80, "FULLCLIP", MessageKey::CheatFullClip),
    Cheat::new(81, "IWANNADRIVEBY", MessageKey::CheatIWannaDriveby),
    Cheat::new(82, "GHOSTTOWN", MessageKey::CheatGhostTown),
    Cheat::new(83, "HICKSVILLE", MessageKey::CheatHicksville),
    Cheat::new(84, "WANNABEINMYGANG", MessageKey::CheatWannaBeInMyGang),
    Cheat::new(85, "NOONECANSTOPUS", MessageKey::CheatNooneCanStopUs),
    Cheat::new(86, "ROCKETMAYHEM", MessageKey::CheatRocketMayhem),
    Cheat::new(87, "WORSHIPME", MessageKey::CheatWorshipMe),
    Cheat::new(88, "HELLOLADIES", MessageKey::CheatHelloLadies),
    Cheat::new(89, "ICANGOALLNIGHT", MessageKey::CheatICanGoAllNight),
    Cheat::new(
        90,
        "PROFESSIONALKILLER",
        MessageKey::CheatProfessionalKiller,
    ),
    Cheat::new(91, "NATURALTALENT", MessageKey::CheatNaturalTalent),
    Cheat::new(92, "OHDUDE", MessageKey::CheatOhDude),
    Cheat::new(93, "FOURWHEELFUN", MessageKey::CheatFourWheelFun),
    Cheat::new(94, "HITTHEROADJACK", MessageKey::CheatHitTheRoadJack),
    Cheat::new(95, "ITSALLBULL", MessageKey::CheatItsAllBull),
    Cheat::new(96, "FLYINGTOSTUNT", MessageKey::CheatFlyingToStunt),
    Cheat::new(97, "MONSTERMASH", MessageKey::CheatMonsterMash),
    Cheat::new(98, "", MessageKey::CheatProstitutesPay),
    Cheat::new(99, "", MessageKey::CheatCoolTaxis),
    Cheat::new(100, "", MessageKey::CheatSlotMelee),
    Cheat::new(101, "", MessageKey::CheatSlotHandgun),
    Cheat::new(102, "", MessageKey::CheatSlotSmg),
    Cheat::new(103, "", MessageKey::CheatSlotShotgun),
    Cheat::new(104, "", MessageKey::CheatSlotAssaultRifle),
    Cheat::new(105, "", MessageKey::CheatSlotLongRifle),
    Cheat::new(106, "", MessageKey::CheatSlotThrown),
    Cheat::new(107, "", MessageKey::CheatSlotHeavy),
    Cheat::new(108, "", MessageKey::CheatSlotEquipment),
    Cheat::new(109, "", MessageKey::CheatSlotOther),
    Cheat::new(110, "", MessageKey::CheatXboxHelper),
];
