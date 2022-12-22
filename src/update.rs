//! Interfaces with the GitHub API to determine if a CLEO update is available, and manages
//! the version cache.

use crate::{call_original, github::Version, hook, resources, text};
use eyre::Result;
use objc::{runtime::Object, *};
use std::{
    io::{Read, Write},
    sync::Mutex,
};

/// Opens `url` in the user's default browser.
fn open_url(url: impl AsRef<str>) {
    unsafe {
        let url: *const Object = msg_send![
            class!(NSURL),
            URLWithString: crate::gui::create_ns_string(url.as_ref())
        ];

        let shared_app: *const Object = msg_send![class!(UIApplication), sharedApplication];

        // eq: [[UIApplication sharedApplication] openURL: [NSURL URLWithString: ...]]
        let _: () = msg_send![shared_app, openURL: url];
    }
}

/// Presents the user with a menu with "yes" and "no" options on the given screen.
fn show_yes_no_menu(
    screen: *mut u8,
    title: impl AsRef<str>,
    message: impl AsRef<str>,
    callback_arg: usize,
    yes_fn: fn(usize),
    no_fn: fn(usize),
) {
    unsafe {
        screen.offset(0x75).write(0);

        // eq: MobileMenu::Load(...)
        hook::slide::<fn(*mut u8)>(0x100339838)(screen);
    }

    // Create localisation keys for the title and message so we can show them in the menu.
    text::set_kv("NAG_TTL", title.as_ref());
    text::set_kv("NAG_MSG", message.as_ref());

    // eq: nag_menu = operator.new(0x80)
    let menu = hook::slide::<fn(u64) -> usize>(0x1004f9be0)(0x80);

    // eq: MobileMenu::InitForNag(...)
    hook::slide::<fn(usize, *const u8, *const u8, fn(usize), usize, fn(usize), bool) -> u64>(
        0x100348964,
    )(
        menu,                  // Menu structure (uninitialised before call)
        b"NAG_TTL\0".as_ptr(), // Title
        b"NAG_MSG\0".as_ptr(), // Message
        yes_fn,                // "Yes" function
        callback_arg,          // Callback argument
        no_fn,                 // "No" function
        false,                 // Enable 'back' button
    );

    // We could create a repl(C) struct, but the fields we need are at fairly large
    //  offsets, so it's easiest just to mess with pointers.
    let screen: *mut usize = screen.cast();

    // Offset is 6 * u64, so 48 bytes (0x30).
    if unsafe { screen.offset(6).read() } != 0 {
        // eq: MobileMenu::ProcessPending(...)
        hook::slide::<fn(*mut usize)>(0x100338f5c)(screen);
    }

    unsafe {
        screen.offset(6).write(menu);
    }
}

/// Shows the user a yes/no prompt asking if they'd like to update.
fn show_update_prompt(screen: *mut u8, (update_ver, update_url): (Version, String)) {
    // We can't capture any variables in our callback functions, because they have to be
    // C-compatible function pointers. This is a problem, because the 'yes' callback needs to be
    // able to open the URL given to this function. Luckily, we have an 8-byte value that we can
    // pass to the menu that will be passed to our callbacks. We use this to store a boxed pointer
    // to the URL in raw form (`*mut String`). It is very important that we turn this back into a
    // `Box` and drop it in _both_ callbacks in order to free the memory when we no longer need it.

    // `Box<String>` is normally frowned upon, but we don't have much of a choice here: we need an
    // 8-byte pointer only, and `Box<str>` can only be converted to a fat pointer, which won't fit
    // in the space we have.
    let raw_url_box = Box::into_raw(Box::new(update_url));

    fn unbox_string(raw: usize) -> String {
        unsafe {
            let raw_box = raw as *mut String;
            *Box::from_raw(raw_box)
        }
    }

    fn on_yes(raw_url_box: usize) {
        let url = unbox_string(raw_url_box);

        log::info!("User accepted update. Heading to {}...", url);
    }

    fn on_no(raw_url_box: usize) {
        let url = unbox_string(raw_url_box);

        log::info!("User rejected update. URL was {}.", url);
    }

    show_yes_no_menu(
        screen,
        "Update Available",
        format!(
            "CLEO version {update_ver} is available. Do you want to go to GitHub to download it?"
        ),
        raw_url_box as usize,
        on_yes,
        on_no,
    );
}

// This function is responsible for setting up the main flow screen, so we use it to
//  show our update prompt when the game loads.
fn init_for_title(screen: *mut u8) {
    // Set up the title menu.
    call_original!(crate::targets::init_for_title, screen);

    if let Some(result) = crate::github::take_check_result() {
        match result {
            Ok(Some(update)) => {
                show_update_prompt(screen, update);
            }

            Ok(None) => log::info!("No update available"),
            Err(err) => log::error!("Error while checking for update: {:?}", err),
        }
    }
}

pub fn init() {
    log::info!("installing update hook...");
    crate::targets::init_for_title::install(init_for_title);
}
