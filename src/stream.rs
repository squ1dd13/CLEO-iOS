//! Replaces parts of the game's streaming system to allow the loading of replacement files inside IMGs,
//! and also manages the loaded replacements.

// hack: The `stream` module is messy, poorly documented and full of hacky code.
// bug: Opcode 0x04ee seems to break when animations have been swapped.

mod custom;
mod game;
mod load;

pub use load::load_replacement;

pub fn init() {
    custom::hook();
    load::hook();
}
