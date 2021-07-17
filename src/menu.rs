//! Provides a touch interface and logic to allow the user to interact with scripts, cheats and settings.

use std::sync::{
    mpsc::{self, Sender},
    Arc, Mutex,
};

use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use once_cell::sync::OnceCell;

use crate::gui::{self, create_ns_string, CGPoint, CGRect, CGSize};

pub trait RowData {
    fn title(&self) -> &str;
    fn detail(&self) -> Option<&str>;
    fn value(&self) -> &str;
    fn foreground(&self) -> (u8, u8, u8, u8);
    fn handle_tap(&mut self);
}

pub struct TabData {
    pub name: String,
    pub warning: Option<String>,
    pub row_data: Vec<Box<dyn RowData>>,
}

struct Row {
    data: Box<dyn RowData>,
    detail_label: *mut Object,
    value_label: *mut Object,
    button: *mut Object,
}

// fixme: New states don't incorporate controller stuff.
struct TabState {
    selected: bool,
    scroll_y: f64,
}

struct Tab {
    name: String,
    warning: Option<String>,
    scroll_view: *mut Object,
    rows: Vec<Row>,
    state: TabState,
}

struct TabButton {
    view: *mut Object,
}

#[repr(C)]
struct ButtonTag {
    tab: Option<u8>,
    row: Option<u32>,
    is_close: bool,
    _pad: [u8; 2],
}

impl ButtonTag {
    fn new_tab(index: usize) -> ButtonTag {
        ButtonTag {
            tab: Some(index as u8),
            row: None,
            is_close: false,
            _pad: [0u8; 2],
        }
    }

    fn new_row(tab: usize, row: usize) -> ButtonTag {
        ButtonTag {
            tab: Some(tab as u8),
            row: Some(row as u32),
            is_close: false,
            _pad: [0u8; 2],
        }
    }

    fn new_close() -> ButtonTag {
        ButtonTag {
            tab: None,
            row: None,
            is_close: true,
            _pad: [0u8; 2],
        }
    }
}

struct Menu {
    tabs: Vec<Tab>,
    tab_buttons: Vec<TabButton>,
    close_button: *mut Object,
}

static MESSAGE_SENDER: OnceCell<Mutex<Sender<MenuMessage>>> = OnceCell::new();

#[derive(Debug)]
pub enum MenuMessage {
    Show,
    Hide,
}

impl MenuMessage {
    /// Send the message using the default sender. Requires locking the sender mutex, so
    /// will block until the mutex becomes available.
    pub fn send(self) {
        if let Some(sender) = MESSAGE_SENDER.get() {
            self.send_with_sender(&sender.lock().unwrap());
        } else {
            log::error!(
                "cannot send {:?} because sender is not initialised yet",
                self
            );
        }
    }

    /// Attempts to clone the default sender in order to create a sender that can be used
    /// without needing to wait for other threads to finish using the default one.
    pub fn clone_sender() -> Option<Sender<Self>> {
        Some(MESSAGE_SENDER.get()?.lock().ok()?.clone())
    }

    /// Directly send this message using the specified sender.
    pub fn send_with_sender(self, sender: &Sender<Self>) {
        if let Err(err) = sender.send(self) {
            log::error!("failed to send {:?}", err.0);
        }
    }
}

impl Row {
    fn new(data: Box<dyn RowData>, frame: gui::CGRect) -> Row {
        unsafe {
            let button: *mut Object = msg_send![class!(UIButton), alloc];
            let button: *mut Object = msg_send![button, initWithFrame: frame];

            let _: () = msg_send![button, setTitle: create_ns_string(data.title()) forState: 0u64];

            Row {
                data,
                detail_label: std::ptr::null_mut(),
                value_label: std::ptr::null_mut(),
                button,
            }
        }
    }
}

// Previously, we used multipliers for all of the element sizes. This produced good results on
//  smaller devices, but on iPads, many things were too big and the menu as a whole looked strange.
// The values seen here are loosely based on the values from an iPhone 8 using the old system.
const ROW_HEIGHT: f64 = 85.;
const TAB_BUTTON_HEIGHT: f64 = 65.;
const CLOSE_BUTTON_HEIGHT: f64 = 35.;

impl Tab {
    fn new(data: TabData, tab_frame: gui::CGRect, state: TabState) -> Tab {
        unsafe {
            let scroll_frame = if data.warning.is_some() {
                // Make the scroll view slightly shorter so we can fit the warning above it.
                CGRect::new(
                    tab_frame.origin.x,
                    tab_frame.origin.y + tab_frame.size.height * 0.05,
                    tab_frame.size.width,
                    tab_frame.size.height * 0.95,
                )
            } else {
                tab_frame
            };

            let scroll_view: *mut Object = msg_send![class!(UIScrollView), alloc];
            let scroll_view: *mut Object = msg_send![scroll_view, initWithFrame: scroll_frame];

            let row_width = scroll_frame.size.width;

            let make_row = |(index, data)| {
                Row::new(
                    data,
                    CGRect::new(0., ROW_HEIGHT * index as f64, row_width, ROW_HEIGHT),
                )
            };

            // Move all the RowData elements into Row structures.
            let rows: Vec<Row> = data
                .row_data
                .into_iter()
                .enumerate()
                .map(make_row)
                .collect();

            for row in rows.iter() {
                let _: () = msg_send![scroll_view, addSubview: row.button];
            }

            Tab {
                name: data.name,
                warning: data.warning,
                scroll_view,
                rows,
                state,
            }
        }
    }
}

impl TabButton {
    fn new(title: &str, index: usize, width: f64) -> TabButton {
        let view = unsafe {
            let btn: *mut Object = msg_send![class!(UIButton), alloc];

            let frame = CGRect::new(width * index as f64, 0., width, TAB_BUTTON_HEIGHT);
            let btn: *mut Object = msg_send![btn, initWithFrame: frame];

            let _: () = msg_send![btn, setTitle: create_ns_string(title) forState: 0u64];
            let _: () = msg_send![btn, setTag: ButtonTag::new_tab(index)];

            btn
        };

        TabButton { view }
    }
}

impl Menu {
    fn new(tab_data: Vec<TabData>) -> Menu {
        let frame: CGRect = unsafe {
            let application: *mut Object = msg_send![class!(UIApplication), sharedApplication];
            let key_window: *mut Object = msg_send![application, keyWindow];
            msg_send![key_window, frame]
        };

        let tab_btn_width = frame.size.width / tab_data.len() as f64;

        let tab_buttons = tab_data
            .iter()
            .enumerate()
            .map(|(index, data)| TabButton::new(&data.name, index, tab_btn_width))
            .collect();

        let tab_frame = CGRect::new(
            0.,
            TAB_BUTTON_HEIGHT,
            frame.size.width,
            frame.size.height - (TAB_BUTTON_HEIGHT + CLOSE_BUTTON_HEIGHT),
        );

        // Move all the tab data into Tab structures.
        // todo: Use saved TabState structures rather than making plain ones.
        let tabs = tab_data.into_iter().enumerate().map(|(tab_index, data)| {
            let tab = Tab::new(
                data,
                tab_frame,
                TabState {
                    selected: false,
                    scroll_y: 0.,
                },
            );

            for (row_index, row) in tab.rows.iter().enumerate() {
                unsafe {
                    let _: () =
                        msg_send![row.button, setTag: ButtonTag::new_row(tab_index, row_index)];
                }
            }

            tab
        });

        // We collect here instead of chaining so that the formatting above is nicer.
        let tabs = tabs.collect();

        let close_button: *mut Object = unsafe {
            let btn: *mut Object = msg_send![class!(UIButton), alloc];

            let btn_frame = CGRect::new(
                0.,
                frame.size.height - CLOSE_BUTTON_HEIGHT,
                frame.size.width,
                CLOSE_BUTTON_HEIGHT,
            );

            let btn: *mut Object = msg_send![btn, initWithFrame: btn_frame];
            let _: () = msg_send![btn, setTitle: create_ns_string("Close") forState: 0u64];
            let _: () = msg_send![btn, setTag: ButtonTag::new_close()];

            btn
        };

        Menu {
            tabs,
            tab_buttons,
            close_button,
        }
    }

    fn add_to_window(&self) {
        unsafe {
            let application: *mut Object = msg_send![class!(UIApplication), sharedApplication];
            let key_window: *mut Object = msg_send![application, keyWindow];

            for tab_button in self.tab_buttons.iter() {
                let _: () = msg_send![key_window, addSubview: tab_button.view];
            }

            for tab in self.tabs.iter() {
                let _: () = msg_send![key_window, addSubview: tab.scroll_view];
            }

            let _: () = msg_send![key_window, addSubview: self.close_button];
        }
    }

    fn remove(self) {
        unsafe {
            for tab_button in self.tab_buttons.iter() {
                let _: () = msg_send![tab_button.view, removeFromSuperview];
            }

            for tab in self.tabs.iter() {
                let _: () = msg_send![tab.scroll_view, removeFromSuperview];
            }

            let _: () = msg_send![self.close_button, removeFromSuperview];
        }
    }

    fn get_module_tab_data() -> Vec<TabData> {
        vec![
            crate::scripts::tab_data(),
            crate::cheats::tab_data(),
            // todo: Rework settings module to be compatible with improved menu.
        ]
    }

    fn start_channel_polling() -> Sender<MenuMessage> {
        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            type MenuMutex = Mutex<Option<Menu>>;
            let menu: Arc<MenuMutex> = Arc::new(Mutex::new(None));
            unsafe impl Send for Menu {}

            fn do_on_ui_thread<F, T>(f: F) -> T
            where
                F: FnOnce() -> T,
                F: Send + 'static,
                T: Send + 'static,
            {
                dispatch::Queue::main().exec_sync(f)
            }

            loop {
                match receiver.recv().expect("recv() for menu channel failed") {
                    MenuMessage::Show => {
                        log::trace!("Show menu");

                        let menu = Arc::clone(&menu);

                        do_on_ui_thread(move || {
                            let mut menu = menu.lock().unwrap();

                            if menu.is_none() {
                                *menu = Some(Menu::new(Self::get_module_tab_data()));
                                menu.as_ref().unwrap().add_to_window();
                                log::trace!("menu added to window");
                            } else {
                                log::warn!("menu already exists, but was activated again (which should be impossible)");
                            }
                        });
                    }
                    MenuMessage::Hide => {
                        if menu.lock().unwrap().is_some() {
                            let menu = Arc::clone(&menu);

                            do_on_ui_thread(move || {
                                menu.lock().unwrap().take().unwrap().remove();
                            });
                        }
                    }
                }
            }
        });

        sender
    }
}

pub fn initialise() {
    MESSAGE_SENDER
        .set(Mutex::new(Menu::start_channel_polling()))
        .unwrap();

    // Hooks should be installed here.
}

// todo: Re-enable RC calls.

impl Drop for Row {
    fn drop(&mut self) {
        unsafe {
            // let _: () = msg_send![self.detail_label, release];
            // let _: () = msg_send![self.value_label, release];
            // let _: () = msg_send![self.button, release];
        }
    }
}

impl Drop for Tab {
    fn drop(&mut self) {
        unsafe {
            // let _: () = msg_send![self.scroll_view, release];
        }
    }
}

impl Drop for TabButton {
    fn drop(&mut self) {
        unsafe {
            // let _: () = msg_send![self.view, release];
        }
    }
}

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe {
            // let _: () = msg_send![self.close_button, release];
        }
    }
}
