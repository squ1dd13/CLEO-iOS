use std::os::raw::c_char;

mod hook;
mod logging;
mod scripts;

fn get_log() -> &'static mut logging::Logger {
    static mut STATIC_LOG: Option<logging::Logger> = None;

    unsafe {
        if STATIC_LOG.is_some() {
            return STATIC_LOG.as_mut().unwrap();
        }

        let mut log = logging::Logger::new("cleo");

        log.connect_udp("192.168.1.183:4568");
        log.connect_file("/var/mobile/Documents/tweak.log");

        STATIC_LOG = Some(log);
        STATIC_LOG.as_mut().unwrap()
    }
}

mod targets {
    use super::*;

    create_soft_target!(game_load, 0x100240178, fn(*const c_char));
    create_soft_target!(script_tick, 0x1001d0f40, fn());
}

fn game_load_hook(dat_path: *const c_char) {
    let c_str: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(dat_path) };
    let path_str: &str = c_str.to_str().unwrap();

    get_log().normal(format!("Loading game using file {}", path_str));

    call_original!(targets::game_load, dat_path);
}

fn install_hooks() {
    targets::game_load::install(game_load_hook);
    scripts::Script::install_hooks();
}

fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let backtrace = backtrace::Backtrace::new();

        if let Some(s) = info.payload().downcast_ref::<&str>() {
            get_log().error(format!("\npanic: {:?}\n\nbacktrace:\n{:?}", s, backtrace));
        } else {
            get_log().error(format!("\npanic\n\nbacktrace:\n{:?}", backtrace));
        }
    }));
}

#[ctor::ctor]
fn init() {
    let log = get_log();

    log.connect_udp("192.168.1.183:4568");
    log.connect_file("/var/mobile/Documents/tweak.log");

    set_panic_hook();

    // Log an empty string so we get a break after the output from the last run.
    log.normal("");

    install_hooks();

    log.normal("Test plain string");
    log.warning("Test warning");
    log.error("Test error");
    log.important("Test important");

    let script_vec = scripts::Script::load_dir(&"/var/mobile/Documents/CS");

    if let Err(error) = script_vec {
        log.error(format!("Unable to load scripts directory: {}", error));
        return;
    }

    for script in script_vec.unwrap() {
        if let Ok(script) = script {
            log.normal(format!("Loaded: {}", script.name()));

            unsafe { scripts::LOADED_SCRIPTS.push(script) }

            continue;
        }

        log.error(format!("Unable to load script: {}", script.err().unwrap()));
    }
}
