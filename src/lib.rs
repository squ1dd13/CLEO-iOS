use std::os::raw::c_char;

use ctor::ctor;
use log::{debug, error, info};

mod files;
mod hook;
mod scripts;
mod text;
mod udp_log;
mod ui;

mod targets {
    use super::*;

    create_soft_target!(game_load, 0x100240178, fn(*const c_char));
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

// Log a message when a game loads. This is helpful to let us know that
//  hooking has worked, but also means we have a hook which we can use
//  to initialise anything when a game is started if we need that in the
//  future.
fn game_load_hook(dat_path: *const c_char) {
    debug!("Loading game.");
    call_original!(targets::game_load, dat_path);
}

fn install_hooks() {
    targets::game_load::install(game_load_hook);
    scripts::hook();
    ui::hook();
    text::hook();
}

fn load_script_dir() {
    if let Err(err) = files::load_all(files::get_cleo_dir_path()) {
        error!("Encountered error while loading CS directory: {}", err);
    } else {
        info!("Loaded CS directory with no top-level errors.");
    }
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
    load_script_dir();
}
