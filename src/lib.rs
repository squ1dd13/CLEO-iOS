//! Sets up CLEO when the library is loaded.

#![feature(panic_info_message)]

use std::os::raw::c_char;

use ctor::ctor;
use objc::runtime::Object;
use objc::runtime::Sel;

mod cheats;
mod extras;
mod files;
mod hook;
mod logging;
mod old_cheats;
mod scripts;
mod settings;
mod text;
mod ui;
mod update;

pub mod hooks {
    use super::*;
    use hook::Hook;

    /// Expands to a public static `Hook` structure with the given name, target signature and
    /// target address.
    macro_rules! add_hook {
        ($(#[$meta:meta])* $name:ident, $sig:ty, $address:literal) => {
            $(#[$meta])*
            pub static $name: Hook<$sig> = Hook::new($address);
        };
    }

    add_hook!(
        /// Updates the script runtime. Called every frame.
        SCRIPT_TICK,
        fn(),
        0x1001d0f40
    );

    add_hook!(
        /// Handles touch events on the game canvas.
        PROCESS_TOUCH,
        fn(f32, f32, f64, f32, ui::Stage),
        0x1004e831c
    );

    add_hook!(
        /// The Objective-C `viewDidLoad` method that sets up the legal splash screen shown when
        /// the user opens the app. (This address is only valid for the non-DE version.)
        LEGAL_SPLASH,
        fn(*mut Object, sel: Sel),
        0x1000d7cac
    );

    add_hook!(
        /// Alternative of the above hook that is valid in the DE version of the game.
        LEGAL_SPLASH_GERMAN,
        fn(*mut Object, sel: Sel),
        0x1000c6b40
    );

    add_hook!(
        /// Returns the UTF-16 string associated with the given UTF-8 key in the given text object.
        GET_GXT_STR,
        fn(usize, *const c_char) -> *const u16,
        0x10044142c
    );

    add_hook!(
        /// Normally crashes the game just before exit. We hook it so it doesn't crash and cause
        /// unnecessary crash report notifications to be sent.
        STORE_CRASH,
        fn(*mut Object, Sel) -> *const Object,
        0x100007c1c
    );

    add_hook!(
        /// An unused method that we hijack to use as a callback for various UIKit events.
        BUTTON_HACK,
        fn(*const Object, Sel, *mut Object) -> *mut Object,
        0x1004ebe70
    );

    add_hook!(
        /// Generates a random number plate string of the given length.
        GEN_PLATE,
        fn(*mut u8, i32) -> bool,
        0x10037ba2c
    );

    add_hook!(
        /// Updates a lot of the game's core systems (audio, graphics, menu, etc.) every frame.
        IDLE,
        fn(u64, u64),
        0x100242c20
    );

    add_hook!(
        /// Returns some timer-related value. We hook this because it gives us a place to
        /// manipulate the FPS target.
        CYCLES_PER_MS,
        fn() -> u32,
        0x10026c9c0
    );

    add_hook!(
        /// Updates the cheat system. Called every frame. We completely replace this with our own
        /// implementation because we re-implement the cheat system.
        DO_CHEATS,
        fn(),
        0x1001a7f28
    );

    add_hook!(
        /// Resets various aspects of the game's internals to get it ready for loading a
        /// different save.
        RESET_BEFORE_RESTART,
        fn(),
        0x1002ce55c
    );

    add_hook!(
        /// Returns the real path to a game resource. As the iOS game is a port of the PC
        /// version, it needs a way to translate the Windows paths that the game code uses into
        /// paths that actually relate to real files on the device; this function does that.
        FIND_ABS_PATH,
        fn(i32, *const u8, i32) -> *const u8,
        0x1004e4c48
    );

    add_hook!(
        /// Sets up the main menu and shows it to the user. We use this as a way to run code when
        /// the menu system has finished initialising.
        INIT_MAIN_MENU,
        fn(*mut u8),
        0x100339b44
    );

    add_hook!(
        /// Takes a bitmask and uses the individual bits to write a fragment shader line-by-line.
        ///
        /// Different bits indicate that different sets of pre-written source lines should be
        /// added to the shader. This is either a form of compression, obfuscation, or both.
        ///
        /// The produced source is stored in the global fragment shader buffer.
        WRITE_FRAG_SHADER,
        fn(u32),
        0x100137528
    );

    add_hook!(
        /// Loads the game's settings from the `gta_sa.set` file.
        LOAD_SETTINGS,
        fn(u64),
        0x1002ce8e4
    );

    add_hook!(
        /// Draws a string containing the current FPS to the screen. We completely replace the
        /// game's implementation of this.
        DISPLAY_FPS,
        fn(),
        0x100241cd8
    );

    add_hook!(
        /// Takes input from all connected game controllers.
        UPDATE_PADS,
        fn(),
        0x100244908
    );

    add_hook!(
        /// Reads the table of contents from an archive and registers the resources found with
        /// the appropriate systems.
        LOAD_ARCHIVE,
        fn(path: *const i8, archive_id: i32),
        0x1002f0e18
    );

    add_hook!(
        /// Sets the primary and secondary loading messages.
        ///
        /// It would appear that at some point, the game showed these messages near the loading
        /// bar, but this behaviour does not exist in the final game. We hook this so we can log
        /// the messages.
        REPORT_LOADING,
        fn(*const c_char, *const c_char),
        0x1002b5a78
    );

    add_hook!(
        /// Resets the cheat system, ready for a new load.
        RESET_CHEATS,
        fn(),
        0x1001a8194
    );

    add_hook!(
        /// Carries out the third stage of the game's loading sequence.
        INIT_STAGE_3,
        fn(usize),
        0x1002f9b20
    );
}

fn initialise() {
    files::init();
    settings::init();
    ui::init();
    text::init();
    extras::init();
    scripts::init();
    // old_cheats::init();
    cheats::init();
}

#[ctor]
fn load() {
    // Load the logging system before everything else so we can log from constructors.
    logging::init();

    log::info!(
        r#"
 
                         Welcome to CLEO iOS!                        
             Written by @squ1dd13 (squ1dd13dev@gmail.com).           
        Made with ❤️ in Great Britain. Proudly written in Rust.       
  Check out the GitHub repo at https://github.com/squ1dd13/CLEO-iOS. 
 Need support? Join the Discord server! https://discord.gg/cXwkTUasJU
"#
    );

    // todo: Log game version.
    log::info!("Cargo package version is {}", env!("CARGO_PKG_VERSION"));

    // Start checking for updates in the background.
    update::start_update_check();

    // Load all the modules.
    initialise();
}
