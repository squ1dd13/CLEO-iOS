use crate::gui::*;
use crate::{call_original, cheats, scripts, targets};
use log::{error, trace};
use objc::runtime::Sel;
use objc::{runtime::Object, *};
use std::os::raw::c_long;
use std::sync::Arc;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref MENU: Arc<Mutex<Option<Menu>>> = Arc::new(Mutex::new(None));
}

#[derive(Debug)]
#[repr(C)]
struct ButtonTag {
    index: u32,
    is_tab_button: bool,
    is_cheat_button: bool,
    is_setting_button: bool,
    _unused: u8,
}

struct Tab {
    tab_button: *mut Object,
    views: Vec<*mut Object>,
}

pub struct Menu {
    width: f64,
    height: f64,

    base_view: *mut Object,

    close_view: *mut Object,

    tabs: Vec<Tab>,

    tab: u8,
    cheat_scroll_point: CGPoint,

    settings_changed: bool,

    controller_row: Option<usize>,
}

// We keep our Menu instances behind an Arc<Mutex>, so it is safe to pass it between threads.
unsafe impl Send for Menu {}

impl Menu {
    fn new() -> Menu {
        let (width, height) = unsafe {
            let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
            let window: *mut Object = msg_send![app, keyWindow];
            let window_bounds: CGRect = msg_send![window, bounds];

            (window_bounds.size.width, window_bounds.size.height * 0.9)
        };

        Menu {
            width,
            height,
            base_view: std::ptr::null_mut(),
            close_view: std::ptr::null_mut(),
            tabs: vec![],
            tab: 0,
            cheat_scroll_point: CGPoint { x: 0.0, y: 0.0 },
            settings_changed: false,
            controller_row: None,
        }
    }

    fn correct_row_number(number: isize, row_count: usize) -> usize {
        let n = if number < 0 {
            row_count - 1
        } else if number as usize >= row_count {
            0
        } else {
            number as usize
        };

        if n != number as usize {
            log::trace!("correcting {} to {}", number, n);
        }

        n
    }

    fn move_selected_row(&mut self, delta: i8) {
        let new_row = if let Some(current_row) = self.controller_row {
            current_row as isize + delta as isize
        } else {
            delta as isize
        };

        self.set_selected_row(new_row);
    }

    fn set_selected_row(&mut self, new_row: isize) {
        let tab = &mut self.tabs[self.tab as usize];

        let new_row = unsafe {
            let subviews: *mut Object = msg_send![tab.views[0], subviews];
            let count: usize = msg_send![subviews, count];

            let new_row_index = Self::correct_row_number(new_row, count as usize);

            let clear: *const Object = msg_send![class!(UIColor), clearColor];
            let background_colour: *const Object = msg_send![class!(UIColor), colorWithRed: 78.0 / 255.0 green: 149.0 / 255.0 blue: 64.0 / 255.0 alpha: 0.3];

            for i in 0..count {
                let row: *mut Object = msg_send![subviews, objectAtIndex: i];

                let background = if i == new_row_index {
                    let row_height = {
                        let row_frame: CGRect = msg_send![row, frame];
                        row_frame.size.height
                    };

                    let top_offset_y = row_height * i as f64;
                    let bottom_offset_y = row_height * (i + 1) as f64;

                    let current_offset: CGPoint = msg_send![tab.views[0], contentOffset];

                    let scroll_height = {
                        let scroll_frame: CGRect = msg_send![tab.views[0], frame];
                        scroll_frame.size.height
                    };

                    let visible_top = current_offset.y;
                    let visible_bottom = current_offset.y + scroll_height;

                    let new_offset_y = if top_offset_y.round() < visible_top.round() {
                        // Place the top of the selected row at the top of the scroll window.
                        top_offset_y
                    } else if bottom_offset_y.round() > visible_bottom.round() {
                        // Place the bottom of the selected row in line with the bottom of the scroll window.
                        current_offset.y + (bottom_offset_y - visible_bottom)
                    } else {
                        // No need to scroll: the selected row is already completely visible.
                        current_offset.y
                    };

                    let _: () = msg_send![tab.views[0], setContentOffset: CGPoint {
                        x: 0.0,
                        y: new_offset_y,
                    } animated: false];

                    background_colour
                } else {
                    clear
                };

                let _: () = msg_send![row, setBackgroundColor: background];
            }

            new_row_index
        };

        self.controller_row = Some(new_row);
    }

    /// Creates the invisible view which holds all the menu's components.
    fn create_base_view(&mut self) {
        unsafe {
            let base = create_with_frame(
                class!(UIView),
                CGRect::new(0.0, 0.0, self.width, self.height),
            );

            let background_colour: *const Object = msg_send![class!(UIColor), clearColor];
            let _: () = msg_send![base, setBackgroundColor: background_colour];

            self.base_view = base;
        }
    }

    /// Create the "Close" button at the bottom of the menu.
    fn create_close_button(&mut self) {
        unsafe {
            let window_height = self.height / 0.9;

            let close: *mut Object = create_label(
                CGRect::new(0.0, self.height, self.width, window_height * 0.1),
                "Close",
                get_font("PricedownGTAVInt", 30.0),
                msg_send![class!(UIColor), whiteColor],
                1,
            );

            let background_colour: *const Object = msg_send![class!(UIColor), colorWithRed: 255.0 / 255.0 green: 40.0 / 255.0 blue: 46.0 / 255.0 alpha: 0.3];
            let _: () = msg_send![close, setBackgroundColor: background_colour];

            // If we disable user interaction, touches can pass through to the game view and the menu will close.
            let _: () = msg_send![close, setUserInteractionEnabled: false];

            self.close_view = close;
        }
    }

    /// Create a tab button (used to allow the user to select the scripts view or the cheats view).
    fn create_single_tab_button(&self, text: &str, is_selected: bool, index: u8) -> *mut Object {
        unsafe {
            let (text_colour, background_colour) = if is_selected {
                let background_colour: *const Object =
                    msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.95];
                let text_colour: *const Object = msg_send![class!(UIColor), whiteColor];

                (text_colour, background_colour)
            } else {
                let background_colour: *const Object =
                    msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.50];
                let text_colour: *const Object =
                    msg_send![class!(UIColor), colorWithWhite: 0.7 alpha: 1.0];

                (text_colour, background_colour)
            };

            let button = create_with_frame(
                class!(UIButton),
                CGRect::new(
                    self.width / 3.0 * index as f64,
                    0.0,
                    self.width / 3.0,
                    self.height * 0.2,
                ),
            );

            let tag = ButtonTag {
                index: index as u32,
                is_tab_button: true,
                is_cheat_button: false,
                is_setting_button: false,
                _unused: 0,
            };

            let _: () = msg_send![button, setTag: tag];
            let _: () = msg_send![button, setTitle: create_ns_string(text) forState: 0u64];
            let _: () = msg_send![button, setTitleColor: text_colour forState: 0u64];
            let _: () = msg_send![button, setBackgroundColor: background_colour];
            let _: () = msg_send![button, addTarget: class!(IOSReachability) 
                                                 action: sel!(reachabilityWithHostName:) 
                                       forControlEvents: /* UIControlEventTouchUpInside */ (1 << 6) as c_long];

            let label: *mut Object = msg_send![button, titleLabel];
            let _: () = msg_send![label, setFont: get_font("PricedownGTAVInt", 30.0)];
            let _: () = msg_send![label, setAdjustsFontSizeToFitWidth: true];
            let _: () = msg_send![label, setTextAlignment: 1 as c_long];

            button
        }
    }

    fn create_tab_buttons(&mut self) {
        self.tabs.push(Tab {
            tab_button: self.create_single_tab_button("Scripts", true, 0),
            views: vec![],
        });
        self.tabs.push(Tab {
            tab_button: self.create_single_tab_button("Cheats", false, 1),
            views: vec![],
        });
        self.tabs.push(Tab {
            tab_button: self.create_single_tab_button("Settings", false, 2),
            views: vec![],
        });
    }

    fn create_single_scroll_view(&self, top_inset: f64) -> *mut Object {
        unsafe {
            let scroll_view = create_with_frame(
                class!(UIScrollView),
                CGRect::new(
                    0.0,
                    top_inset + (self.height * 0.2),
                    self.width,
                    (self.height * 0.8) - top_inset,
                ),
            );

            let background_colour: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.95];
            let _: () = msg_send![scroll_view, setBackgroundColor: background_colour];

            let _: () = msg_send![scroll_view, setBounces: false];
            let _: () = msg_send![scroll_view, setShowsHorizontalScrollIndicator: false];
            let _: () = msg_send![scroll_view, setShowsVerticalScrollIndicator: false];

            scroll_view
        }
    }

    fn create_basic_button(
        &self,
        index: usize,
        height: f64,
        value: bool,
        enabled_str: &str,
        disabled_str: &str,
        title: &str,
        tag: ButtonTag,
        running_height: f64,
    ) -> *mut Object {
        unsafe {
            let button = create_with_frame(
                class!(UIButton),
                CGRect::new(0.0, index as f64 * height, self.width, height),
            );

            let button_label: *mut Object = msg_send![button, titleLabel];
            let font = get_font("ChaletComprime-CologneSixty", 25.0);

            let _: () = msg_send![button_label, setFont: font];

            let _: () = msg_send![button, setTag: tag];
            let _: () = msg_send![button, setContentHorizontalAlignment: 1 as c_long];

            let title = create_ns_string(title);

            let _: () =
                msg_send![button, setTitle: title forState: /* UIControlStateNormal */ 0 as c_long];
            let _: () = msg_send![button, addTarget: class!(IOSReachability) action: sel!(reachabilityWithHostName:) forControlEvents: /* UIControlEventTouchUpInside */ (1 << 6) as c_long];

            // If we need a red in the future, that's 255, 40, 46.
            let text_colour: *const Object = if value {
                msg_send![class!(UIColor), colorWithRed: 78.0 / 255.0 green: 149.0 / 255.0 blue: 64.0 / 255.0 alpha: 1.0]
            } else {
                msg_send![class!(UIColor), whiteColor]
            };

            let _: () = msg_send![button, setTitleColor: text_colour forState: /* UIControlStateNormal */ 0 as c_long];

            let running = create_label(
                CGRect::new(self.width * 0.05, 0.0, self.width * 0.9, running_height),
                if value { enabled_str } else { disabled_str },
                font,
                text_colour,
                2,
            );

            let _: () = msg_send![button, addSubview: running];
            let _: () = msg_send![running, release];

            button
        }
    }

    fn create_single_script_button(
        &self,
        index: usize,
        script: &scripts::Script,
        height: f64,
    ) -> *mut Object {
        let tag = ButtonTag {
            index: index as u32,
            is_tab_button: false,
            is_cheat_button: false,
            is_setting_button: false,
            _unused: 0,
        };

        let button = self.create_basic_button(
            index,
            height,
            script.is_active(),
            "Running",
            "Not running",
            script.display_name.as_str(),
            tag,
            height,
        );

        if script.is_active() {
            unsafe {
                // Show the button as disabled so the user can't fuck up the script by starting it when
                //  it's already active.
                let _: () = msg_send![button, setEnabled: false];
                let _: () = msg_send![button, setAlpha: 0.4];
            }
        }

        unsafe {
            let _: () = msg_send![button, setTitleEdgeInsets: UIEdgeInsets::new(0.0, self.width * 0.05, 0.0, 0.0)];
        }

        button
    }

    fn create_bigger_button(
        &self,
        index: usize,
        title: &str,
        description: &str,
        value: bool,
        enabled_str: &str,
        disabled_str: &str,
        height: f64,
        tag: ButtonTag,
    ) -> *mut Object {
        let button = self.create_basic_button(
            index,
            height,
            value,
            enabled_str,
            disabled_str,
            title,
            tag,
            height * 0.6,
        );

        unsafe {
            let _: () = msg_send![button, setTitleEdgeInsets: UIEdgeInsets::new(0.0, self.width * 0.05, height * 0.4, 0.0)];

            let font = get_font("ChaletComprime-CologneSixty", 20.0);

            let description = create_label(
                CGRect::new(
                    self.width * 0.05,
                    height * 0.6,
                    self.width * 0.9,
                    height * 0.4,
                ),
                description,
                font,
                msg_send![class!(UIColor), whiteColor],
                0,
            );

            let _: () = msg_send![description, sizeToFit];

            let _: () = msg_send![button, addSubview: description];
            let _: () = msg_send![description, release];
        }

        button
    }

    fn create_single_cheat_button(&self, cheat: &cheats::Cheat, height: f64) -> *mut Object {
        self.create_bigger_button(
            cheat.index as usize,
            if cheat.code.is_empty() {
                "<No code>"
            } else {
                cheat.code
            },
            cheat.description,
            cheat.is_active(),
            "Active",
            "Inactive",
            height,
            ButtonTag {
                index: cheat.index as u32,
                is_tab_button: false,
                is_cheat_button: true,
                is_setting_button: false,
                _unused: 0,
            },
        )
    }

    fn create_single_setting_button(
        &self,
        index: usize,
        option: &crate::settings::OptionInfo,
        height: f64,
    ) -> *mut Object {
        self.create_bigger_button(
            index,
            option.title,
            option.description,
            option.value,
            "On",
            "Off",
            height,
            ButtonTag {
                index: index as u32,
                is_tab_button: false,
                is_cheat_button: false,
                is_setting_button: true,
                _unused: 0,
            },
        )
    }

    fn create_scroll_views(&mut self) {
        let scroll_view = self.create_single_scroll_view(0.0);
        self.tabs[0].views.push(scroll_view);

        let scroll_view = self.create_single_scroll_view(self.height * 0.1);
        self.tabs[1].views.push(scroll_view);

        let font = get_font("Helvetica-Bold", 25.0);

        let colour: *mut Object = unsafe { msg_send![class!(UIColor), orangeColor] };

        let warning_label = create_label(
            CGRect::new(self.width * 0.05, 0.0, self.width * 0.9, self.height * 0.1),
            r#"Cheats may break your save. It is strongly advised that you save to a different slot before using any cheats.
Additionally, some – especially those without codes – can crash the game in some situations."#,
            font,
            colour,
            1,
        );

        unsafe {
            let _: () = msg_send![warning_label, setNumberOfLines: 2i64];

            let cheats_warning = create_with_frame(
                class!(UIView),
                CGRect::new(0.0, self.height * 0.2, self.width, self.height * 0.1),
            );

            let background: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.95];
            let _: () = msg_send![cheats_warning, setBackgroundColor: background];

            let _: () = msg_send![cheats_warning, addSubview: warning_label];
            let _: () = msg_send![warning_label, release];

            self.tabs[1].views.push(cheats_warning);
        }

        let scroll_view = self.create_single_scroll_view(0.0);
        self.tabs[2].views.push(scroll_view);

        self.populate_scroll_views();
    }

    fn populate_scroll_views(&mut self) {
        let injected_scripts: Vec<&'static mut scripts::Script> = scripts::loaded_scripts()
            .iter_mut()
            .filter(|s| s.injected)
            .collect();

        unsafe {
            let _: () = msg_send![self.tabs[0].views[0], setContentSize: CGSize {
                width: self.width,
                height: injected_scripts.len() as f64 * self.height * 0.15,
            }];

            let _: () = msg_send![self.tabs[1].views[0], setContentSize: CGSize {
                width: self.width,
                height: cheats::CHEATS.len() as f64 * self.height * 0.25,
            }];

            // There are a lot of cheats, so we save how far the user has scrolled so they don't have to
            //  go back to the same point every time.
            let _: () = msg_send![self.tabs[1].views[0], setContentOffset: self.cheat_scroll_point animated: false];
        }

        for (index, item) in injected_scripts.iter().enumerate() {
            let button = self.create_single_script_button(index, item, self.height * 0.15);

            unsafe {
                let _: () = msg_send![self.tabs[0].views[0], addSubview: button];
                let _: () = msg_send![button, release];
            }
        }

        for cheat in cheats::CHEATS.iter() {
            let button = self.create_single_cheat_button(cheat, self.height * 0.25);

            unsafe {
                let _: () = msg_send![self.tabs[1].views[0], addSubview: button];
                let _: () = msg_send![button, release];
            }
        }

        unsafe {
            let _: () = msg_send![self.tabs[1].views[0], setContentOffset: self.cheat_scroll_point animated: false];
        }

        crate::settings::with_shared(&mut |options| {
            unsafe {
                let _: () = msg_send![self.tabs[2].views[0], setContentSize: CGSize {
                    width: self.width,
                    height: options.0.len() as f64 * self.height * 0.25,
                }];
            }

            for (i, option) in options.0.iter().enumerate() {
                let button = self.create_single_setting_button(i, option, self.height * 0.25);

                unsafe {
                    let _: () = msg_send![self.tabs[2].views[0], addSubview: button];
                    let _: () = msg_send![button, release];
                }
            }
        });
    }

    fn reload(&mut self) {
        // fixme: We should just update the individual buttons instead of reloading everything.

        // Remove all subviews from the scroll views so we can add them again but with newer data.
        for tab in self.tabs.iter() {
            unsafe {
                let subviews: *mut Object = msg_send![tab.views[0], subviews];
                let _: () = msg_send![
                    subviews,
                    makeObjectsPerformSelector: sel!(removeFromSuperview)
                ];
            }
        }

        self.populate_scroll_views();
    }

    fn switch_to_tab(&mut self, tab_index: u8) {
        // It's possible that the game crashes after the player launches a script, so we save
        //  any changes to the settings when they change tab. This way, if they change from
        //  the 'Settings' tab to the 'Scripts' tab, their settings are saved before they can
        //  crash the game with a fucked up script.
        self.save_settings_if_needed();

        let should_set_row = self.tab != tab_index;

        self.tab = tab_index;

        unsafe {
            let selected_background: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.95];
            let selected_foreground: *const Object = msg_send![class!(UIColor), whiteColor];
            let inactive_background: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.50];
            let inactive_foreground: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.7 alpha: 1.0];

            for (i, tab) in self.tabs.iter().enumerate() {
                let is_this_tab = i == tab_index as usize;

                for view in tab.views.iter() {
                    let _: () = msg_send![*view, setHidden: !is_this_tab];
                }

                if is_this_tab {
                    let _: () = msg_send![tab.tab_button, setBackgroundColor: selected_background];
                    let _: () = msg_send![tab.tab_button, setTitleColor: selected_foreground forState: 0u64];
                } else {
                    let _: () = msg_send![tab.tab_button, setBackgroundColor: inactive_background];
                    let _: () = msg_send![tab.tab_button, setTitleColor: inactive_foreground forState: 0u64];
                }
            }
        }

        if should_set_row {
            self.set_selected_row(0);
        }
    }

    fn create_layout(&mut self) {
        self.create_base_view();
        self.create_close_button();
        self.create_tab_buttons();

        unsafe {
            for tab in self.tabs.iter() {
                let _: () = msg_send![self.base_view, addSubview: tab.tab_button];
            }
        }

        self.create_scroll_views();

        unsafe {
            for tab in self.tabs.iter() {
                for view in tab.views.iter() {
                    let _: () = msg_send![self.base_view, addSubview: *view];
                }
            }

            self.switch_to_tab(self.tab);

            let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
            let window: *mut Object = msg_send![app, keyWindow];

            let _: () = msg_send![window, addSubview: self.base_view];
            let _: () = msg_send![window, addSubview: self.close_view];
        }
    }

    fn show(&mut self) {
        if !self.base_view.is_null() {
            return;
        }

        self.settings_changed = false;

        let game_state = unsafe { *crate::hook::slide::<*const u32>(0x1006806d0) };

        // If the game state is 9, it means we are in a game. If we aren't in a game,
        //  we don't want to show the menu.
        if game_state != 9 {
            return;
        }

        crate::hook::slide::<fn()>(0x10026ca5c)();
        self.create_layout();
    }

    fn save_settings_if_needed(&mut self) {
        if self.settings_changed {
            crate::settings::with_shared(&mut |options| {
                options.save();
            });

            self.settings_changed = false;
        }
    }

    fn destroy(&mut self) {
        if self.base_view.is_null() {
            return;
        }

        self.save_settings_if_needed();

        unsafe {
            // Save the cheat scroll distance.
            self.cheat_scroll_point = msg_send![self.tabs[1].views[0], contentOffset];

            let _: () = msg_send![self.base_view, removeFromSuperview];
            let _: () = msg_send![self.close_view, removeFromSuperview];

            let _: () = msg_send![self.close_view, release];

            for tab in self.tabs.iter() {
                for view in tab.views.iter() {
                    let _: () = msg_send![*view, removeFromSuperview];
                    let _: () = msg_send![*view, release];
                }

                let _: () = msg_send![tab.tab_button, removeFromSuperview];
                let _: () = msg_send![tab.tab_button, release];
            }

            self.tabs.clear();
        }

        self.base_view = std::ptr::null_mut();

        // Unpause the game.
        crate::hook::slide::<fn()>(0x10026ca6c)();
    }

    pub fn handle_controller_input(&mut self, input: &crate::controller::ControllerState) {
        let tab_delta = if input.dpad_left != 0 || input.left_shoulder_2 != 0 {
            -1
        } else if input.dpad_right != 0 || input.right_shoulder_2 != 0 {
            1
        } else {
            0
        };

        let row_delta = if input.dpad_down != 0 {
            1
        } else if input.dpad_up != 0 {
            -1
        } else {
            0
        };

        let old_tab_number = self.tab as isize;
        let new_tab_number = old_tab_number + tab_delta;

        let new_tab_number = if new_tab_number < 0 {
            2
        } else if new_tab_number > 2 {
            0
        } else {
            new_tab_number
        };

        self.switch_to_tab(new_tab_number as u8);
        self.move_selected_row(row_delta);
    }
}

/// Obtains a reference to the menu and calls the given closure with that reference,
/// assuming that the closure is valid when run from the current thread.
fn with_menu_on_this_thread<T>(with: impl Fn(&mut Menu) -> T) -> Option<T> {
    let mut locked = MENU.lock();
    let option = locked.as_mut().unwrap();

    if option.is_some() {
        let menu_ref = option.as_mut().unwrap();
        Some(with(menu_ref))
    } else {
        None
    }
}

pub fn with_shared_menu<T: Send>(with: impl Fn(&mut Menu) -> T + Sync) -> Option<T> {
    with_menu_on_this_thread(|menu| dispatch::Queue::main().exec_sync(|| Some(with(menu))))
        .and_then(|v| v)
}

pub fn show() {
    let game_state = unsafe { *crate::hook::slide::<*const u32>(0x1006806d0) };

    if game_state != 9 {
        return;
    }

    if with_shared_menu(|menu| {
        log::trace!("Menu exists already.");
        menu.show();
    })
    .is_none()
    {
        log::trace!("Menu does not yet exist.");
        dispatch::Queue::main().exec_sync(|| {
            // Menu wasn't shown because it doesn't exist, so we need to create it and try again.
            let mut locked = MENU.lock().unwrap();
            *locked = Some(Menu::new());
            locked.as_mut().unwrap().show();
        });
    }
}

pub fn hide_on_main_thread() {
    with_menu_on_this_thread(|menu| {
        menu.destroy();
    });

    *MENU.lock().unwrap() = None;
}

/*
        This hook allows us to handle button presses by giving us a method with a rough
    signature match for a button handler. Normally, this method has nothing to do with
    buttons. It is +[IOSReachability reachabilityWithHostName:(NSString *)], which creates
    an IOSReachability object.

        UIButton handlers are typically defined on objects created by the programmer.
    However, those objects are Objective-C objects; we don't have the ability to easily
    make such objects, especially not by writing our own class out. Given the aim for
    CLEO to be pure Rust, we need to find a workaround. The workaround here is using an
    object that already exists - such as the IOSReachability class - and hook a method
    that has the signature we need. We can keep the original functionality of the method
    by checking the class of the parameter: if we have been given a hostname in the form
    of a UIButton, we know that this is actually a button press; otherwise, it probably
    /is/ a hostname.
*/
fn reachability_with_hostname(
    this_class: *const Object,
    sel: Sel,
    hostname: *mut Object,
) -> *mut Object {
    unsafe {
        let is_button: bool = msg_send![hostname, isKindOfClass: class!(UIButton)];

        if is_button {
            let tag: ButtonTag = msg_send![hostname, tag];

            trace!("tag = {:?}", tag);

            if tag.is_tab_button {
                trace!("Tab button pressed.");

                with_menu_on_this_thread(|menu| {
                    menu.switch_to_tab(tag.index as u8);
                });
            } else if tag.is_cheat_button {
                trace!("Cheat button pressed.");
                cheats::CHEATS[tag.index as usize].queue();

                hide_on_main_thread();
            } else if tag.is_setting_button {
                trace!("Setting button pressed.");

                crate::settings::with_shared(&mut |options| {
                    options.0[tag.index as usize].value = !options.0[tag.index as usize].value;
                });

                with_menu_on_this_thread(|menu| {
                    menu.settings_changed = true;
                    menu.reload();
                });
            } else {
                if let Some(script) = scripts::loaded_scripts()
                    .iter_mut()
                    .filter(|s| s.injected)
                    .nth(tag.index as usize)
                {
                    script.activate();
                } else {
                    error!("Requested script seems to have disappeared.");
                }

                hide_on_main_thread();
            }

            std::ptr::null_mut()
        } else {
            trace!("Normal IOSReachability call.");
            call_original!(targets::button_hack, this_class, sel, hostname)
        }
    }
}

pub fn hook() {
    targets::button_hack::install(reachability_with_hostname);
}
