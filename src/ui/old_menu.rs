//! Provides a touch interface and accompanying logic to allow the user to interact with scripts,
//! cheats and settings.

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
};

use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use once_cell::{sync::OnceCell, unsync::Lazy};

use super::gui::{self, create_ns_string, CGPoint, CGRect, CGSize};

pub enum RowDetail {
    Info(String),
    Warning(String),
}

pub trait RowData {
    fn title(&self) -> String;
    fn detail(&self) -> RowDetail;
    fn value(&self) -> &str;
    fn tint(&self) -> Option<(u8, u8, u8)>;

    /// Should return true if the rows in the menu should be reloaded.
    fn handle_tap(&mut self) -> bool;
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

struct TabButton {
    view: *mut Object,
}

#[repr(C)]
#[derive(Debug)]
struct ButtonTag {
    row: i32,
    tab: i8,
    is_close: bool,
    _pad: [u8; 2],
}

impl ButtonTag {
    fn new_tab(index: usize) -> ButtonTag {
        ButtonTag {
            tab: index as i8,
            row: -1,
            is_close: false,
            _pad: [0u8; 2],
        }
    }

    fn new_row(tab: usize, row: usize) -> ButtonTag {
        ButtonTag {
            tab: tab as i8,
            row: row as i32,
            is_close: false,
            _pad: [0u8; 2],
        }
    }

    fn new_close() -> ButtonTag {
        ButtonTag {
            tab: -1,
            row: -1,
            is_close: true,
            _pad: [0u8; 2],
        }
    }
}

static MESSAGE_SENDER: OnceCell<Mutex<Sender<MenuMessage>>> = OnceCell::new();

#[derive(Debug)]
pub enum MenuMessage {
    Show,
    Hide,

    ReloadRows,

    SelectTab(usize),
    HitRow(usize, usize),
}

impl MenuMessage {
    /// Send the message using the default sender. Requires locking the sender mutex, so will block
    /// until the mutex becomes available.
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

    /// Directly send this message using the specified sender.
    pub fn send_with_sender(self, sender: &Sender<Self>) {
        if let Err(err) = sender.send(self) {
            log::error!("Failed to send {:?}", err.0);
        }
    }
}

impl Row {
    fn new(data: Box<dyn RowData>, frame: gui::CGRect) -> Row {
        unsafe {
            let button: *mut Object = msg_send![class!(UIButton), alloc];
            let button: *mut Object = msg_send![button, initWithFrame: frame];

            let _: () = msg_send![button, setContentHorizontalAlignment: 1u64];

            let edge_insets =
                gui::UIEdgeInsets::new(0., frame.size.width * 0.05, frame.size.height * 0.4, 0.);
            let _: () = msg_send![button, setTitleEdgeInsets: edge_insets];

            let label: *mut Object = msg_send![button, titleLabel];
            let font = gui::get_font("ChaletComprime-CologneSixty", ROW_TOP_FONT_SIZE);
            let _: () = msg_send![label, setFont: font];

            let value_frame = CGRect::new(
                frame.size.width * 0.05,
                0.0,
                frame.size.width * 0.9,
                frame.size.height * 0.6,
            )
            .rounded();

            let value_label: *mut Object = msg_send![class!(UILabel), alloc];
            let value_label: *mut Object = msg_send![value_label, initWithFrame: value_frame];
            let _: () = msg_send![value_label, setFont: font];
            let _: () = msg_send![value_label, setTextAlignment: 2u64];

            let detail_frame = CGRect::new(
                frame.size.width * 0.05,
                // 0.5 to move the detail up towards the title. This makes it more obvious that the
                // detail goes with the title, and makes the rows easier to read.
                frame.size.height * 0.5,
                frame.size.width * 0.9,
                frame.size.height * 0.4,
            )
            .rounded();

            let detail_label: *mut Object = msg_send![class!(UILabel), alloc];
            let detail_label: *mut Object = msg_send![detail_label, initWithFrame: detail_frame];

            let font = gui::get_font("ChaletComprime-CologneSixty", ROW_DETAIL_FONT_SIZE);
            let _: () = msg_send![detail_label, setFont: font];
            let _: () = msg_send![detail_label, setAdjustsFontSizeToFitWidth: true];
            let _: () = msg_send![detail_label, setTextAlignment: 0u64];

            let mut row = Row {
                data,
                detail_label,
                value_label,
                button,
            };

            row.load();
            row
        }
    }

    fn load(&mut self) {
        let (detail_text, foreground_colour, background_colour) = match self.data.detail() {
            RowDetail::Info(s) => (
                s,
                gui::colours::white_with_alpha(1., 0.95),
                gui::colours::white_with_alpha(0., 0.),
            ),
            RowDetail::Warning(s) => (
                s,
                gui::colours::get(gui::colours::ORANGE, 1.),
                gui::colours::get(gui::colours::ORANGE, 0.2),
            ),
        };

        let (background_colour, value_colour) = if let Some(tint) = self.data.tint() {
            (gui::colours::get(tint, 0.2), gui::colours::get(tint, 0.95))
        } else {
            (background_colour, gui::colours::white_with_alpha(1., 0.95))
        };

        unsafe {
            let _: () = msg_send![self.button, setBackgroundColor: background_colour];
            let _: () = msg_send![self.button, setTitle: create_ns_string(&self.data.title()) forState: 0u64];
            let _: () = msg_send![self.button, setTitleColor: foreground_colour forState: 0u64];

            let _: () = msg_send![self.value_label, setText: create_ns_string(self.data.value())];
            let _: () = msg_send![self.value_label, setTextColor: value_colour];

            let _: () = msg_send![self.detail_label, setText: create_ns_string(&detail_text)];
            let _: () = msg_send![self.detail_label, setTextColor: foreground_colour];
        }
    }

    fn hit(&mut self) {
        if self.data.handle_tap() {
            MenuMessage::ReloadRows.send();
        }
    }
}

// Previously, we used multipliers for all of the element sizes. This produced good results on
// smaller devices, but on iPads, many things were too big and the menu as a whole looked strange.
// Now we just hardcode the same values for all displays.

const ROW_HEIGHT: f64 = 57.;
const ROW_TOP_FONT_SIZE: f64 = 21.;
const ROW_DETAIL_FONT_SIZE: f64 = 15.;

const TAB_BUTTON_HEIGHT: f64 = 50.;
const TAB_NAME_FONT_SIZE: f64 = 26.;

const CLOSE_BUTTON_HEIGHT: f64 = 30.;
const CLOSE_BTN_FONT_SIZE: f64 = 23.;

// Some elements are still proportional to others. The height of the warning label is proportional
// to the height of the tab view as a whole.
const WARNING_HEIGHT_FRAC: f64 = 0.1;
const WARNING_LBL_FONT_SIZE: f64 = 10.;

struct Tab {
    // We don't use the name or warning, but we may in the future.
    _name: String,
    _warning: Option<String>,
    scroll_view: *mut Object,
    warning_label: Option<*mut Object>,
    rows: Vec<Row>,
}

impl Tab {
    fn new(data: TabData, tab_frame: gui::CGRect, state: TabState) -> Tab {
        let scroll_frame = if data.warning.is_some() {
            // Make the scroll view slightly shorter so we can fit the warning above it.
            CGRect::new(
                tab_frame.origin.x,
                tab_frame.origin.y + tab_frame.size.height * WARNING_HEIGHT_FRAC,
                tab_frame.size.width,
                tab_frame.size.height * (1. - WARNING_HEIGHT_FRAC),
            )
        } else {
            tab_frame
        };

        let scroll_view: *mut Object = unsafe {
            let scroll_view: *mut Object = msg_send![class!(UIScrollView), alloc];
            msg_send![scroll_view, initWithFrame: scroll_frame]
        };

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

        for row in &rows {
            unsafe {
                let _: () = msg_send![row.button, addSubview: row.value_label];
                let _: () = msg_send![row.button, addSubview: row.detail_label];
                let _: () = msg_send![scroll_view, addSubview: row.button];
            }
        }

        let content_size = CGSize {
            width: scroll_frame.size.width,
            height: ROW_HEIGHT * rows.len() as f64,
        };

        unsafe {
            let _: () = msg_send![scroll_view, setContentSize: content_size];

            let content_offset = CGPoint {
                x: 0.,
                y: state.scroll_y,
            };

            let _: () = msg_send![scroll_view, setContentOffset: content_offset animated: false];

            let background = gui::colours::white_with_alpha(0., 0.95);
            let _: () = msg_send![scroll_view, setBackgroundColor: background];
        }

        let warning_label = data.warning.as_ref().map(|warning| unsafe {
            let warning_frame = CGRect::new(
                0.,
                tab_frame.origin.y,
                tab_frame.size.width,
                tab_frame.size.height * WARNING_HEIGHT_FRAC,
            );

            let label: *mut Object = msg_send![class!(UILabel), alloc];
            let label: *mut Object = msg_send![label, initWithFrame: warning_frame];

            let colour = gui::colours::get(gui::colours::ORANGE, 1.);
            let font = gui::get_font("HelveticaNeue-Bold", WARNING_LBL_FONT_SIZE);
            let _: () = msg_send![label, setTextColor: colour];
            let _: () = msg_send![label, setFont: font];
            let _: () = msg_send![label, setText: create_ns_string(warning)];
            let _: () = msg_send![label, setTextAlignment: 1u64];
            let _: () = msg_send![label, setAdjustsFontSizeToFitWidth: true];
            let _: () = msg_send![label, setNumberOfLines: 0u64];

            let colour = gui::colours::get((0, 0, 0), 0.95);
            let _: () = msg_send![label, setBackgroundColor: colour];

            label
        });

        let mut tab = Tab {
            _name: data.name,
            _warning: data.warning,
            scroll_view,
            warning_label,
            rows,
        };

        tab.set_selected(state.selected);
        tab
    }

    fn set_selected(&mut self, selected: bool) {
        unsafe {
            let _: () = msg_send![self.scroll_view, setHidden: !selected];

            if let Some(label) = self.warning_label {
                let _: () = msg_send![label, setHidden: !selected];
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

            let label: *mut Object = msg_send![btn, titleLabel];
            let _: () =
                msg_send![label, setFont: gui::get_font("PricedownGTAVInt", TAB_NAME_FONT_SIZE)];

            add_button_handler(btn, ButtonTag::new_tab(index));

            btn
        };

        TabButton { view }
    }

    fn set_selected(&mut self, selected: bool) {
        let colour_alpha = if selected { 0.95 } else { 0.7 };

        let foreground = gui::colours::white_with_alpha(1., colour_alpha);
        let background = gui::colours::white_with_alpha(0., colour_alpha);

        unsafe {
            let _: () = msg_send![self.view, setTitleColor: foreground forState: 0u64];
            let _: () = msg_send![self.view, setBackgroundColor: background];
        }
    }
}

// bug: The pause-reset system sometimes allows the game to unpause within the pause menu (unless that's a game bug).
fn set_game_timer_paused(want_pause: bool) {
    // Previously, opening and closing the CLEO menu inside the pause menu would unpause the game,
    // because we use the same mechanism as the pause menu for pausing the game (so when CLEO
    // unpaused the game, it undid the pause menu's pausing). To stop this being possible, we use
    // PAUSED_ALREADY to say whether the game was paused when CLEO first tried to pause it (on
    // opening the menu). Then, when CLEO tries to unpause the game, it only happens if the game
    // was not paused already. This means that users can't unpause the game when the game itself
    // wants to be paused.
    static mut PAUSED_ALREADY: bool = false;

    let var_ptr = crate::hook::slide::<*mut bool>(0x1007d3b34);

    if want_pause {
        let currently_paused: bool = crate::hook::get_global(0x1007d3b34);

        if currently_paused {
            unsafe {
                PAUSED_ALREADY = true;
            }

            return;
        }

        unsafe {
            PAUSED_ALREADY = false;
            *var_ptr = true;
        }
    } else {
        // We want to unpause.

        if unsafe { PAUSED_ALREADY } {
            // Don't do anything, because the game was paused when we found it.
            log::info!("Game was paused when found, so not unpausing.");
            return;
        }

        // The game wasn't paused, so we can unpause it.
        unsafe {
            *var_ptr = false;
        }
    }
}

// hack: Using names for TabState structures will not work if the tab changes its contents.
// todo: Remember the selected tab.
static mut TAB_STATES: Lazy<HashMap<String, TabState>> = Lazy::new(HashMap::new);

struct Menu {
    tabs: Vec<Tab>,
    tab_buttons: Vec<TabButton>,
    close_button: *mut Object,
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
        let tabs = tab_data.into_iter().enumerate().map(|(tab_index, data)| {
            let state = unsafe {
                // We remove the existing state from the map if it exists, because we need to move
                // it into the Tab we're about to create. It will be returned to the map when the
                // menu is closed.
                TAB_STATES.remove(&data.name).unwrap_or(TabState {
                    selected: false,
                    scroll_y: 0.,
                })
            };

            let tab = Tab::new(data, tab_frame, state);

            for (row_index, row) in tab.rows.iter().enumerate() {
                add_button_handler(row.button, ButtonTag::new_row(tab_index, row_index));
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
            let _: () = msg_send![
                btn,
                setBackgroundColor: gui::colours::get(gui::colours::RED, 0.35)
            ];

            let label: *mut Object = msg_send![btn, titleLabel];
            let _: () =
                msg_send![label, setFont: gui::get_font("PricedownGTAVInt", CLOSE_BTN_FONT_SIZE)];

            add_button_handler(btn, ButtonTag::new_close());

            btn
        };

        Menu {
            tabs,
            tab_buttons,
            close_button,
        }
    }

    fn add_to_window(&mut self) {
        set_game_timer_paused(true);

        unsafe {
            let application: *mut Object = msg_send![class!(UIApplication), sharedApplication];
            let key_window: *mut Object = msg_send![application, keyWindow];

            for tab_button in &self.tab_buttons {
                let _: () = msg_send![key_window, addSubview: tab_button.view];
            }

            for tab in &self.tabs {
                let _: () = msg_send![key_window, addSubview: tab.scroll_view];

                if let Some(label) = tab.warning_label {
                    let _: () = msg_send![key_window, addSubview: label];
                }
            }

            let _: () = msg_send![key_window, addSubview: self.close_button];
        }

        for i in 0..self.tabs.len() {
            self.tabs[i].set_selected(i == 0);
            self.tab_buttons[i].set_selected(i == 0);
        }
    }

    fn remove(self) {
        unsafe {
            for tab_button in &self.tab_buttons {
                let _: () = msg_send![tab_button.view, removeFromSuperview];
            }

            for tab in &self.tabs {
                let content_offset: CGPoint = msg_send![tab.scroll_view, contentOffset];

                // todo: Add tab selection state to TAB_STATES when menu is removed.
                TAB_STATES.insert(
                    tab._name.clone(),
                    TabState {
                        selected: false,
                        scroll_y: content_offset.y,
                    },
                );

                let _: () = msg_send![tab.scroll_view, removeFromSuperview];

                if let Some(label) = tab.warning_label {
                    let _: () = msg_send![label, removeFromSuperview];
                }
            }

            let _: () = msg_send![self.close_button, removeFromSuperview];
        }

        set_game_timer_paused(false);
    }

    fn get_module_tab_data() -> Vec<TabData> {
        let game_state = unsafe { *crate::hook::slide::<*const u32>(0x1006806d0) };

        // The menu will be automatically created from any TabData structures in the vector that we
        // return, so adding another tab simply requires adding other stuff here.
        if game_state == 9 {
            // In a game, so allow access to all the tabs.
            vec![
                // crate::scripts::tab_data_csi(),
                // crate::scripts::tab_data_csa(),
                crate::old_cheats::tab_data(),
                crate::settings::tab_data(),
            ]
        } else {
            // Not in a game, so only allow access to the settings tab.
            vec![crate::settings::tab_data()]
        }
    }

    fn reload_rows(&mut self) {
        for tab in &mut self.tabs {
            for row in &mut tab.rows {
                row.load();
            }
        }
    }

    fn start_channel_polling() -> Sender<MenuMessage> {
        let (sender, receiver) = mpsc::channel();

        // fixme: start_channel_polling contains lots of prototyping code that needs to be updated.
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
                                menu.as_mut().unwrap().add_to_window();
                                log::trace!("Menu added to window");
                            } else {
                                log::warn!("Menu already exists, but was activated again (which should be impossible)");
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

                    MenuMessage::ReloadRows => {
                        if menu.lock().unwrap().is_some() {
                            let menu = Arc::clone(&menu);

                            do_on_ui_thread(move || {
                                let mut menu = menu.lock().unwrap();
                                let menu = menu.as_mut().unwrap();

                                menu.reload_rows();
                            });
                        }
                    }

                    MenuMessage::SelectTab(tab_index) => {
                        if menu.lock().unwrap().is_some() {
                            let menu = Arc::clone(&menu);

                            do_on_ui_thread(move || {
                                let mut menu = menu.lock().unwrap();
                                let menu = menu.as_mut().unwrap();

                                for (index, tab) in menu.tabs.iter_mut().enumerate() {
                                    tab.set_selected(index == tab_index);
                                }

                                for (index, tab_button) in menu.tab_buttons.iter_mut().enumerate() {
                                    tab_button.set_selected(index == tab_index);
                                }
                            });
                        } else {
                            log::warn!("Tab select message delivered, but menu does not exist!");
                        }
                    }

                    MenuMessage::HitRow(tab_index, row_index) => {
                        if menu.lock().unwrap().is_some() {
                            let menu = Arc::clone(&menu);

                            do_on_ui_thread(move || {
                                let mut menu = menu.lock().unwrap();
                                let menu = menu.as_mut().unwrap();
                                let row = &mut menu.tabs[tab_index].rows[row_index];
                                row.hit();
                            });
                        } else {
                            log::warn!("Tab select message delivered, but menu does not exist!");
                        }
                    }
                }
            }
        });

        sender
    }
}

crate::declare_hook!(
    /// An unused method that we hijack to use as a callback for various UIKit events.
    BUTTON_HACK,
    fn(*const Object, objc::runtime::Sel, *mut Object) -> *mut Object,
    0x1004ebe70
);

fn reachability_with_hostname(
    this_class: *const Object,
    sel: objc::runtime::Sel,
    hostname: *mut Object,
) -> *mut Object {
    unsafe {
        let is_button: bool = msg_send![hostname, isKindOfClass: class!(UIButton)];

        if is_button {
            let tag: ButtonTag = msg_send![hostname, tag];

            if tag.is_close {
                log::trace!("Close button pressed");
                MenuMessage::Hide.send();
                return std::ptr::null_mut();
            }

            if tag.tab == -1 {
                log::error!("tag.tab cannot be -1 when tag.is_close is false");
            } else if tag.row == -1 {
                MenuMessage::SelectTab(tag.tab as usize).send();
            } else {
                MenuMessage::HitRow(tag.tab as usize, tag.row as usize).send();
            }

            std::ptr::null_mut()
        } else {
            BUTTON_HACK.original()(this_class, sel, hostname)
        }
    }
}

fn add_button_handler(button: *mut Object, tag: ButtonTag) {
    let reachability = class!(IOSReachability);
    let selector = sel!(reachabilityWithHostName:);
    let touch_up_inside = (1 << 6) as u64;

    unsafe {
        let _: () = msg_send![button, setTag: tag];
        let _: () = msg_send![button, addTarget: reachability action: selector forControlEvents: touch_up_inside];
    }
}

pub fn init() {
    BUTTON_HACK.install(reachability_with_hostname);

    MESSAGE_SENDER
        .set(Mutex::new(Menu::start_channel_polling()))
        .unwrap();
}

impl Drop for Row {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.detail_label, release];
            let _: () = msg_send![self.value_label, release];
            let _: () = msg_send![self.button, release];
        }
    }
}

impl Drop for Tab {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.scroll_view, release];

            if let Some(label) = self.warning_label {
                let _: () = msg_send![label, release];
            }
        }
    }
}

impl Drop for TabButton {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.view, release];
        }
    }
}

impl Drop for Menu {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.close_button, release];
        }
    }
}
