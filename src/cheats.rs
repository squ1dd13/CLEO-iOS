use crate::ui::menu::{data, view};
use std::borrow::Cow;

/// Describes the cheat's impact on the game.
#[derive(Clone, Copy, PartialEq, Eq)]
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

pub fn init() {
    todo!()
}
