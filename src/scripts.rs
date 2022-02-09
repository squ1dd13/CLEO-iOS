mod asm;
mod base;
mod ctrl;
mod game;
mod game_old;
mod js;

pub use game_old::{load_invoked_script, load_running_script, tab_data_csa, tab_data_csi};

pub fn init() {
    js::init();
    ctrl::init();
}
