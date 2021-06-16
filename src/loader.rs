use lazy_static::lazy_static;
use log::{info, trace};
use std::path::PathBuf;
use std::sync::Mutex;
use std::{os::raw::c_char, path::Path};

use crate::call_original;

lazy_static! {
    static ref LOADER_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
}

fn process_path(path_str: &str) -> Option<String> {
    let path = Path::new(path_str);

    if let Some(parent) = path.parent() {
        let extension = parent
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("blah")
            .to_lowercase()
            .to_string();

        // If the file is not a direct child of the .app folder, then we don't mess with it.
        if extension != "app" {
            return None;
        }
    }

    // todo: Should we be case-sensitive here?
    let file_name = path.file_name()?.to_str()?.to_lowercase();

    // See if we have a file that should replace this one.
    let dir_path = LOADER_PATH.lock().ok()?;

    if dir_path.is_none() {
        return None;
    }

    let mut replacement_path = dir_path.as_ref().unwrap().clone();
    replacement_path.push(file_name);
    let replacement_path = replacement_path.as_path();

    if replacement_path.exists() {
        info!(
            "{:?} should replace {:?}",
            replacement_path.display(),
            path.display()
        );

        return Some(replacement_path.as_os_str().to_str().expect("tf?").into());
    }

    None
}

pub fn set_path(buf: &PathBuf) -> bool {
    let mut locked = LOADER_PATH.lock().unwrap();

    if locked.is_none() {
        *locked = Some(buf.clone());
        true
    } else {
        false
    }
}

fn open_file(path: *const c_char, mode: *const c_char) -> u64 {
    let skip = if path.is_null() {
        trace!("Null path!");
        true
    } else if mode.is_null() {
        trace!("Null mode!");
        true
    } else {
        false
    };

    if !skip {
        let path = unsafe { std::ffi::CStr::from_ptr(path) }.to_str().unwrap();
        let mode = unsafe { std::ffi::CStr::from_ptr(mode) }.to_str().unwrap();

        trace!("CFileMgr::OpenFile(\"{}\", \"{}\")", path, mode);
    }

    call_original!(crate::targets::open_file, path, mode)
}

fn sort_out_path(p1: *const c_char, p2: *const c_char, p3: i32) -> bool {
    let r = call_original!(crate::targets::sort_out_path, p1, p2, p3);

    let p1 = unsafe { std::ffi::CStr::from_ptr(p1) }
        .to_str()
        .unwrap_or("invalid");

    let p2 = unsafe { std::ffi::CStr::from_ptr(p2) }
        .to_str()
        .unwrap_or("invalid");

    trace!(
        "sort_out_path finished with ({}, {}, {}) -> {}",
        p1,
        p2,
        p3,
        r
    );

    r
}

fn more_sorting_out(p1: i32, p2: *const c_char, p3: i32) -> *const c_char {
    let r = call_original!(crate::targets::more_sorting_out, p1, p2, p3);

    let ret = unsafe { std::ffi::CStr::from_ptr(r) }.to_str().ok();

    if let Some(output) = ret {
        if let Some(processed) = process_path(output) {
            // let c_str = std::ffi::CString::new(processed).unwrap();
            unsafe {
                let buf = libc::malloc(processed.len() + 1) as *mut c_char;

                for (i, c) in processed.chars().enumerate() {
                    buf.offset(i as isize).write(c as c_char);
                }

                buf.offset(processed.len() as isize).write(0);
                return buf;
            }
            // return c_str.as_ptr() as *const c_char;
        }
    }

    r
}

pub fn hook() {
    crate::targets::open_file::install(open_file);
    crate::targets::sort_out_path::install(sort_out_path);
    crate::targets::more_sorting_out::install(more_sorting_out);
}
