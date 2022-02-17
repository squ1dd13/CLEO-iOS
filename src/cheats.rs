use std::borrow::Cow;
use crate::ui::menu::{data, view};

/// Describes the cheat's impact on the game.
#[derive(Clone, Copy)]
enum State {
    /// The cheat is either on or off.
    Concrete(bool),

    /// The cheat is *going to be* on or off, but is currently transitioning.
    Queued(bool),
}

impl State {
    fn desc(self) -> &'static str {
        match self {
            State::Concrete(true) => "On",
            State::Concrete(false) => "Off",
            State::Queued(true) => "Queued On",
            State::Queued(false) => "Queued Off",
        }
    }
}

/// An action that is performed to change the way the game runs, either once or with an ongoing
/// effect.
struct Cheat {
    /// The number of the cheat in arrays in game code. We need this to communicate with the game
    /// about this specific cheat.
    index: usize,

    /// The cheat code. Some cheats don't have codes because they were never intended for players to
    /// use, so will have `None` here.
    code: Option<&'static str>,

    /// A description of the cheat. This should include any warnings about the script's stability.
    desc: &'static str,

    /// How and when the cheat will affect the game.
    state: State,
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
        // todo: Consider using red/amber/green to imitate traffic lights.
        match self.state {
            State::Concrete(true) => view::Tint::Green,
            State::Concrete(false) => view::Tint::White,
            State::Queued(true) => view::Tint::Blue,
            State::Queued(false) => view::Tint::Red,
        }
    }
}

pub fn init() {
    todo!()
}