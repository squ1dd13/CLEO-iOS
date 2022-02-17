use crate::ui::menu::{data, view};
use crossbeam_channel::{Receiver, Sender};
use once_cell::sync::Lazy;
use std::{borrow::Cow, sync::Mutex};

/// Describes the cheat's impact on the game.
#[derive(Clone, Copy)]
enum State {
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

/// An action that is performed to change the way the game runs, either once or with an ongoing
/// effect.
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

impl data::RowData for Cheat {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(match self.code {
            Some(s) => s,
            None => "???",
        })
    }

    fn detail(&self) -> Vec<Cow<'_, str>> {
        vec![Cow::Borrowed(self.desc)]
    }

    fn value(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.state.desc())
    }

    fn tint(&self) -> view::Tint {
        match self.state {
            State::Concrete(true) => view::Tint::Green,
            State::Queued(true) => view::Tint::Blue,
            State::Concrete(false) | State::Queued(false) => view::Tint::White,
        }
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

        // Loop over and process all of the messages we've received.
        loop {
            // We use `try_recv` because it doesn't block when there are no more messages.
            let next = self.receiver.try_recv();

            match next {
                Ok((cheat_index, state)) => {
                    state_balance += if let State::Queued(_) = state { 1 } else { -1 };

                    if state_balance < 0 {
                        log::error!("State balance should never be < 0");
                        state_balance = 0;
                    }

                    self.cheats[cheat_index].state = state;
                }

                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(err) => panic!("try_recv error: {}", err),
            }
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
}

pub fn init() {
    todo!()
}

fn cheats_vec() -> Vec<Cheat> {
    const PROTOTYPES: [(&str, &str); 111] = [
        ("THUGSARMOURY", "Weapon set 1"),
        ("PROFESSIONALSKIT", "Weapon set 2"),
        ("NUTTERSTOYS", "Weapon set 3"),
        ("", "Give dildo, minigun and thermal/night-vision goggles"),
        ("", "Advance clock by 4 hours"),
        ("", "Skip to completion on some missions"),
        ("", "Debug (show mappings)"),
        ("", "Full invincibility"),
        ("", "Debug (show tap to target)"),
        ("", "Debug (show targeting)"),
        ("INEEDSOMEHELP", "Give health, armour and $250,000"),
        ("TURNUPTHEHEAT", "Increase wanted level by two stars"),
        ("TURNDOWNTHEHEAT", "Clear wanted level"),
        ("PLEASANTLYWARM", "Sunny weather"),
        ("TOODAMNHOT", "Very sunny weather"),
        ("DULLDULLDAY", "Overcast weather"),
        ("STAYINANDWATCHTV", "Rainy weather"),
        ("CANTSEEWHEREIMGOING", "Foggy weather"),
        ("TIMEJUSTFLIESBY", "Faster time"),
        ("SPEEDITUP", "Faster gameplay"),
        ("SLOWITDOWN", "Slower gameplay"),
        (
            "ROUGHNEIGHBOURHOOD",
            "Pedestrians riot, give player golf club",
        ),
        ("STOPPICKINGONME", "Pedestrians attack the player"),
        ("SURROUNDEDBYNUTTERS", "Give pedestrians weapons"),
        ("TIMETOKICKASS", "Spawn Rhino tank"),
        ("OLDSPEEDDEMON", "Spawn Bloodring Banger"),
        ("", "Spawn stock car"),
        ("NOTFORPUBLICROADS", "Spawn Hotring Racer A"),
        ("JUSTTRYANDSTOPME", "Spawn Hotring Racer B"),
        ("WHERESTHEFUNERAL", "Spawn Romero"),
        ("CELEBRITYSTATUS", "Spawn Stretch Limousine"),
        ("TRUEGRIME", "Spawn Trashmaster"),
        ("18HOLES", "Spawn Caddy"),
        ("ALLCARSGOBOOM", "Explode all vehicles"),
        ("WHEELSONLYPLEASE", "Invisible cars"),
        ("STICKLIKEGLUE", "Improved suspension and handling"),
        ("GOODBYECRUELWORLD", "Suicide"),
        ("DONTTRYANDSTOPME", "Traffic lights are always green"),
        (
            "ALLDRIVERSARECRIMINALS",
            "All NPC drivers drive aggressively and have a wanted level",
        ),
        ("PINKISTHENEWCOOL", "Pink traffic"),
        ("SOLONGASITSBLACK", "Black traffic"),
        ("", "Cars have sideways wheels"),
        ("FLYINGFISH", "Flying boats"),
        ("WHOATEALLTHEPIES", "Maximum fat"),
        ("BUFFMEUP", "Maximum muscle"),
        ("", "Maximum gambling skill"),
        ("LEANANDMEAN", "Minimum fat and muscle"),
        ("BLUESUEDESHOES", "All pedestrians are Elvis Presley"),
        (
            "ATTACKOFTHEVILLAGEPEOPLE",
            "Pedestrians attack the player with guns and rockets",
        ),
        ("LIFESABEACH", "Beach party theme"),
        ("ONLYHOMIESALLOWED", "Gang wars"),
        (
            "BETTERSTAYINDOORS",
            "Pedestrians replaced with fighting gang members",
        ),
        ("NINJATOWN", "Triad theme"),
        ("LOVECONQUERSALL", "Pimp mode"),
        ("EVERYONEISPOOR", "Rural traffic"),
        ("EVERYONEISRICH", "Sports car traffic"),
        ("CHITTYCHITTYBANGBANG", "Flying cars"),
        ("CJPHONEHOME", "Very high bunny hops"),
        ("JUMPJET", "Spawn Hydra"),
        ("IWANTTOHOVER", "Spawn Vortex"),
        ("TOUCHMYCARYOUDIE", "Destroy other vehicles on collision"),
        ("SPEEDFREAK", "All cars have nitro"),
        ("BUBBLECARS", "Cars float away when hit"),
        ("NIGHTPROWLER", "Always midnight"),
        ("DONTBRINGONTHENIGHT", "Always 9PM"),
        ("SCOTTISHSUMMER", "Stormy weather"),
        ("SANDINMYEARS", "Sandstorm"),
        ("", "Predator?"),
        ("KANGAROO", "10x jump height"),
        ("NOONECANHURTME", "Infinite health"),
        ("MANFROMATLANTIS", "Infinite lung capacity"),
        ("LETSGOBASEJUMPING", "Spawn Parachute"),
        ("ROCKETMAN", "Spawn Jetpack"),
        ("IDOASIPLEASE", "Lock wanted level"),
        ("BRINGITON", "Six-star wanted level"),
        ("STINGLIKEABEE", "Super punches"),
        ("IAMNEVERHUNGRY", "Player never gets hungry"),
        ("STATEOFEMERGENCY", "Pedestrians riot"),
        ("CRAZYTOWN", "Carnival theme"),
        ("TAKEACHILLPILL", "Adrenaline effects"),
        ("FULLCLIP", "Everyone has unlimited ammo"),
        ("IWANNADRIVEBY", "Full weapon control in vehicles"),
        ("GHOSTTOWN", "No pedestrians, reduced live traffic"),
        ("HICKSVILLE", "Rural theme"),
        ("WANNABEINMYGANG", "Recruit anyone with pistols"),
        ("NOONECANSTOPUS", "Recruit anyone with AK-47s"),
        ("ROCKETMAYHEM", "Recruit anyone with rocket launchers"),
        ("WORSHIPME", "Maximum respect"),
        ("HELLOLADIES", "Maximum sex appeal"),
        ("ICANGOALLNIGHT", "Maximum stamina"),
        ("PROFESSIONALKILLER", "Hitman level for all weapons"),
        ("NATURALTALENT", "Maximum vehicle skills"),
        ("OHDUDE", "Spawn Hunter"),
        ("FOURWHEELFUN", "Spawn Quad"),
        ("HITTHEROADJACK", "Spawn Tanker with Tanker Trailer"),
        ("ITSALLBULL", "Spawn Dozer"),
        ("FLYINGTOSTUNT", "Spawn Stunt Plane"),
        ("MONSTERMASH", "Spawn Monster Truck"),
        ("", "Prostitutes pay you"),
        ("", "Taxis have hydraulics and nitro"),
        ("", "CRASHES! Slot cheat 1"),
        ("", "CRASHES! Slot cheat 2"),
        ("", "CRASHES! Slot cheat 3"),
        ("", "CRASHES! Slot cheat 4"),
        ("", "CRASHES! Slot cheat 5"),
        ("", "CRASHES! Slot cheat 6"),
        ("", "CRASHES! Slot cheat 7"),
        ("", "CRASHES! Slot cheat 8"),
        ("", "CRASHES! Slot cheat 9"),
        ("", "CRASHES! Slot cheat 10"),
        ("", "Xbox helper"),
    ];

    PROTOTYPES
        .iter()
        .enumerate()
        .map(|(index, (code, desc))| Cheat {
            index,
            code: if code.is_empty() { None } else { Some(code) },
            desc,
            state: State::Concrete(false),
        })
        .collect()
}
