use ctor::ctor;

use files::ComponentSystem;
use objc::runtime::Sel;

use objc::runtime::Object;

use log::{debug, error, info};
use std::os::raw::c_char;

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
    create_soft_target!(legal_splash, 0x1000d7cac, fn(*mut Object, sel: Sel));
    create_soft_target!(
        store_crash_fix,
        0x100007c1c,
        fn(*mut Object, sel: Sel) -> *const Object
    );
}

static mut COMPONENT_SYSTEM: Option<ComponentSystem> = None;

fn get_component_system() -> &'static mut Option<ComponentSystem> {
    unsafe { &mut COMPONENT_SYSTEM }
}

fn load_scripts_hook() {
    debug!("Loading scripts.");
    call_original!(targets::game_load_scripts);

    get_component_system().as_mut().unwrap().reset_all();
}

fn install_hooks() {
    targets::game_load_scripts::install(load_scripts_hook);
    scripts::hook();
    ui::hook();
    text::hook();
}

#[ctor]
fn load() {
    // Load the logger before everything else so we can log from constructors.
    let logger = udp_log::Logger::new("cleo");
    logger.connect_udp("192.168.1.183:4568");

    if let Err(err) = files::setup_cleo_fs() {
        error!("setup_cleo_fs error: {}", err);
    }

    files::ComponentSystem::register_extension("csa", scripts::ScriptComponent::new);
    files::ComponentSystem::register_extension("csi", scripts::ScriptComponent::new);
    files::ComponentSystem::register_extension("fxt", files::LanguageFile::new);

    let component_system = ComponentSystem::new(files::get_cleo_dir_path());

    if let Err(err) = component_system {
        error!("Unable to create component system! {}", err);
        panic!("{}", err);
    }

    unsafe {
        COMPONENT_SYSTEM = Some(component_system.unwrap());
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
