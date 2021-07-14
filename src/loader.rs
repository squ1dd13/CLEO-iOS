//! Provides CLEO's file swapping functionality.

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref PATH_SWAPS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

fn swap_path(game_path: &str) -> String {
    if let Some(swapped) = PATH_SWAPS.lock().unwrap().get(game_path) {
        swapped.clone()
    } else {
        game_path.to_string()
    }
}

pub fn get_game_path() -> Option<std::path::PathBuf> {
    Some(std::env::current_exe().ok()?.parent()?.to_path_buf())
}

fn path_in_game_dir(path: &impl AsRef<std::path::Path>) -> Option<std::path::PathBuf> {
    let name = path.as_ref().file_name()?.to_str()?.to_string();

    let mut path = get_game_path()?;
    path.push(name);

    Some(path)
}

// Used when we need to open a game file that could potentially have been swapped.
pub fn find_absolute_path(path: &impl AsRef<str>) -> Option<String> {
    let path = path.as_ref().replace('\\', "/");
    Some(swap_path(
        path_in_game_dir(&path)?.display().to_string().as_str(),
    ))
}

fn find_absolute_path_c(p1: i32, p2: *const u8, p3: i32) -> *const u8 {
    let c_path = crate::call_original!(crate::targets::find_absolute_path, p1, p2, p3);
    let resolved_path = unsafe { std::ffi::CStr::from_ptr(c_path.cast()) }
        .to_str()
        .ok();

    if let Some(resolved_path) = resolved_path {
        let final_path = swap_path(resolved_path);

        unsafe {
            let buf = libc::malloc(final_path.len() + 1) as *mut u8;

            for (i, c) in final_path.chars().enumerate() {
                buf.add(i).write(c as u8);
            }

            // Null terminator.
            buf.add(final_path.len()).write(0);

            return buf;
        }
    }

    c_path
}

pub fn load_replacement(path: &impl AsRef<std::path::Path>) -> std::io::Result<()> {
    // fixme: File replacements should not be case-sensitive.
    let game_file_path = path_in_game_dir(path).unwrap();

    if !game_file_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "target file '{}' does not exist (you may need to change the capitalisation)",
                game_file_path.display()
            ),
        ));
    }

    PATH_SWAPS.lock().unwrap().insert(
        game_file_path.display().to_string(),
        path.as_ref().display().to_string(),
    );

    Ok(())
}

pub fn hook() {
    crate::targets::find_absolute_path::install(find_absolute_path_c);
}
