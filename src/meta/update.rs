//! Interfaces with the GitHub API to determine if a CLEO update is available, and manages
//! the version cache.

use crate::{
    call_original,
    github::{CheckStatus, Version},
    hook, text,
};

use objc::{runtime::Object, *};

/// Opens `url` in the user's default browser.
fn open_url(url: impl AsRef<str>) {
    unsafe {
        let url: *const Object = msg_send![
            class!(NSURL),
            URLWithString: crate::gui::ns_string(url.as_ref())
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
    no_fn: fn(),
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
    hook::slide::<fn(usize, *const u8, *const u8, fn(usize), usize, fn(), bool) -> u64>(
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
fn show_update_prompt(screen: *mut u8, update_ver: Version) {
    fn on_yes(_: usize) {
        let version =
            if let CheckStatus::Finished(Ok(Some(version))) = *crate::github::get_check_status() {
                version
            } else {
                panic!("User answered 'yes' to update, but no update found");
            };

        let url = version.url();

        log::info!("User accepted update to {}. Heading to {}...", version, url);

        open_url(url);
    }

    fn on_no() {
        log::info!("User rejected update.");
    }

    show_yes_no_menu(
        screen,
        "Update Available",
        format!(
            "CLEO version {update_ver} is available. Do you want to go to GitHub to download it?"
        ),
        0,
        on_yes,
        on_no,
    );
}

// This function is responsible for setting up the main flow screen, so we use it to
//  show our update prompt when the game loads.
fn init_for_title(screen: *mut u8) {
    // Set up the title menu.
    call_original!(crate::targets::init_for_title, screen);

    match &*crate::github::get_check_status() {
        CheckStatus::NotStarted => log::info!("Update check never started"),
        CheckStatus::NotFinished => log::warn!("Update check took too long"),
        CheckStatus::Finished(result) => match result {
            Ok(update) => match update {
                Some(update) => {
                    log::info!("Check successful. Found update: {}", update);
                    show_update_prompt(screen, *update);
                }

                None => log::info!("Check successful, but no update available"),
            },

            Err(err) => log::error!("Update check failed: {:?}", err),
        },
    }
}

pub fn init() {
    log::info!("installing update hook...");
    crate::targets::init_for_title::install(init_for_title);
}
