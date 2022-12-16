//! Replaces parts of the game's streaming system to allow the loading of replacement files inside IMGs,
//! and also manages the loaded replacements.

// hack: The `stream` module is messy, poorly documented and full of hacky code.
// bug: Opcode 0x04ee seems to break when animations have been swapped.

use std::{
    collections::HashMap,
    ffi::CStr,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::Mutex,
};

use byteorder::{LittleEndian, ReadBytesExt};
use eyre::Context;
use itertools::Itertools;
use libc::c_char;

mod custom;
mod game;
mod load;

use crate::{call_original, hook, targets};

pub use load::load_replacement;

pub fn init() {
    custom::hook();
    load::hook();
}
