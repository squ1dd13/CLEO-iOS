mod base;
mod ctrl;
mod js;
mod scm;

pub use ctrl::{load_invoked_script, load_running_script, tab_data_csa, tab_data_csi};

pub fn init() {
    ctrl::init();
    js::init();
}
