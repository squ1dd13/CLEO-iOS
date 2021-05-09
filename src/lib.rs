use ctor::ctor;

use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use std::{
    os::raw::c_char,
    sync::atomic::{AtomicBool, Ordering},
};

mod files;
mod hook;
mod scripts;
mod text;
mod udp_log;
mod ui;

mod targets {
    use super::*;

    create_soft_target!(game_load_scripts, 0x1001cff00, fn());
    create_soft_target!(script_tick, 0x1001d0f40, fn());
    create_soft_target!(
        process_touch,
        0x1004e831c,
        fn(f32, f32, f64, f32, ui::TouchType)
    );
    create_soft_target!(
        get_gxt_string,
        0x10044142c,
        fn(usize, *const c_char) -> *const u16
    );
    // create_soft_target!(vertex_shader, 0x100137cd0, fn(u64));
}

fn load_existing_game() {
    debug!("Loading scripts.");
    call_original!(targets::game_load_scripts);

    load_resources_if_needed();
}

fn install_hooks() {
    targets::game_load_scripts::install(load_existing_game);
    scripts::hook();
    ui::hook();
    text::hook();
}

fn reset() {
    debug!("Resetting hooked systems.");
    text::reset();
    scripts::reset();
}

fn load_resources_if_needed() {
    if scripts::are_scripts_fresh() {
        // Scripts have not been invalidated, so no reload needed.
        // However, we emit a warning because this could be incorrect if the scripts are actually invalid.
        warn!("Scripts reported as valid on resource load check.");
        return;
    }

    reset();

    if let Err(err) = files::load_all(files::get_cleo_dir_path()) {
        error!("Encountered error while loading CLEO directory: {}", err);
    }

    scripts::set_scripts_fresh(true);
}

#[ctor]
fn load() {
    // Load the logger before everything else so we can log from constructors.
    let logger = udp_log::Logger::new("cleo");
    logger.connect_udp("192.168.1.183:4568");

    if let Err(err) = files::setup_cleo_fs() {
        error!("setup_cleo_fs error: {}", err);
    }

    logger.connect_file(files::get_log_path());

    log::set_logger(unsafe { udp_log::GLOBAL_LOGGER.as_ref().unwrap() })
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

    install_hooks();
}
