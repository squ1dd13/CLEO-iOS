//! Replaces parts of the game's streaming system to allow the loading of replacement files inside IMGs,
//! and also manages the loaded replacements.

// hack: The `stream` module is messy, poorly documented and full of hacky code.
// bug: Opcode 0x04ee seems to break when animations have been swapped.

mod game;
mod load;
mod stream;

pub use load::load_replacements;

pub fn init() {
    stream::hook();
    load::hook();
}
