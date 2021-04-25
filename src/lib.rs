use std::os::raw::c_char;

use ctor::ctor;
use log::{debug, error, info};

mod hook;
mod udp_log;
mod scripts;
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
    // create_soft_target!(vertex_shader, 0x100137cd0, fn(u64));
}

fn game_load_hook(dat_path: *const c_char) {
    let c_str: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(dat_path) };
    let path_str: &str = c_str.to_str().unwrap();

    debug!("Loading game using file {}", path_str);

    call_original!(targets::game_load, dat_path);
}

fn install_hooks() {
    targets::game_load::install(game_load_hook);
    scripts::Script::install_hooks();
    ui::install_hooks();
}

fn load_script_dir() {
    let script_vec = scripts::Script::load_dir(&"/var/mobile/Documents/CS");

    if let Err(error) = script_vec {
        error!("Unable to load scripts directory: {}", error);
        return;
    }

    for script in script_vec.unwrap() {
        if let Ok(script) = script {
            info!("Loaded: {}", script.name());

            scripts::loaded_scripts().push(script);
            continue;
        }

        error!("Unable to load script: {}", script.err().unwrap());
    }
}

#[ctor]
fn load() {
    // Load the logger before everything else so we can log from constructors.
    let logger = udp_log::Logger::new("cleo");
    logger.connect_udp("192.168.1.183:4568");
    logger.connect_file("/var/mobile/Documents/tweak.log");

    log::set_logger(unsafe { udp_log::GLOBAL_LOGGER.as_ref().unwrap() })
        .map(|()| log::set_max_level(log::LevelFilter::max()))
        .expect("should work");

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
  Check out the GitHub repo at https://github.com/Squ1dd13/CLEO-iOS.  
**********************************************************************
"#
    );

    install_hooks();
    load_script_dir();
}
