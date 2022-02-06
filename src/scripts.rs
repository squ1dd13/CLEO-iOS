mod base;
mod ctrl;
mod game;
mod game_old;
mod js;
mod scm;

pub use game_old::{load_invoked_script, load_running_script, tab_data_csa, tab_data_csi};

pub fn init() {
    game_old::init();
    js::init();
}
