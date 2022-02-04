mod js;
mod run;
mod scm;

pub fn load_invoked_script(path: &impl AsRef<std::path::Path>) -> eyre::Result<()> {
    run::load_invoked_script(path)
}

pub fn load_running_script(path: &impl AsRef<std::path::Path>) -> eyre::Result<()> {
    run::load_running_script(path)
}

pub fn tab_data_csa() -> crate::ui::TabData {
    run::tab_data_csa()
}

pub fn tab_data_csi() -> crate::ui::TabData {
    run::tab_data_csi()
}

pub fn init() {
    run::init();
    js::init();
}
