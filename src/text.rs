//! Exposes a primitive Rust API for the game's text system, and manages the loading
//! of FXT language files.

use crate::{call_original, targets};
use lazy_static::lazy_static;
use log::warn;
use std::collections::HashMap;
use std::{os::raw::c_char, sync::Mutex};

lazy_static! {
    static ref CUSTOM_STRINGS: Mutex<HashMap<String, Vec<u16>>> = Mutex::new(HashMap::new());
}

fn get_gxt_string(text_obj_ptr: usize, key: *const c_char) -> *const u16 {
    if !key.is_null() {
        let key_str = unsafe { std::ffi::CStr::from_ptr(key) }.to_str();

        if let Ok(key_str) = key_str {
            if let Ok(custom_strings) = CUSTOM_STRINGS.lock() {
                if let Some(value) = custom_strings.get(key_str) {
                    return value.as_ptr();
                }
            }
        }
    }

    return call_original!(targets::get_gxt_string, text_obj_ptr, key);
}

pub fn set_kv(key: &str, value: &str) {
    if let Ok(mut custom_strings) = CUSTOM_STRINGS.lock() {
        let mut utf16: Vec<u16> = value.encode_utf16().collect();

        // The game expects the strings to be null-terminated.
        utf16.push(0);

        if custom_strings.insert(key.into(), utf16).is_some() {
            warn!(
                "Replacing previous value for key '{}' with new value '{}'.",
                key, value
            );
        }
    }
}

fn generate_numberplate(chars: *mut u8, length: i32) -> bool {
    static mut PLATE_TICK: u8 = 0;

    let tick = unsafe {
        PLATE_TICK += 1;

        if PLATE_TICK > 5 {
            PLATE_TICK = 0;
        }

        PLATE_TICK
    };

    if (tick == 3 || tick == 5) && length == 8 {
        // ;)
        const CUSTOM_3: &[u8] = b"CLEO IOS";
        const CUSTOM_5: &[u8] = b"SQUI DDY";

        let custom = if tick == 3 { CUSTOM_3 } else { CUSTOM_5 };

        for (i, c) in custom.iter().enumerate() {
            unsafe {
                chars.add(i).write(*c);
            }
        }

        true
    } else {
        call_original!(targets::gen_plate, chars, length)
    }
}

pub fn load_fxt(path: &impl AsRef<std::path::Path>) -> std::io::Result<()> {
    // todo: Remove the regex so we don't need the crate anymore.
    let comment_pattern: regex::Regex = regex::Regex::new(r"//|#").unwrap();

    for line in std::fs::read_to_string(path.as_ref())?.lines() {
        let line = comment_pattern.split(line).next().map(|s| s.trim());

        if let Some(line) = line {
            if line.is_empty() {
                continue;
            }

            // split_once isn't stable yet, so we have to do this.
            let mut split = line.splitn(2, ' ');
            let (key, value) = (split.next(), split.next());

            if key.is_none() || value.is_none() {
                warn!("Unable to find key and value in line '{}'", line);
                continue;
            }

            set_kv(key.unwrap(), value.unwrap());
        }
    }

    Ok(())
}

pub fn hook() {
    targets::get_gxt_string::install(get_gxt_string);
    targets::gen_plate::install(generate_numberplate);
}
