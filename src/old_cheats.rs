//! Replaces the game's broken cheats system with our own system that integrates with the menu.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

use log::error;
use once_cell::sync::Lazy;

use crate::{
    call_original, hook,
    settings::Settings,
    ui::{self, RowData, TabData},
};

pub struct Cheat {
    index: usize,
    code: &'static str,
    description: &'static str,
}

static WAITING_CHEATS: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(vec![]));

impl Cheat {
    const fn new(index: usize, code: &'static str, description: &'static str) -> Cheat {
        Cheat {
            index,
            code,
            description,
        }
    }

    fn get_function(&self) -> Option<fn()> {
        let entry_address = 0x10065c358 + (self.index as usize * 8);
        let ptr = hook::slide::<*const *const u64>(entry_address);

        // The array pointer shouldn't be null, but we check it just in case. The more important
        // check is the second, which ensures that the function pointer is not null.
        if ptr.is_null() || unsafe { *ptr }.is_null() {
            None
        } else {
            // Get the value again, but this time as a pointer to a function. The reason we don't
            // get it as a *const fn() the first time is that 'fn' is itself the function pointer,
            // but we can't check if it is null. We use *const *const u64 instead because we can
            // check the inner pointer as well.
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

// (a, b) where a = "check b" and b = "save the cheat states". a is false when a save is in
// progress. This is a weak system for avoiding two saves happening at the same time, but it works
// well enough. Besides, it's practically impossible for a save to be triggered when one is already
// in progress because saving is very fast.
static SAVE_FLAGS: Lazy<(AtomicBool, AtomicBool)> =
    Lazy::new(|| (AtomicBool::new(true), AtomicBool::new(false)));

// CCheat::DoCheats is where cheat codes are checked and then cheats activated (indirectly), so we
// need to do our cheat stuff here to ensure that the cheats don't fuck up the game by doing stuff
// at weird times. The point in CGame::Process where DoCheats is called is where every other system
// in the game expects cheats to be activated. Cheats that need textures to be loaded - such as
// weapon or vehicle cheats - can crash the game if they are executed on the wrong thread or at the
// wrong time, so it is very important that we get this right.
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

        // We need to save the cheat states now. We only do this after the cheat functions have
        // been called because some will clear their "enabled" status when they run.
        // todo: Check that saving cheats after execution is always a good idea.

        // We just save an array of bytes in the order of the cheats, with each being 1 or 0
        // depending on the status of that cheat. We do this outside of our saving thread because
        // we don't want the statuses to change while we're accessing them.
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
                crate::files::get_documents_path("cleo_saved_cheats.u8"),
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
    fn title(&self) -> String {
        if self.cheat.code.is_empty() {
            "???"
        } else {
            self.cheat.code
        }
        .into()
    }

    fn detail(&self) -> ui::RowDetail {
        ui::RowDetail::Info(self.cheat.description.into())
    }

    fn value(&self) -> &str {
        /*
            State                       Tint        Status

            In queue, turning on        Blue        "Queued On"
            In queue, turning off       Red         "Queued Off"
            Not in queue, on            Green       "On"
            Not in queue, off           None        "Off"
        */

        let will_be_active = self.will_be_active();

        if self.cheat.is_in_queue() {
            if will_be_active {
                "Queued On"
            } else {
                "Queued Off"
            }
        } else if will_be_active {
            "On"
        } else {
            "Off"
        }
    }

    fn tint(&self) -> Option<(u8, u8, u8)> {
        let will_be_active = self.will_be_active();

        if self.cheat.is_in_queue() {
            if will_be_active {
                Some(ui::colours::BLUE)
            } else {
                Some(ui::colours::RED)
            }
        } else if will_be_active {
            Some(ui::colours::GREEN)
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

        if Settings::shared().save_cheats.load(Ordering::SeqCst) {
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
                // Push cheats without codes to the end. If we don't do this, the cheat menu only
                // shows "???" for the first few rows.
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

    if !Settings::shared().save_cheats.load(Ordering::SeqCst) {
        log::info!("Cheat saving/loading is disabled.");
        return;
    }

    log::info!("Loading saved cheats.");

    let path = crate::files::get_documents_path("cleo_saved_cheats.u8");

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
    Cheat::new(0, "THUGSARMOURY", "Weapon set 1"),
    Cheat::new(1, "PROFESSIONALSKIT", "Weapon set 2"),
    Cheat::new(2, "NUTTERSTOYS", "Weapon set 3"),
    Cheat::new(
        3,
        "",
        "Give dildo, minigun and thermal/night-vision goggles",
    ),
    Cheat::new(4, "", "Advance clock by 4 hours"),
    Cheat::new(5, "", "Skip to completion on some missions"),
    Cheat::new(6, "", "Debug (show mappings)"),
    Cheat::new(7, "", "Full invincibility"),
    Cheat::new(8, "", "Debug (show tap to target)"),
    Cheat::new(9, "", "Debug (show targeting)"),
    Cheat::new(10, "INEEDSOMEHELP", "Give health, armour and $250,000"),
    Cheat::new(11, "TURNUPTHEHEAT", "Increase wanted level by two stars"),
    Cheat::new(12, "TURNDOWNTHEHEAT", "Clear wanted level"),
    Cheat::new(13, "PLEASANTLYWARM", "Sunny weather"),
    Cheat::new(14, "TOODAMNHOT", "Very sunny weather"),
    Cheat::new(15, "DULLDULLDAY", "Overcast weather"),
    Cheat::new(16, "STAYINANDWATCHTV", "Rainy weather"),
    Cheat::new(17, "CANTSEEWHEREIMGOING", "Foggy weather"),
    Cheat::new(18, "TIMEJUSTFLIESBY", "Faster time"),
    Cheat::new(19, "SPEEDITUP", "Faster gameplay"),
    Cheat::new(20, "SLOWITDOWN", "Slower gameplay"),
    Cheat::new(
        21,
        "ROUGHNEIGHBOURHOOD",
        "Pedestrians riot, give player golf club",
    ),
    Cheat::new(22, "STOPPICKINGONME", "Pedestrians attack the player"),
    Cheat::new(23, "SURROUNDEDBYNUTTERS", "Give pedestrians weapons"),
    Cheat::new(24, "TIMETOKICKASS", "Spawn Rhino tank"),
    Cheat::new(25, "OLDSPEEDDEMON", "Spawn Bloodring Banger"),
    Cheat::new(26, "", "Spawn stock car"),
    Cheat::new(27, "NOTFORPUBLICROADS", "Spawn Hotring Racer A"),
    Cheat::new(28, "JUSTTRYANDSTOPME", "Spawn Hotring Racer B"),
    Cheat::new(29, "WHERESTHEFUNERAL", "Spawn Romero"),
    Cheat::new(30, "CELEBRITYSTATUS", "Spawn Stretch Limousine"),
    Cheat::new(31, "TRUEGRIME", "Spawn Trashmaster"),
    Cheat::new(32, "18HOLES", "Spawn Caddy"),
    Cheat::new(33, "ALLCARSGOBOOM", "Explode all vehicles"),
    Cheat::new(34, "WHEELSONLYPLEASE", "Invisible cars"),
    Cheat::new(35, "STICKLIKEGLUE", "Improved suspension and handling"),
    Cheat::new(36, "GOODBYECRUELWORLD", "Suicide"),
    Cheat::new(37, "DONTTRYANDSTOPME", "Traffic lights are always green"),
    Cheat::new(
        38,
        "ALLDRIVERSARECRIMINALS",
        "All NPC drivers drive aggressively and have a wanted level",
    ),
    Cheat::new(39, "PINKISTHENEWCOOL", "Pink traffic"),
    Cheat::new(40, "SOLONGASITSBLACK", "Black traffic"),
    Cheat::new(41, "", "Cars have sideways wheels"),
    Cheat::new(42, "FLYINGFISH", "Flying boats"),
    Cheat::new(43, "WHOATEALLTHEPIES", "Maximum fat"),
    Cheat::new(44, "BUFFMEUP", "Maximum muscle"),
    Cheat::new(45, "", "Maximum gambling skill"),
    Cheat::new(46, "LEANANDMEAN", "Minimum fat and muscle"),
    Cheat::new(47, "BLUESUEDESHOES", "All pedestrians are Elvis Presley"),
    Cheat::new(
        48,
        "ATTACKOFTHEVILLAGEPEOPLE",
        "Pedestrians attack the player with guns and rockets",
    ),
    Cheat::new(49, "LIFESABEACH", "Beach party theme"),
    Cheat::new(50, "ONLYHOMIESALLOWED", "Gang wars"),
    Cheat::new(
        51,
        "BETTERSTAYINDOORS",
        "Pedestrians replaced with fighting gang members",
    ),
    Cheat::new(52, "NINJATOWN", "Triad theme"),
    Cheat::new(53, "LOVECONQUERSALL", "Pimp mode"),
    Cheat::new(54, "EVERYONEISPOOR", "Rural traffic"),
    Cheat::new(55, "EVERYONEISRICH", "Sports car traffic"),
    Cheat::new(56, "CHITTYCHITTYBANGBANG", "Flying cars"),
    Cheat::new(57, "CJPHONEHOME", "Very high bunny hops"),
    Cheat::new(58, "JUMPJET", "Spawn Hydra"),
    Cheat::new(59, "IWANTTOHOVER", "Spawn Vortex"),
    Cheat::new(
        60,
        "TOUCHMYCARYOUDIE",
        "Destroy other vehicles on collision",
    ),
    Cheat::new(61, "SPEEDFREAK", "All cars have nitro"),
    Cheat::new(62, "BUBBLECARS", "Cars float away when hit"),
    Cheat::new(63, "NIGHTPROWLER", "Always midnight"),
    Cheat::new(64, "DONTBRINGONTHENIGHT", "Always 9PM"),
    Cheat::new(65, "SCOTTISHSUMMER", "Stormy weather"),
    Cheat::new(66, "SANDINMYEARS", "Sandstorm"),
    Cheat::new(67, "", "Predator?"),
    Cheat::new(68, "KANGAROO", "10x jump height"),
    Cheat::new(69, "NOONECANHURTME", "Infinite health"),
    Cheat::new(70, "MANFROMATLANTIS", "Infinite lung capacity"),
    Cheat::new(71, "LETSGOBASEJUMPING", "Spawn Parachute"),
    Cheat::new(72, "ROCKETMAN", "Spawn Jetpack"),
    Cheat::new(73, "IDOASIPLEASE", "Lock wanted level"),
    Cheat::new(74, "BRINGITON", "Six-star wanted level"),
    Cheat::new(75, "STINGLIKEABEE", "Super punches"),
    Cheat::new(76, "IAMNEVERHUNGRY", "Player never gets hungry"),
    Cheat::new(77, "STATEOFEMERGENCY", "Pedestrians riot"),
    Cheat::new(78, "CRAZYTOWN", "Carnival theme"),
    Cheat::new(79, "TAKEACHILLPILL", "Adrenaline effects"),
    Cheat::new(80, "FULLCLIP", "Everyone has unlimited ammo"),
    Cheat::new(81, "IWANNADRIVEBY", "Full weapon control in vehicles"),
    Cheat::new(82, "GHOSTTOWN", "No pedestrians, reduced live traffic"),
    Cheat::new(83, "HICKSVILLE", "Rural theme"),
    Cheat::new(84, "WANNABEINMYGANG", "Recruit anyone with pistols"),
    Cheat::new(85, "NOONECANSTOPUS", "Recruit anyone with AK-47s"),
    Cheat::new(86, "ROCKETMAYHEM", "Recruit anyone with rocket launchers"),
    Cheat::new(87, "WORSHIPME", "Maximum respect"),
    Cheat::new(88, "HELLOLADIES", "Maximum sex appeal"),
    Cheat::new(89, "ICANGOALLNIGHT", "Maximum stamina"),
    Cheat::new(90, "PROFESSIONALKILLER", "Hitman level for all weapons"),
    Cheat::new(91, "NATURALTALENT", "Maximum vehicle skills"),
    Cheat::new(92, "OHDUDE", "Spawn Hunter"),
    Cheat::new(93, "FOURWHEELFUN", "Spawn Quad"),
    Cheat::new(94, "HITTHEROADJACK", "Spawn Tanker with Tanker Trailer"),
    Cheat::new(95, "ITSALLBULL", "Spawn Dozer"),
    Cheat::new(96, "FLYINGTOSTUNT", "Spawn Stunt Plane"),
    Cheat::new(97, "MONSTERMASH", "Spawn Monster Truck"),
    Cheat::new(98, "", "Prostitutes pay you"),
    Cheat::new(99, "", "Taxis have hydraulics and nitro"),
    Cheat::new(100, "", "CRASHES! Slot cheat 1"),
    Cheat::new(101, "", "CRASHES! Slot cheat 2"),
    Cheat::new(102, "", "CRASHES! Slot cheat 3"),
    Cheat::new(103, "", "CRASHES! Slot cheat 4"),
    Cheat::new(104, "", "CRASHES! Slot cheat 5"),
    Cheat::new(105, "", "CRASHES! Slot cheat 6"),
    Cheat::new(106, "", "CRASHES! Slot cheat 7"),
    Cheat::new(107, "", "CRASHES! Slot cheat 8"),
    Cheat::new(108, "", "CRASHES! Slot cheat 9"),
    Cheat::new(109, "", "CRASHES! Slot cheat 10"),
    Cheat::new(110, "", "Xbox helper"),
];
