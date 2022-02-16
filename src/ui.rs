mod controller;
mod gui;
pub mod menu;
mod old_menu;
pub mod touch;

pub use gui::{colours, exit_to_homescreen};
pub use old_menu::{MenuMessage, RowData, RowDetail, TabData};
pub use touch::Stage;

pub fn init() {
    gui::init();
    old_menu::init();
    touch::init();
    controller::init();
}
