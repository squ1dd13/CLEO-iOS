use ctor::ctor;
use log::{error, info};
use objc::runtime::Object;
use objc::runtime::Sel;
use std::os::raw::c_char;

mod cheats;
mod controller;
mod gui;
mod hook;
mod loader;
mod menu;
mod render;
mod resources;
mod scripts;
mod settings;
mod stream;
mod text;
mod touch;
mod udp;
mod update;

mod targets {
    use super::*;

    create_soft_target!(game_load_scripts, 0x1001cff00, fn());

    create_soft_target!(script_tick, 0x1001d0f40, fn());

    create_soft_target!(
        process_touch,
        0x1004e831c,
        fn(f32, f32, f64, f32, touch::TouchType)
    );

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

    create_soft_target!(load_settings, 0x1002ce8e4, fn(u64));

    create_hard_target!(display_fps, 0x100241cd8, fn());

    create_soft_target!(update_pads, 0x100244908, fn());

    create_soft_target!(
        load_cd_directory,
        0x1002f0e18,
        fn(*const i8, archive_id: i32)
    );

    create_soft_target!(
        end_dragging,
        0x1000cbd08,
        fn(*const Object, Sel, *mut Object, bool)
    );
}

fn install_hooks() {
    stream::hook();
    update::hook();
    loader::hook();
    settings::hook();
    gui::hook();
    menu::hook();
    touch::hook();
    text::hook();
    render::hook();
    scripts::hook();
    cheats::hook();
    controller::hook();

    resources::initialise();
}

#[ctor]
fn load() {
    // Load the logger before everything else so we can log from constructors.
    let logger = udp::Logger::new("cleo");
    logger.connect_udp("192.168.1.183:4568");
    logger.connect_file(resources::get_log_path());

    log::set_logger(unsafe { udp::GLOBAL_LOGGER.as_ref().unwrap() })
        .map(|_| log::set_max_level(log::LevelFilter::max()))
        .unwrap();

    // Install the panic hook so we can print useful stuff rather than just exiting on a panic.
    std::panic::set_hook(Box::new(|info| {
        let backtrace = backtrace::Backtrace::new();

        if let Some(s) = info.payload().downcast_ref::<&str>() {
            error!("\npanic: {:?}\n\nbacktrace:\n{:?}", s, backtrace);
        } else {
            error!("\npanic\n\nbacktrace:\n{:?}", backtrace);
        }
    }));

    info!(
        r#"

**********************************************************************
                         Welcome to CLEO iOS!                         
                By @squ1dd13 (squ1dd13dev@gmail.com).                 
                        Made with ❤️ in 🇬🇧.                           
  Check out the GitHub repo at https://github.com/squ1dd13/CLEO-iOS.  
**********************************************************************
"#
    );

    crate::update::start_update_checking();
    install_hooks();
}
