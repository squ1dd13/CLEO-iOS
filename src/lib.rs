use std::os::raw::c_char;

mod hook;
mod logging;
mod scripts;

static mut STATIC_LOG: Option<logging::Logger> = None;

fn get_log() -> &'static mut logging::Logger {
    unsafe { STATIC_LOG.as_mut() }.unwrap()
}

mod targets {
    use super::*;

    create_soft_target!(game_load, 0x100240178, fn(*const c_char));
}

fn game_load_hook(dat_path: *const c_char) {
    let c_str: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(dat_path) };
    let path_str: &str = c_str.to_str().unwrap();

    get_log().normal(format!("Loading game using file {}", path_str));

    call_original!(targets::game_load, dat_path);
}

#[ctor::ctor]
fn install_hooks() {
    targets::game_load::install(game_load_hook);
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
    unsafe { STATIC_LOG = Some(logging::Logger::new("cleo")) };

    let log = get_log();

    log.connect_udp("192.168.1.183:4568");
    log.connect_file("/var/mobile/Documents/tweak.log");

    set_panic_hook();

    // Log an empty string so we get a break after the output from the last run.
    log.normal("");

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
            continue;
        }

        log.error(format!("Unable to load script: {}", script.err().unwrap()));
    }
}
