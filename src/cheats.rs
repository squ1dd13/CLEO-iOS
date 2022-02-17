use crate::ui::menu::{data, view};
use crossbeam_channel::{Receiver, Sender};
use once_cell::sync::Lazy;
use std::{borrow::Cow, sync::Mutex};

/// Describes the cheat's impact on the game.
#[derive(Clone, Copy)]
pub enum State {
    /// The cheat is either on or off.
    Concrete(bool),

    /// The cheat will change state on the next update.
    Queued(bool),
}

impl State {
    /// Returns a description of the state that can be shown to the user.
    fn desc(self) -> &'static str {
        match self {
            State::Concrete(true) => "On",
            State::Queued(true) => "Queued",

            // We hide the queueing mechanic for disabling cheats from the user. As far as they are
            // concerned, a cheat turns off as soon as they tell it to.
            State::Concrete(false) | State::Queued(false) => "Off",
        }
    }

    /// Returns the state that this one can be toggled to.
    fn opposite(self) -> State {
        match self {
            // Concrete states can be toggled to queued states with the opposite value, as the
            // queued state represents a transition to the opposite concrete state.
            State::Concrete(v) => State::Queued(!v),

            // Queued states turn back into whatever they were before they changed, which can only
            // be a concrete state.
            State::Queued(v) => State::Concrete(!v),
        }
    }
}

/// The messages that are sent from the rows in the menu to the cheat manager.
/// Format: (cheat index, new state)
pub type StateUpdate = (usize, State);

/// An action that is performed to change the way the game runs, either once or with an ongoing
/// effect.
#[derive(Clone, Copy)]
struct Cheat {
    /// The number of the cheat in arrays in game code. We need this to communicate with the game
    /// about this specific cheat.
    index: usize,

    /// The cheat code. Some cheats don't have codes because they were never intended for players
    /// to use, so will have `None` here.
    code: Option<&'static str>,

    /// A description of the cheat. This should include any warnings about the cheat's stability.
    desc: &'static str,

    /// How and when the cheat will affect the game.
    state: State,

    /// Whether the cheat is known to crash the game.
    crashes: bool,
}

impl Cheat {
    /// Runs the cheat if the user has requested to change its state.
    fn update(&mut self) {
        // Skip the update if the state is not changing.
        if let State::Concrete(_) = self.state {
            return;
        }

        let bool_state = self.bool_state();

        // Cheats with functions handle their own states. We just need to call the function.
        if let Some(func) = self.function() {
            // Call the game's implementation of the cheat.
            func();

            // Set a concrete state now that we know the outcome of the cheat function.
            self.state = State::Concrete(*bool_state);

            return;
        }

        // We have to set the state ourselves because there is no function to call.
        *bool_state = match self.state {
            State::Concrete(v) | State::Queued(v) => v,
        };

        self.state = State::Concrete(*bool_state);
    }

    /// Returns a mutable reference to the boolean value that the game uses to keep track of
    /// whether this cheat is enabled or not.
    fn bool_state(&self) -> &'static mut bool {
        todo!()
    }

    /// If this cheat has a function associated with it, returns the pointer to that function. If
    /// there is no function, `None` is returned.
    fn function(&self) -> Option<fn()> {
        todo!()
    }
}

impl data::RowData<StateUpdate> for Cheat {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(match self.code {
            Some(s) => s,
            None => "???",
        })
    }

    fn detail(&self) -> Vec<Cow<'_, str>> {
        if self.crashes {
            vec![
                Cow::Borrowed(self.desc),
                Cow::Borrowed("This cheat crashes the game."),
            ]
        } else {
            vec![Cow::Borrowed(self.desc)]
        }
    }

    fn value(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.state.desc())
    }

    fn tint(&self) -> view::Tint {
        match self.state {
            _ if self.crashes => view::Tint::Orange,
            State::Concrete(true) => view::Tint::Green,
            State::Queued(true) => view::Tint::Blue,
            State::Concrete(false) | State::Queued(false) => view::Tint::White,
        }
    }

    fn tap_msg(&mut self) -> Option<StateUpdate> {
        // Update the state here so that we can show the user.
        self.state = self.state.opposite();

        // Send the new state to the cheat manager so it can be applied to the real cheat when the
        // game resumes.
        Some((self.index, self.state))
    }
}

/// A structure that holds and manages access to all of the cheats we control.
struct Manager {
    /// The cheats themselves.
    cheats: Vec<Cheat>,

    /// A sender that we can clone to give to the menu. The menu can then report back cheat state
    /// changes.
    sender: Sender<(usize, State)>,

    /// The receiver that we use to hold and process menu events.
    receiver: Receiver<(usize, State)>,
}

impl Manager {
    /// Returns a mutable reference to the shared cheat manager. This will create the manager if it
    /// doesn't already exist.
    fn shared_mut<'mgr>() -> std::sync::MutexGuard<'mgr, Manager> {
        static SHARED: Lazy<Mutex<Manager>> = Lazy::new(|| {
            let (sender, receiver) = crossbeam_channel::unbounded();

            Mutex::new(Manager {
                cheats: cheats_vec(),
                sender,
                receiver,
            })
        });

        SHARED.lock().unwrap()
    }

    /// Updates all of the cheats that require state changes.
    fn update(&mut self) {
        // The number of queued states that have been set minus the number of concrete states. If
        // this is > 0 after the update, a cheat state has changed and we need to save.
        let mut state_balance = 0i32;

        // Iterate over and process all of the messages we've received.
        for (cheat_index, state) in self.receiver.try_iter() {
            state_balance += if let State::Queued(_) = state { 1 } else { -1 };

            if state_balance < 0 {
                log::error!("State balance should never be < 0");
                state_balance = 0;
            }

            self.cheats[cheat_index].state = state;
        }

        // Update the cheats *before* saving their states so that we don't save the states of
        // functional cheats. We also won't save any cheats that crash the game immediately.
        for cheat in &mut self.cheats {
            // `update` does nothing if there is no state change.
            cheat.update();
        }

        if state_balance > 0 {
            log::info!("State balance > 0, so cheat states need saving.");
        } else {
            log::info!("No overall changes to cheat states, so no save required.");
        }
    }

    /// Disables all of the cheats.
    fn reset(&mut self) {
        while self.receiver.try_recv().is_ok() {
            log::info!("Ignoring state message during reset.");
        }

        for cheat in &mut self.cheats {
            cheat.state = State::Concrete(false);
        }
    }

    fn tab_data(&self) -> data::TabData<'static, StateUpdate, Cheat> {
        data::TabData {
            title: Cow::Borrowed("Cheats"),

            // Show a message to warn the user of the dangers of messing with cheats.
            message: Some(data::TabMsg {
                text: Cow::Borrowed(
                    r"Some cheats (or combinations of cheats) may crash the game or corrupt your save file.
Make sure you back up your save if you don't want to risk losing your progress.",
                ),

                // This is a warning, so tint it orange.
                tint: view::Tint::Orange,
            }),

            // The rows are actually just cloned cheat structures that send updates back to us so
            // we can modify the real cheats.
            rows: self.cheats.clone(),

            sender: self.sender.clone(),
        }
    }
}

// fixme: We shouldn't need to expose `StateUpdate` here. Find a better way to do this.
pub fn tab_data() -> data::TabData<'static, StateUpdate, impl data::RowData<StateUpdate>> {
    Manager::shared_mut().tab_data()
}

pub fn init() {
    // Hook the function that updates the cheat system. This is called once every tick.
    crate::targets::do_cheats::install(|| {
        Manager::shared_mut().update();
    });

    // Hook the function that resets the cheat states between games.
    crate::targets::reset_cheats::install(|| {
        log::info!("Resetting cheats");
        crate::call_original!(crate::targets::reset_cheats);
        Manager::shared_mut().reset();
    });
}

fn cheats_vec() -> Vec<Cheat> {
    // Pages I found useful for writing this list:
    //  https://docs.google.com/spreadsheets/d/1-rmga12W9reALga7fct22tJ-1thxbbsfGiGltK2qgh0/edit
    //  https://gta.fandom.com/wiki/Cheats_in_GTA_San_Andreas
    const PROTOTYPES: [(&str, &str, bool); 111] = [
        ("THUGSARMOURY", "Weapon set 1", false),
        ("PROFESSIONALSKIT", "Weapon set 2", false),
        ("NUTTERSTOYS", "Weapon set 3", false),
        (
            "",
            "Give dildo, minigun and thermal/night-vision goggles",
            false,
        ),
        ("", "Advance clock by 4 hours", false),
        ("", "Skip to completion on some missions", false),
        ("", "Debug (show mappings)", false),
        ("", "Full invincibility", false),
        ("", "Debug (show tap to target)", false),
        ("", "Debug (show targeting)", false),
        ("INEEDSOMEHELP", "Give health, armour and $250,000", false),
        ("TURNUPTHEHEAT", "Increase wanted level by two stars", false),
        ("TURNDOWNTHEHEAT", "Clear wanted level", false),
        ("PLEASANTLYWARM", "Sunny weather", false),
        ("TOODAMNHOT", "Very sunny weather", false),
        ("DULLDULLDAY", "Overcast weather", false),
        ("STAYINANDWATCHTV", "Rainy weather", false),
        ("CANTSEEWHEREIMGOING", "Foggy weather", false),
        ("TIMEJUSTFLIESBY", "Faster time", false),
        ("SPEEDITUP", "Faster gameplay", false),
        ("SLOWITDOWN", "Slower gameplay", false),
        (
            "ROUGHNEIGHBOURHOOD",
            "Pedestrians riot, give player golf club",
            false,
        ),
        ("STOPPICKINGONME", "Pedestrians attack the player", false),
        ("SURROUNDEDBYNUTTERS", "Give pedestrians weapons", false),
        ("TIMETOKICKASS", "Spawn Rhino tank", false),
        ("OLDSPEEDDEMON", "Spawn Bloodring Banger", false),
        ("", "Spawn stock car", false),
        ("NOTFORPUBLICROADS", "Spawn Hotring Racer A", false),
        ("JUSTTRYANDSTOPME", "Spawn Hotring Racer B", false),
        ("WHERESTHEFUNERAL", "Spawn Romero", false),
        ("CELEBRITYSTATUS", "Spawn Stretch Limousine", false),
        ("TRUEGRIME", "Spawn Trashmaster", false),
        ("18HOLES", "Spawn Caddy", false),
        ("ALLCARSGOBOOM", "Explode all vehicles", false),
        ("WHEELSONLYPLEASE", "Invisible cars", false),
        ("STICKLIKEGLUE", "Improved suspension and handling", false),
        ("GOODBYECRUELWORLD", "Suicide", false),
        ("DONTTRYANDSTOPME", "Traffic lights are always green", false),
        (
            "ALLDRIVERSARECRIMINALS",
            "All NPC drivers drive aggressively and have a wanted level",
            false,
        ),
        ("PINKISTHENEWCOOL", "Pink traffic", false),
        ("SOLONGASITSBLACK", "Black traffic", false),
        ("", "Cars have sideways wheels", false),
        ("FLYINGFISH", "Flying boats", false),
        ("WHOATEALLTHEPIES", "Maximum fat", false),
        ("BUFFMEUP", "Maximum muscle", false),
        ("", "Maximum gambling skill", false),
        ("LEANANDMEAN", "Minimum fat and muscle", false),
        ("BLUESUEDESHOES", "All pedestrians are Elvis Presley", false),
        (
            "ATTACKOFTHEVILLAGEPEOPLE",
            "Pedestrians attack the player with guns and rockets",
            false,
        ),
        ("LIFESABEACH", "Beach party theme", false),
        ("ONLYHOMIESALLOWED", "Gang wars", false),
        (
            "BETTERSTAYINDOORS",
            "Pedestrians replaced with fighting gang members",
            false,
        ),
        ("NINJATOWN", "Triad theme", false),
        ("LOVECONQUERSALL", "Pimp mode", false),
        ("EVERYONEISPOOR", "Rural traffic", false),
        ("EVERYONEISRICH", "Sports car traffic", false),
        ("CHITTYCHITTYBANGBANG", "Flying cars", false),
        ("CJPHONEHOME", "Very high bunny hops", false),
        ("JUMPJET", "Spawn Hydra", false),
        ("IWANTTOHOVER", "Spawn Vortex", false),
        (
            "TOUCHMYCARYOUDIE",
            "Destroy other vehicles on collision",
            false,
        ),
        ("SPEEDFREAK", "All cars have nitro", false),
        ("BUBBLECARS", "Cars float away when hit", false),
        ("NIGHTPROWLER", "Always midnight", false),
        ("DONTBRINGONTHENIGHT", "Always 9PM", false),
        ("SCOTTISHSUMMER", "Stormy weather", false),
        ("SANDINMYEARS", "Sandstorm", false),
        ("", "Predator?", false),
        ("KANGAROO", "10x jump height", false),
        ("NOONECANHURTME", "Infinite health", false),
        ("MANFROMATLANTIS", "Infinite lung capacity", false),
        ("LETSGOBASEJUMPING", "Spawn Parachute", false),
        ("ROCKETMAN", "Spawn Jetpack", false),
        ("IDOASIPLEASE", "Lock wanted level", false),
        ("BRINGITON", "Six-star wanted level", false),
        ("STINGLIKEABEE", "Super punches", false),
        ("IAMNEVERHUNGRY", "Player never gets hungry", false),
        ("STATEOFEMERGENCY", "Pedestrians riot", false),
        ("CRAZYTOWN", "Carnival theme", false),
        ("TAKEACHILLPILL", "Adrenaline effects", false),
        ("FULLCLIP", "Everyone has unlimited ammo", false),
        ("IWANNADRIVEBY", "Full weapon control in vehicles", false),
        ("GHOSTTOWN", "No pedestrians, reduced live traffic", false),
        ("HICKSVILLE", "Rural theme", false),
        ("WANNABEINMYGANG", "Recruit anyone with pistols", false),
        ("NOONECANSTOPUS", "Recruit anyone with AK-47s", false),
        (
            "ROCKETMAYHEM",
            "Recruit anyone with rocket launchers",
            false,
        ),
        ("WORSHIPME", "Maximum respect", false),
        ("HELLOLADIES", "Maximum sex appeal", false),
        ("ICANGOALLNIGHT", "Maximum stamina", false),
        ("PROFESSIONALKILLER", "Hitman level for all weapons", false),
        ("NATURALTALENT", "Maximum vehicle skills", false),
        ("OHDUDE", "Spawn Hunter", false),
        ("FOURWHEELFUN", "Spawn Quad", false),
        ("HITTHEROADJACK", "Spawn Tanker with Tanker Trailer", false),
        ("ITSALLBULL", "Spawn Dozer", false),
        ("FLYINGTOSTUNT", "Spawn Stunt Plane", false),
        ("MONSTERMASH", "Spawn Monster Truck", false),
        ("", "Prostitutes pay you", false),
        ("", "Taxis have hydraulics and nitro", false),
        ("", "Slot cheat 1", true),
        ("", "Slot cheat 2", true),
        ("", "Slot cheat 3", true),
        ("", "Slot cheat 4", true),
        ("", "Slot cheat 5", true),
        ("", "Slot cheat 6", true),
        ("", "Slot cheat 7", true),
        ("", "Slot cheat 8", true),
        ("", "Slot cheat 9", true),
        ("", "Slot cheat 10", true),
        ("", "Xbox helper", false),
    ];

    PROTOTYPES
        .iter()
        .enumerate()
        .map(|(index, &(code, desc, crashes))| Cheat {
            index,
            code: if code.is_empty() { None } else { Some(code) },
            desc,
            state: State::Concrete(false),
            crashes,
        })
        .collect()
}
