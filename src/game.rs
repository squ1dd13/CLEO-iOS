//! Code for game systems.

pub mod cheats;
mod controller;
mod extras;
pub mod loader;
pub mod scripts;
mod sound;
pub mod streaming;
pub mod text;

pub fn init() {
    streaming::init();
    loader::init();
    text::init();
    extras::init();
    scripts::init();
    cheats::init();
    controller::init();
}
