use std::{borrow::Cow, sync::Mutex};

use crossbeam_channel::{Receiver, Sender};
use once_cell::sync::Lazy;

use crate::ui::menu::{data, view};

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

/// Whether a cheat is known to cause game crashes.
#[derive(Clone, Copy)]
enum Stability {
    /// The cheat is considered safe. No warning will be shown in the menu.
    Stable,

    /// The cheat is likely to crash the game. The user will be warned.
    Crashes,
}

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
    stability: Stability,
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
        match self.stability {
            Stability::Stable => vec![Cow::Borrowed(self.desc)],
            Stability::Crashes => vec![
                Cow::Borrowed(self.desc),
                Cow::Borrowed("This cheat crashes the game."),
            ],
        }
    }

    fn value(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.state.desc())
    }

    fn tint(&self) -> view::Tint {
        match self.state {
            State::Concrete(true) => view::Tint::Green,
            State::Queued(true) => view::Tint::Blue,

            State::Concrete(false) | State::Queued(false) => match self.stability {
                Stability::Stable => view::Tint::White,

                // Only show the orange tint on cheats that are disabled. All cheats start
                // disabled (unless saved enabled), so the user will see this tint at least once
                // before they decide to enable the cheat anyway.
                Stability::Crashes => view::Tint::Orange,
            },
        }
    }

    fn tap_msg(&mut self) -> Option<StateUpdate> {
        // Update the state here so that we can show the user.
        self.state = self.state.opposite();

        // Send the new state to the cheat manager so that it can be applied to the real cheat
        // when the game resumes.
        Some((self.index, self.state))
    }
}

/// A structure that holds and manages access to all of the cheats we control.
struct Manager {
    /// The cheats themselves.
    cheats: Vec<Cheat>,

    /// A sender that we can clone to give to the menu. The menu can then report back cheat state
    /// changes.
    sender: Sender<StateUpdate>,

    /// The receiver that we use to hold and process menu events.
    receiver: Receiver<StateUpdate>,
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
        // this is > 0 after the update, a cheat state has changed, and we need to save.
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

    /// Creates the tab data for allowing the user to interact with the cheat system from the menu.
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
            // that we can modify the real cheats.
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
    crate::declare_hook!(
        /// Updates the cheat system. Called every frame. We completely replace this with our own
        /// implementation because we re-implement the cheat system.
        DO_CHEATS,
        fn(),
        0x1001a7f28
    );

    // Hook the function that updates the cheat system. This is called once every tick.
    DO_CHEATS.install(|| {
        Manager::shared_mut().update();
    });

    crate::declare_hook!(
        /// Resets the cheat system, ready for a new load.
        RESET_CHEATS,
        fn(),
        0x1001a8194
    );

    // Hook the function that resets the cheat states between games.
    RESET_CHEATS.install(|| {
        log::info!("Resetting cheats");
        RESET_CHEATS.original()();
        Manager::shared_mut().reset();
    });
}

fn cheats_vec() -> Vec<Cheat> {
    use Stability::*;

    // Pages I found useful for writing this list:
    //  https://docs.google.com/spreadsheets/d/1-rmga12W9reALga7fct22tJ-1thxbbsfGiGltK2qgh0/edit
    //  https://gta.fandom.com/wiki/Cheats_in_GTA_San_Andreas
    const PROTOTYPES: [(&str, &str, Stability); 111] = [
        ("THUGSARMOURY", "Weapon set 1", Stable),
        ("PROFESSIONALSKIT", "Weapon set 2", Stable),
        ("NUTTERSTOYS", "Weapon set 3", Stable),
        (
            "",
            "Give dildo, minigun and thermal/night-vision goggles",
            Stable,
        ),
        ("", "Advance clock by 4 hours", Stable),
        ("", "Skip to completion on some missions", Stable),
        ("", "Debug (show mappings)", Stable),
        ("", "Full invincibility", Stable),
        ("", "Debug (show tap to target)", Stable),
        ("", "Debug (show targeting)", Stable),
        ("INEEDSOMEHELP", "Give health, armour and $250,000", Stable),
        (
            "TURNUPTHEHEAT",
            "Increase wanted level by two stars",
            Stable,
        ),
        ("TURNDOWNTHEHEAT", "Clear wanted level", Stable),
        ("PLEASANTLYWARM", "Sunny weather", Stable),
        ("TOODAMNHOT", "Very sunny weather", Stable),
        ("DULLDULLDAY", "Overcast weather", Stable),
        ("STAYINANDWATCHTV", "Rainy weather", Stable),
        ("CANTSEEWHEREIMGOING", "Foggy weather", Stable),
        ("TIMEJUSTFLIESBY", "Faster time", Stable),
        ("SPEEDITUP", "Faster gameplay", Stable),
        ("SLOWITDOWN", "Slower gameplay", Stable),
        (
            "ROUGHNEIGHBOURHOOD",
            "Pedestrians riot, give player golf club",
            Stable,
        ),
        ("STOPPICKINGONME", "Pedestrians attack the player", Stable),
        ("SURROUNDEDBYNUTTERS", "Give pedestrians weapons", Stable),
        ("TIMETOKICKASS", "Spawn Rhino tank", Stable),
        ("OLDSPEEDDEMON", "Spawn Bloodring Banger", Stable),
        ("", "Spawn stock car", Stable),
        ("NOTFORPUBLICROADS", "Spawn Hotring Racer A", Stable),
        ("JUSTTRYANDSTOPME", "Spawn Hotring Racer B", Stable),
        ("WHERESTHEFUNERAL", "Spawn Romero", Stable),
        ("CELEBRITYSTATUS", "Spawn Stretch Limousine", Stable),
        ("TRUEGRIME", "Spawn Trashmaster", Stable),
        ("18HOLES", "Spawn Caddy", Stable),
        ("ALLCARSGOBOOM", "Explode all vehicles", Stable),
        ("WHEELSONLYPLEASE", "Invisible cars", Stable),
        ("STICKLIKEGLUE", "Improved suspension and handling", Stable),
        ("GOODBYECRUELWORLD", "Suicide", Stable),
        (
            "DONTTRYANDSTOPME",
            "Traffic lights are always green",
            Stable,
        ),
        (
            "ALLDRIVERSARECRIMINALS",
            "All NPC drivers drive aggressively and have a wanted level",
            Stable,
        ),
        ("PINKISTHENEWCOOL", "Pink traffic", Stable),
        ("SOLONGASITSBLACK", "Black traffic", Stable),
        ("", "Cars have sideways wheels", Stable),
        ("FLYINGFISH", "Flying boats", Stable),
        ("WHOATEALLTHEPIES", "Maximum fat", Stable),
        ("BUFFMEUP", "Maximum muscle", Stable),
        ("", "Maximum gambling skill", Stable),
        ("LEANANDMEAN", "Minimum fat and muscle", Stable),
        (
            "BLUESUEDESHOES",
            "All pedestrians are Elvis Presley",
            Stable,
        ),
        (
            "ATTACKOFTHEVILLAGEPEOPLE",
            "Pedestrians attack the player with guns and rockets",
            Stable,
        ),
        ("LIFESABEACH", "Beach party theme", Stable),
        ("ONLYHOMIESALLOWED", "Gang wars", Stable),
        (
            "BETTERSTAYINDOORS",
            "Pedestrians replaced with fighting gang members",
            Stable,
        ),
        ("NINJATOWN", "Triad theme", Stable),
        ("LOVECONQUERSALL", "Pimp mode", Stable),
        ("EVERYONEISPOOR", "Rural traffic", Stable),
        ("EVERYONEISRICH", "Sports car traffic", Stable),
        ("CHITTYCHITTYBANGBANG", "Flying cars", Stable),
        ("CJPHONEHOME", "Very high bunny hops", Stable),
        ("JUMPJET", "Spawn Hydra", Stable),
        ("IWANTTOHOVER", "Spawn Vortex", Stable),
        (
            "TOUCHMYCARYOUDIE",
            "Destroy other vehicles on collision",
            Stable,
        ),
        ("SPEEDFREAK", "All cars have nitro", Stable),
        ("BUBBLECARS", "Cars float away when hit", Stable),
        ("NIGHTPROWLER", "Always midnight", Stable),
        ("DONTBRINGONTHENIGHT", "Always 9PM", Stable),
        ("SCOTTISHSUMMER", "Stormy weather", Stable),
        ("SANDINMYEARS", "Sandstorm", Stable),
        ("", "Predator?", Stable),
        ("KANGAROO", "10x jump height", Stable),
        ("NOONECANHURTME", "Infinite health", Stable),
        ("MANFROMATLANTIS", "Infinite lung capacity", Stable),
        ("LETSGOBASEJUMPING", "Spawn Parachute", Stable),
        ("ROCKETMAN", "Spawn Jetpack", Stable),
        ("IDOASIPLEASE", "Lock wanted level", Stable),
        ("BRINGITON", "Six-star wanted level", Stable),
        ("STINGLIKEABEE", "Super punches", Stable),
        ("IAMNEVERHUNGRY", "Player never gets hungry", Stable),
        ("STATEOFEMERGENCY", "Pedestrians riot", Stable),
        ("CRAZYTOWN", "Carnival theme", Stable),
        ("TAKEACHILLPILL", "Adrenaline effects", Stable),
        ("FULLCLIP", "Everyone has unlimited ammo", Stable),
        ("IWANNADRIVEBY", "Full weapon control in vehicles", Stable),
        ("GHOSTTOWN", "No pedestrians, reduced live traffic", Stable),
        ("HICKSVILLE", "Rural theme", Stable),
        ("WANNABEINMYGANG", "Recruit anyone with pistols", Stable),
        ("NOONECANSTOPUS", "Recruit anyone with AK-47s", Stable),
        (
            "ROCKETMAYHEM",
            "Recruit anyone with rocket launchers",
            Stable,
        ),
        ("WORSHIPME", "Maximum respect", Stable),
        ("HELLOLADIES", "Maximum sex appeal", Stable),
        ("ICANGOALLNIGHT", "Maximum stamina", Stable),
        ("PROFESSIONALKILLER", "Hitman level for all weapons", Stable),
        ("NATURALTALENT", "Maximum vehicle skills", Stable),
        ("OHDUDE", "Spawn Hunter", Stable),
        ("FOURWHEELFUN", "Spawn Quad", Stable),
        ("HITTHEROADJACK", "Spawn Tanker with Tanker Trailer", Stable),
        ("ITSALLBULL", "Spawn Dozer", Stable),
        ("FLYINGTOSTUNT", "Spawn Stunt Plane", Stable),
        ("MONSTERMASH", "Spawn Monster Truck", Stable),
        ("", "Prostitutes pay you", Stable),
        ("", "Taxis have hydraulics and nitro", Stable),
        ("", "Slot cheat 1", Crashes),
        ("", "Slot cheat 2", Crashes),
        ("", "Slot cheat 3", Crashes),
        ("", "Slot cheat 4", Crashes),
        ("", "Slot cheat 5", Crashes),
        ("", "Slot cheat 6", Crashes),
        ("", "Slot cheat 7", Crashes),
        ("", "Slot cheat 8", Crashes),
        ("", "Slot cheat 9", Crashes),
        ("", "Slot cheat 10", Crashes),
        ("", "Xbox helper", Stable),
    ];

    PROTOTYPES
        .iter()
        .enumerate()
        .map(|(index, &(code, desc, stability))| Cheat {
            index,
            code: if code.is_empty() { None } else { Some(code) },
            desc,
            state: State::Concrete(false),
            stability,
        })
        .collect()
}
