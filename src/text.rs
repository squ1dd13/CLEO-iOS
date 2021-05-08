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

pub fn hook() {
    targets::get_gxt_string::install(get_gxt_string);
}
