use std::os::raw::c_char;

mod hook;
mod logging;

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

#[ctor::ctor]
fn init() {
    unsafe { STATIC_LOG = Some(logging::Logger::new("tweak")) };

    let log = get_log();

    log.connect_udp("192.168.1.183:4568");
    log.connect_file("/var/mobile/Documents/tweak.log");

    // Log an empty string so we get a break after the output from the last run.
    log.normal("");

    log.normal("Test plain string");
    log.warning("Test warning");
    log.error("Test error");
    log.important("Test important");
}
