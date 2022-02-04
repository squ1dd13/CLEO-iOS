mod js;
mod run;
mod scm;

pub use run::{load_invoked_script, load_running_script, tab_data_csa, tab_data_csi};

pub fn init() {
    run::init();
    js::init();
}
