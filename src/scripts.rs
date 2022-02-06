mod base;
mod ctrl;
mod game;
mod js;
mod scm;

pub use game::{load_invoked_script, load_running_script, tab_data_csa, tab_data_csi};

pub fn init() {
    game::init();
    js::init();
}
