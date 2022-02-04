mod controller;
mod gui;
mod menu;
pub mod touch;

pub use gui::{colours, exit_to_homescreen};
pub use menu::{MenuMessage, RowData, RowDetail, TabData};
pub use touch::TouchType;

pub fn init() {
    gui::init();
    menu::init();
    touch::init();
    controller::init();
}
