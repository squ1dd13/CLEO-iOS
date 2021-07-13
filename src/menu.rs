//! Provides the CLEO menu, which is the primary way users can interact with the library.
// fixme: This file is too long.

use crate::{call_original, cheats, scripts, targets};
use crate::{gui::*, settings};
use log::{error, trace};
use objc::runtime::Sel;
use objc::{runtime::Object, *};
use std::os::raw::c_long;
use std::sync::Arc;
use std::sync::Mutex;

fn get_scroll_view_delegate() -> *const Object {
    // We know this is only going to be accessed from the main thread.
    static mut SCROLL_DELEGATE: *const Object = std::ptr::null();

    unsafe {
        if SCROLL_DELEGATE.is_null() {
            SCROLL_DELEGATE = msg_send![class!(UIPullDownTableView), new];
        }

        SCROLL_DELEGATE
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ButtonTag {
    index: u32,
    is_tab_button: bool,
    is_cheat_button: bool,
    is_setting_button: bool,
    _unused: u8,
}

impl ButtonTag {
    fn perform_action(&self) {
        if self.is_tab_button {
            trace!("Tab button pressed.");
            MenuAction::queue(MenuAction::SetTab(self.index as u8));
        } else if self.is_cheat_button {
            trace!("Cheat button pressed.");
            cheats::CHEATS[self.index as usize].queue();

            MenuAction::queue(MenuAction::Hide);
        } else if self.is_setting_button {
            trace!("Setting button pressed.");

            settings::with_shared(&mut |options| {
                options.0[self.index as usize].value = !options.0[self.index as usize].value;
            });

            MenuAction::queue(MenuAction::SaveSettings);
            MenuAction::queue(MenuAction::Reload);
        } else {
            if let Some(script) = scripts::MenuInfo::all().get_mut(self.index as usize) {
                script.activate();
            } else {
                error!("Requested script seems to have disappeared.");
            }

            MenuAction::queue(MenuAction::Hide);
        }
    }
}

struct Tab {
    tab_button: *mut Object,
    views: Vec<*mut Object>,
}

struct TabState {
    selected_row: usize,
    touch_scroll_offset: f64,
}

struct MenuState {
    tabs: Vec<TabState>,
    tab_index: usize,
}

impl MenuState {
    fn default() -> MenuState {
        MenuState {
            tabs: vec![
                TabState {
                    selected_row: 0,
                    touch_scroll_offset: 0.0,
                },
                TabState {
                    selected_row: 0,
                    touch_scroll_offset: 0.0,
                },
                TabState {
                    selected_row: 0,
                    touch_scroll_offset: 0.0,
                },
            ],
            tab_index: 0,
        }
    }
}

pub struct Menu {
    width: f64,
    height: f64,

    base_view: *mut Object,

    close_view: *mut Object,

    tabs: Vec<Tab>,

    settings_changed: bool,
    touch_mode: bool,

    state: MenuState,
}

// We keep our Menu instances behind an Arc<Mutex>, so it is safe to pass it between threads.
unsafe impl Send for Menu {}

impl Menu {
    fn new(state: MenuState) -> Menu {
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
            settings_changed: false,
            touch_mode: true,
            state,
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

    fn move_selected_tab_row(&mut self, delta: isize) {
        let tab = &mut self.tabs[self.state.tab_index];

        let row_count: usize = unsafe {
            let subviews: *mut Object = msg_send![tab.views[0], subviews];
            msg_send![subviews, count]
        };

        let selected_row = &mut self.state.tabs[self.state.tab_index].selected_row;
        *selected_row = Self::correct_row_number(*selected_row as isize + delta, row_count);

        self.refresh_rows();
    }

    fn refresh_rows(&mut self) {
        let tab = &self.tabs[self.state.tab_index];

        unsafe {
            let subviews: *mut Object = msg_send![tab.views[0], subviews];
            let count: usize = msg_send![subviews, count];

            let selected_index = self.state.tabs[self.state.tab_index].selected_row;

            let clear: *const Object = msg_send![class!(UIColor), clearColor];
            let background_colour: *const Object = msg_send![class!(UIColor), colorWithRed: 78.0 / 255.0 green: 149.0 / 255.0 blue: 64.0 / 255.0 alpha: 0.3];

            for i in 0..count {
                let row: *mut Object = msg_send![subviews, objectAtIndex: i];

                let background = if !self.touch_mode && i == selected_index {
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
        }
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

            let background_colour: *const Object = msg_send![class!(UIColor), colorWithRed: 1.0 green: 40.0 / 255.0 blue: 46.0 / 255.0 alpha: 0.3];
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
            let _: () = msg_send![scroll_view, setDelegate: get_scroll_view_delegate()];

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
        script: &scripts::MenuInfo,
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
            script.running,
            "Running",
            "Not running",
            script.name.as_str(),
            tag,
            height,
        );

        if script.running {
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
        option: &settings::OptionInfo,
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
        // let injected_scripts: Vec<&'static mut scripts::CleoScript> = scripts::loaded_scripts()
        //     .iter_mut()
        //     .filter(|s| s.injected)
        //     .collect();

        let injected_scripts = scripts::MenuInfo::all();

        unsafe {
            let _: () = msg_send![self.tabs[0].views[0], setContentSize: CGSize {
                width: self.width,
                height: injected_scripts.len() as f64 * self.height * 0.15,
            }];

            let _: () = msg_send![self.tabs[1].views[0], setContentSize: CGSize {
                width: self.width,
                height: cheats::CHEATS.len() as f64 * self.height * 0.25,
            }];
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

        settings::with_shared(&mut |options| {
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

        for (i, tab) in self.tabs.iter().enumerate() {
            let offset = CGPoint {
                x: 0.0,
                y: self.state.tabs[i].touch_scroll_offset,
            };

            unsafe {
                let _: () = msg_send![tab.views[0], setContentOffset: offset animated: false];
            }
        }
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
        self.refresh_rows();
    }

    fn switch_to_tab(&mut self, tab_index: usize) {
        // It's possible that the game crashes after the player launches a script, so we save
        //  any changes to the settings when they change tab. This way, if they change from
        //  the 'Settings' tab to the 'Scripts' tab, their settings are saved before they can
        //  crash the game with a fucked up script.
        self.save_settings_if_needed();

        if self.touch_mode {
            unsafe {
                let offset: CGPoint =
                    msg_send![self.tabs[self.state.tab_index].views[0], contentOffset];
                self.state.tabs[self.state.tab_index].touch_scroll_offset = offset.y;
            }
        }

        self.state.tab_index = tab_index;

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

        self.refresh_rows();

        if self.touch_mode {
            unsafe {
                let offset = CGPoint {
                    x: 0.0,
                    y: self.state.tabs[self.state.tab_index].touch_scroll_offset,
                };

                let _: () = msg_send![
                    self.tabs[self.state.tab_index].views[0],
                    setContentOffset: offset
                    animated: false
                ];
            }
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

            self.switch_to_tab(self.state.tab_index);

            let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
            let window: *mut Object = msg_send![app, keyWindow];

            let _: () = msg_send![window, addSubview: self.base_view];
            let _: () = msg_send![window, addSubview: self.close_view];
        }
    }

    fn show(&mut self, from_controller: bool) {
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

        // If the menu was shown from controller input, we want the input mode
        //  to be controller first.
        self.set_touch_mode(!from_controller);

        if !self.touch_mode {
            // We want the controller selection to show instantly.
            crate::controller::request_update();
        }
    }

    fn save_settings_if_needed(&mut self) {
        if self.settings_changed {
            settings::save();

            self.settings_changed = false;
        }
    }

    fn destroy(mut self) -> MenuState {
        if self.base_view.is_null() {
            trace!("Nothing to do: base_view is already null.");
            return self.state;
        }

        trace!("Saving settings before closing menu...");
        self.save_settings_if_needed();

        trace!("Removing visual components.");
        unsafe {
            let _: () = msg_send![self.base_view, removeFromSuperview];
            let _: () = msg_send![self.close_view, removeFromSuperview];

            let _: () = msg_send![self.close_view, release];

            for (i, tab) in self.tabs.iter().enumerate() {
                let offset: CGPoint = msg_send![tab.views[0], contentOffset];
                self.state.tabs[i].touch_scroll_offset = offset.y;

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
        trace!("Returning game to pre-menu state.");
        crate::hook::slide::<fn()>(0x10026ca6c)();

        self.state
    }

    fn set_touch_mode(&mut self, is_touch: bool) {
        if self.touch_mode == is_touch {
            // Nothing to do.
            return;
        }

        self.touch_mode = is_touch;

        self.refresh_rows();
    }
}

pub fn queue_controller_input(input: &mut crate::controller::ControllerState) {
    if !input.has_input() {
        return;
    } else {
        // There is controller input, so disable touch mode.
        MenuAction::queue(MenuAction::SetTouchMode(false));
    }

    // fixme: The close button leaks through into the game, making the player punch on exit.

    // Back button.
    if input.button_circle != 0 {
        MenuAction::queue(MenuAction::Hide);
        return;
    }

    // Confirm button.
    if input.button_cross != 0 {
        // Perform the action currently selected.
        MenuAction::queue(MenuAction::PerformAction);
    }

    let tab_delta = if input.dpad_left != 0 || input.left_shoulder_2 != 0 {
        -1
    } else if input.dpad_right != 0 || input.right_shoulder_2 != 0 {
        1
    } else {
        0
    };

    MenuAction::queue(MenuAction::ChangeTab(tab_delta));

    let row_delta = if input.dpad_down != 0 {
        1
    } else if input.dpad_up != 0 {
        -1
    } else {
        0
    };

    MenuAction::queue(MenuAction::ChangeRow(row_delta));
}

/*
        This hook allows us to handle button presses by giving us a method with a rough
    signature match for a button handler. Normally, this method has nothing to do with
    buttons. It is +[IOSReachability reachabilityWithHostName:(NSString *)], which creates
    an IOSReachability object.

        UIButton handlers are typically defined on objects created by the programmer.
    However, those objects are Objective-C objects. We don't have the ability to easily
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
            // This function only runs when the button itself was pressed, and not if the
            //  action is triggered by a controller. Therefore, we need to switch the mode
            //  of the menu to touch here because we know the most recent interaction was touch.
            trace!("Queueing stuff...");
            MenuAction::queue(MenuAction::SetTouchMode(true));
            MenuAction::queue(MenuAction::PerformFromTouch(msg_send![hostname, tag]));
            trace!("Done");

            std::ptr::null_mut()
        } else {
            trace!("Normal IOSReachability call.");
            call_original!(targets::button_hack, this_class, sel, hostname)
        }
    }
}

/*
    In order to let us find out when the user interacts with a scroll view, we need a delegate for
    that scroll view. The game contains a class called UIPullDownTableView which acts as its own
    delegate, so we can use a dummy instance of that as our delegate, and hook the methods we want
    in order to detect events.
*/
fn scroll_view_did_end_dragging(
    this: *const Object,
    sel: Sel,
    scroll_view: *mut Object,
    decelerate: bool,
) {
    if this == get_scroll_view_delegate() {
        // One of our own calls.
        MenuAction::queue(MenuAction::SetTouchMode(true));

        return;
    }

    call_original!(targets::end_dragging, this, sel, scroll_view, decelerate);
}

/**
    The MenuAction queue is a way to easily ensure that we don't get deadlocks when two systems
    want to modify the menu. Only the queue system has access to the menu, and everyone else
    can only request that a certain thing be done to the menu by the event queue.
*/
#[derive(Clone, Copy)]
pub enum MenuAction {
    // bool is whether the action comes from a controller.
    Show(bool),
    Toggle(bool),
    Hide,

    PerformAction,
    PerformFromTouch(ButtonTag),

    SetTouchMode(bool),

    // isize values are deltas.
    ChangeRow(isize),
    ChangeTab(isize),

    SetTab(u8),

    Reload,
    SaveSettings,
}

lazy_static::lazy_static! {
    static ref MENU_EVENT_QUEUE: Mutex<Vec<MenuAction>> = Mutex::new(vec![]);
}

impl MenuAction {
    pub fn queue(action: MenuAction) {
        MENU_EVENT_QUEUE.lock().as_mut().unwrap().push(action);
    }

    fn perform(&self) {
        lazy_static::lazy_static! {
           static ref MENU: Arc<Mutex<Option<Menu>>> = Arc::new(Mutex::new(None));
        }

        static mut STATE: Option<MenuState> = None;

        fn with_menu_ref(with: impl Fn(&mut Menu) + Sync) -> Option<()> {
            MENU.lock()
                .unwrap()
                .as_mut()
                .map(|menu| dispatch::Queue::main().exec_sync(|| with(menu)))
        }

        match self {
            MenuAction::Show(from_controller) => {
                let game_state = unsafe { *crate::hook::slide::<*const u32>(0x1006806d0) };

                if game_state != 9 {
                    return;
                }

                dispatch::Queue::main().exec_sync(|| {
                    let state = if let Some(state) = unsafe { STATE.take() } {
                        log::info!("Using existing menu state.");
                        state
                    } else {
                        log::info!("No menu state saved. Default wil be used.");
                        MenuState::default()
                    };

                    let mut locked = MENU.lock().unwrap();
                    *locked = Some(Menu::new(state));
                    locked.as_mut().unwrap().show(*from_controller);
                });
            }

            MenuAction::Toggle(from_controller) => {
                if MENU.lock().unwrap().is_none() {
                    MenuAction::Show(*from_controller).perform();
                } else {
                    MenuAction::Hide.perform();
                }
            }

            MenuAction::Hide => {
                dispatch::Queue::main().exec_sync(|| {
                    if let Some(menu) = MENU.lock().unwrap().take() {
                        unsafe {
                            log::info!("Saving state.");
                            STATE = Some(menu.destroy());
                        }
                    }

                    *MENU.lock().unwrap() = None;
                });
            }

            MenuAction::PerformAction => {
                with_menu_ref(|menu| {
                    let row_index = menu.state.tabs[menu.state.tab_index].selected_row;
                    let tab = &mut menu.tabs[menu.state.tab_index];

                    unsafe {
                        let subviews: *mut Object = msg_send![tab.views[0], subviews];
                        let count: usize = msg_send![subviews, count];

                        if row_index > count {
                            log::error!("row_index > count");
                            return;
                        }

                        let row: *mut Object = msg_send![subviews, objectAtIndex: row_index];
                        let tag: ButtonTag = msg_send![row, tag];
                        tag.perform_action();
                    }
                });
            }

            MenuAction::PerformFromTouch(tag) => {
                tag.perform_action();
            }

            MenuAction::SetTouchMode(touch_mode) => {
                with_menu_ref(|menu| {
                    menu.set_touch_mode(*touch_mode);
                });
            }

            MenuAction::ChangeRow(row_delta) => {
                with_menu_ref(|menu| {
                    menu.move_selected_tab_row(*row_delta);
                    menu.refresh_rows();
                });
            }

            MenuAction::ChangeTab(tab_delta) => {
                with_menu_ref(|menu| {
                    let new_tab_number = menu.state.tab_index as isize + tab_delta;

                    let new_tab_number = if new_tab_number < 0 {
                        2
                    } else if new_tab_number > 2 {
                        0
                    } else {
                        new_tab_number
                    };

                    menu.switch_to_tab(new_tab_number as usize);
                });
            }

            MenuAction::SetTab(index) => {
                with_menu_ref(|menu| {
                    menu.switch_to_tab(*index as usize);
                });
            }

            MenuAction::Reload => {
                with_menu_ref(|menu| {
                    menu.reload();
                });
            }

            MenuAction::SaveSettings => {
                with_menu_ref(|menu| {
                    menu.settings_changed = true;
                    menu.save_settings_if_needed();
                });
            }
        }
    }

    pub fn process_queue() {
        let queue = {
            let mut locked = MENU_EVENT_QUEUE.lock().unwrap();

            // Clone the event queue so we don't need to keep it locked while we're processing the actions.
            // This allows actions to queue new events while we're still iterating over the current actions,
            //  which is helpful if, for example, the menu should be hidden after a script button is pressed.
            let queue = locked.clone();
            locked.clear();

            queue
        };

        for action in queue.iter() {
            action.perform();
        }
    }
}

pub fn hook() {
    targets::button_hack::install(reachability_with_hostname);
    targets::end_dragging::install(scroll_view_did_end_dragging);

    // Start the menu update loop.
    std::thread::spawn(|| loop {
        MenuAction::process_queue();
    });
}
