//! Sets up CLEO when the library is loaded.

#![feature(panic_info_message)]
#![feature(cstr_from_bytes_until_nul)]
#![feature(map_try_insert)]
#![feature(drain_filter)]

use ctor::ctor;
use objc::runtime::Object;
use objc::runtime::Sel;
use std::os::raw::c_char;

mod game;
mod hook;
mod logging;
mod meta;

mod targets {
    #![allow(clippy::unreadable_literal)]

    use super::{c_char, create_hard_target, create_soft_target, Object, Sel};

    create_soft_target!(script_tick, 0x1001d0f40, fn());

    create_soft_target!(process_touch, 0x1004e831c, fn(f32, f32, f64, f32, u64));

    create_soft_target!(
        get_gxt_string,
        0x10044142c,
        fn(usize, *const c_char) -> *const u16
    );

    create_soft_target!(legal_splash, 0x1000d7cac, fn(*mut Object, sel: Sel));
    create_soft_target!(legal_splash_german, 0x1000c6b40, fn(*mut Object, sel: Sel));

    create_soft_target!(
        store_crash_fix,
        0x100007c1c,
        fn(*mut Object, Sel) -> *const Object
    );

    create_soft_target!(
        button_hack,
        0x1004ebe70,
        fn(*const Object, Sel, *mut Object) -> *mut Object
    );

    create_soft_target!(gen_plate, 0x10037ba2c, fn(*mut u8, i32) -> bool);

    create_soft_target!(idle, 0x100242c20, fn(u64, u64));

    create_soft_target!(cycles_per_millisecond, 0x10026c9c0, fn() -> u32);

    create_soft_target!(do_game_state, 0x1004b6a54, fn());

    create_hard_target!(do_cheats, 0x1001a7f28, fn());

    create_soft_target!(reset_before_start, 0x1002ce55c, fn());

    create_soft_target!(
        find_absolute_path,
        0x1004e4c48,
        fn(i32, *const u8, i32) -> *const u8
    );

    create_soft_target!(init_for_title, 0x100339b44, fn(*mut u8));

    create_soft_target!(write_fragment_shader, 0x100137528, fn(u32));

    create_soft_target!(write_vertex_shader, 0x100137cd0, fn(u32));

    create_soft_target!(load_settings, 0x1002ce8e4, fn(u64));

    create_hard_target!(display_fps, 0x100241cd8, fn());

    create_soft_target!(update_pads, 0x100244908, fn());

    create_soft_target!(
        load_cd_directory,
        0x1002f0e18,
        fn(*const i8, archive_id: u32)
    );

    create_soft_target!(
        end_dragging,
        0x1000cbd08,
        fn(*const Object, Sel, *mut Object, bool)
    );

    create_hard_target!(
        loading_messages,
        0x1002b5a78,
        fn(*const c_char, *const c_char)
    );

    create_soft_target!(reset_cheats, 0x1001a8194, fn());

    create_soft_target!(
        height_above_ceiling,
        0x1004801e0,
        fn(usize, f32, usize) -> f32
    );

    create_soft_target!(init_stage_three, 0x1002f9b20, fn(usize));
}

#[ctor]
fn load() {
    // Load the logging system before everything else so we can log from constructors.
    logging::init();

    if hook::can_hook() {
        log::info!("hook test successful! CLEO should work ok :)");
    } else {
        log::error!("hook test failed! CLEO probably won't work :( please report this error!");
    }

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

    log::info!(
        "game ASLR slide is {:#x}",
        crate::hook::get_game_aslr_offset(),
    );

    // Set up CLEO first.
    meta::init();

    // Load all of our game systems.
    game::init();
}
