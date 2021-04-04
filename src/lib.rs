use std::os::raw::c_char;

mod hook;
mod logging;

static mut STATIC_LOG: Option<logging::Logger> = None;

fn get_log() -> &'static mut logging::Logger {
    unsafe { STATIC_LOG.as_mut() }.unwrap()
}

mod targets {
    use super::*;

    define_target!(GAME_LOAD, 0x100240178, fn(*const c_char));
}

static mut LOAD_ORIGINAL: Option<fn(*const c_char)> = None;

fn load_replacement(dat_path: *const c_char) {
    let c_str: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(dat_path) };
    let path_str: &str = c_str.to_str().unwrap();

    get_log().normal(format!("loading game from {}", path_str));
    unsafe {
        LOAD_ORIGINAL.unwrap()(dat_path);
    }
}

#[ctor::ctor]
fn init() {
    unsafe { STATIC_LOG = Some(logging::Logger::new("tweak")) };

    let log = get_log();

    log.connect_udp("192.168.1.183:4568");
    log.connect_file("/var/mobile/Documents/tweak.log");

    // Log an empty string so we get a break after the output from the last run.
    log.normal("");

    targets::GAME_LOAD.hook_soft(load_replacement, unsafe { &mut LOAD_ORIGINAL });

    log.normal("Test plain string");
    log.warning("Test warning");
    log.error("Test error");
    log.important("Test important");
}
