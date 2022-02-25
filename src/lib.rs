//! Sets up CLEO when the library is loaded.

#![feature(panic_info_message)]

use ctor::ctor;

mod cheats;
mod extras;
mod files;
mod hook;
mod logging;
mod old_cheats;
mod scripts;
mod settings;
mod text;
mod ui;
mod update;

fn initialise() {
    files::init();
    settings::init();
    ui::init();
    text::init();
    extras::init();
    scripts::init();
    cheats::init();
}

#[ctor]
fn load() {
    // Load the logging system before everything else so we can log from constructors.
    logging::init();

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

    // Start checking for updates in the background.
    update::start_update_check();

    // Load all the modules.
    initialise();
}
