use crate::{call_original, cheats, scripts, targets};
use cached::proc_macro::cached;
use lazy_static::lazy_static;
use objc::runtime::Sel;
use objc::{runtime::Object, *};
use std::{
    os::raw::c_long,
    sync::Mutex,
};

use log::{error, trace, warn};

#[repr(C)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[cached]
fn get_screen_size() -> (f64, f64) {
    unsafe {
        let cls = class!(UIScreen);

        let screen: *mut Object = msg_send![cls, mainScreen];
        let bounds: CGRect = msg_send![screen, nativeBounds];

        // Flip width and height because the game is always in landscape.
        (bounds.size.height, bounds.size.width)
    }
}

#[repr(u64)]
#[derive(std::fmt::Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum TouchType {
    Up = 0,
    Down = 2,
    Move = 3,
}

fn get_zone(x: f32, y: f32) -> Option<i8> {
    let (w, h) = get_screen_size();

    let x = x / w as f32;
    let y = y / h as f32;

    fn coordinate_zone(coordinate: f32) -> i64 {
        (coordinate * 3.0).ceil() as i64
    }

    let zone = coordinate_zone(y) + coordinate_zone(x) * 3 - 3;

    // Sometimes -2 pops up. Other invalid values are probably possible.
    if zone >= 1 && zone <= 9 {
        Some(zone as i8)
    } else {
        warn!("Bad touch zone {}", zone);
        None
    }
}

lazy_static! {
    static ref TOUCH_ZONES: Mutex<[bool; 9]> = Mutex::new([false; 9]);
    static ref CURRENT_TOUCHES: Mutex<Vec<((f32, f32, f64), (f32, f32))>> = Mutex::new(Vec::new());
}

pub fn query_zone(zone: usize) -> Option<bool> {
    let zones = TOUCH_ZONES.lock().ok();

    if zones.is_none() {
        warn!("Unable to lock TOUCH_ZONES!");
        return None;
    }

    let zones = zones.unwrap();

    if zone < 10 {
        Some(zones[zone - 1])
    } else {
        warn!("query({})", zone);
        None
    }
}

fn is_menu_swipe(x1: f32, y1: f32, time1: f64, x2: f32, y2: f32, time2: f64) -> bool {
    const MIN_SPEED: f32 = 800.0;
    const MIN_DISTANCE: f32 = 35.0;

    if time1 <= 0.0 {
        return false;
    }

    let delta_x = x2 - x1;
    let delta_y = y2 - y1;
    let delta_t = time2 - time1;

    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

    if distance < MIN_DISTANCE {
        return false;
    }

    let speed = distance / delta_t as f32;

    if speed < MIN_SPEED {
        return false;
    }

    // Only allow a downwards swipe, so don't tolerate very much sideways movement.
    let x_is_static = (delta_x / distance).abs() < 0.4;
    let y_is_downwards = delta_y / distance > 0.4;

    x_is_static && y_is_downwards
}

// Hook the touch handler so we can use touch zones like CLEO Android does.
// todo: Don't pick up touches that have been handled by a non-joypad control.
fn process_touch(x: f32, y: f32, timestamp: f64, force: f32, touch_type: TouchType) {
    // Find the closest touch to the given position that we know about.
    fn find_closest_index(
        touches: &[((f32, f32, f64), (f32, f32))],
        x: f32,
        y: f32,
    ) -> Option<usize> {
        touches
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let a = a.1;
                let b = b.1;

                // Compare taxicab distance (in order to avoid square rooting).
                let distance_a = (a.0 - x).abs() + (a.1 - y).abs();
                let distance_b = (b.0 - x).abs() + (b.1 - y).abs();

                distance_a
                    .partial_cmp(&distance_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index)
    }

    // If we have registered a touch, it means the user has touched outside the menu (because
    //  if they touch the menu, we don't get the event). This is a way of dismissing the menu.
    hide_script_menu();

    /*
        Problem:  We don't know how each touch event is connected to ones we already know about.
                  Therefore, we can't easily track touches between calls to the touch handler,
                  because all we get is the touch position/type/time/force info, and not the
                  previous position of the touch.

        Solution: Keep a record of all the touches we know about (CURRENT_TOUCHES), and every
                  time we receive a new touch up/move event, we modify the closest touch to
                  the event's position. This is reliable because touch up and move events can
                  only happen to existing touches, so we must know about the touch that is
                  being released/moved already, and that touch should be whatever is closest
                  to the modified position. Touch down events are easy, because they simply
                  require us to add a new touch to CURRENT_TOUCHES to be modified later with
                  an up/move signal.
    */
    match CURRENT_TOUCHES.lock() {
        Ok(mut touches) => {
            match touch_type {
                TouchType::Up => {
                    if let Some(close_index) = find_closest_index(&touches[..], x, y) {
                        let (x1, y1, initial_timestamp) = touches[close_index].0;

                        if is_menu_swipe(x1, y1, initial_timestamp, x, y, timestamp) {
                            trace!("Menu swipe");

                            if let Some(menu) = unsafe { MENU.as_mut() } {
                                menu.show();
                            } else {
                                unsafe {
                                    MENU = Some(Menu::new());
                                    MENU.as_mut().unwrap().show();
                                }
                            }
                        }

                        touches.remove(close_index);
                    } else {
                        error!("Unable to find touch to release!");
                    }
                }

                TouchType::Down => {
                    touches.push(((x, y, timestamp), (x, y)));
                }

                TouchType::Move => {
                    if let Some(close_index) = find_closest_index(&touches[..], x, y) {
                        touches[close_index].1 = (x, y);
                    } else {
                        error!("Unable to find touch to move!");
                    }
                }
            }

            // Update the touch zones to match the current touches.
            match TOUCH_ZONES.lock() {
                Ok(mut touch_zones) => {
                    // Clear old zone statuses.
                    for zone_status in touch_zones.iter_mut() {
                        *zone_status = false;
                    }

                    // Trigger the zone for each touch we find.
                    for touch in touches.iter() {
                        if let Some(zone) = get_zone(touch.1 .0, touch.1 .1) {
                            touch_zones[zone as usize - 1] = true;
                        }
                    }
                }

                Err(err) => {
                    error!("Error when trying to lock touch zone mutex: {}", err);
                }
            }
        }

        Err(err) => {
            error!(
                "Unable to lock touch vector mutex! Touch will not be registered. err = {}",
                err
            );
        }
    }

    call_original!(targets::process_touch, x, y, timestamp, force, touch_type);
}

fn create_ns_string(rust_string: &str) -> *const Object {
    unsafe {
        let c_string = std::ffi::CString::new(rust_string).expect("CString::new failed");
        let ns_string: *const Object =
            msg_send![class!(NSString), stringWithUTF8String: c_string.as_ptr()];

        ns_string
    }
}

fn legal_splash_did_load(this: *mut Object, sel: Sel) {
    unsafe {
        // todo? Individually animate our label and show the legal splash after.

        let view: *mut Object = msg_send![this, view];
        let bounds: CGRect = msg_send![view, bounds];
        let label: *mut Object = msg_send![class!(UILabel), alloc];
        let label: *mut Object = msg_send![label, initWithFrame: bounds];

        let text_colour: *const Object = msg_send![class!(UIColor), whiteColor];
        let background_colour: *const Object = msg_send![class!(UIColor), blackColor];
        let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("PricedownGTAVInt") size: 50.0];

        let _: () = msg_send![label, setText: create_ns_string("CLEO")];
        let _: () = msg_send![label, setTextColor: text_colour];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![label, setTextAlignment: /* NSTextAlignmentCenter */ 1 as c_long];
        let _: () = msg_send![label, setBackgroundColor: background_colour];

        call_original!(targets::legal_splash, this, sel);

        let _: () = msg_send![view, addSubview: label];
        let _: () = msg_send![label, release];
    }
}

static mut MENU: Option<Menu> = None;

fn hide_script_menu() {
    unsafe {
        // Remove the menu if it exists.
        if let Some(menu) = MENU.as_mut() {
            menu.hide();
        }
    }
}

fn create_label(
    frame: CGRect,
    text: &str,
    font: *const Object,
    colour: *const Object,
    alignment: u32,
) -> *mut Object {
    unsafe {
        let running: *mut Object = msg_send![class!(UILabel), alloc];
        let label: *mut Object = msg_send![running, initWithFrame: frame];

        let _: () = msg_send![label, setText: create_ns_string(text)];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![label, setTextColor: colour];
        let _: () = msg_send![label, setAdjustsFontSizeToFitWidth: true];
        let _: () = msg_send![label, setTextAlignment: alignment as c_long];

        label
    }
}

#[derive(Debug)]
#[repr(C)]
struct ButtonTag {
    index: u32,
    is_tab_button: bool,
    is_cheat_button: bool,
    _unused: [u8; 2],
}

struct Menu {
    width: f64,
    height: f64,

    base_view: *mut Object,

    scripts_tab_btn: *mut Object,
    scripts_scroll_view: *mut Object,

    cheats_tab_btn: *mut Object,
    cheats_scroll_view: *mut Object,
    cheats_warning: *mut Object,

    tab: u8,
    cheat_scroll_point: CGPoint,
}

impl Menu {
    fn new() -> Menu {
        let (width, height) = unsafe {
            let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
            let window: *mut Object = msg_send![app, keyWindow];
            let window_bounds: CGRect = msg_send![window, bounds];

            (
                window_bounds.size.width * 0.8,
                window_bounds.size.height * 0.8,
            )
        };

        Menu {
            width,
            height,
            base_view: std::ptr::null_mut(),
            scripts_tab_btn: std::ptr::null_mut(),
            scripts_scroll_view: std::ptr::null_mut(),
            cheats_tab_btn: std::ptr::null_mut(),
            cheats_scroll_view: std::ptr::null_mut(),
            cheats_warning: std::ptr::null_mut(),
            tab: 0,
            cheat_scroll_point: CGPoint{ x: 0.0, y: 0.0 },
        }
    }

    fn get_views_for_tab(&mut self, tab: u8) -> Vec<*mut Object> {
        if tab == 0 {
            // Scripts
            vec![self.scripts_scroll_view]
        } else {
            // Cheats
            vec![self.cheats_warning, self.cheats_scroll_view]
        }
    }

    /// Creates the invisible view which holds all the menu's components.
    fn create_base_view(&mut self) {
        unsafe {
            let base: *mut Object = msg_send![class!(UIView), alloc];
            let base: *mut Object = msg_send![base, initWithFrame: CGRect {
                origin: CGPoint {
                    x: ((self.width * 1.25) * 0.1),
                    y: ((self.height * 1.25) * 0.1),
                },
                size: CGSize {
                    width: self.width,
                    height: self.height,
                },
            }];

            let background_colour: *const Object = msg_send![class!(UIColor), clearColor];
            let _: () = msg_send![base, setBackgroundColor: background_colour];

            self.base_view = base;
        }
    }

    /// Create a tab button (used to allow the user to select the scripts view or the cheats view).
    fn create_single_tab_button(&self, text: &str, is_right: bool) -> *mut Object {
        unsafe {
            let frame = CGRect {
                origin: CGPoint {
                    x: if is_right { self.width * 0.5 } else { 0.0 },
                    y: 0.0,
                },
                size: CGSize {
                    width: (self.width * 0.5),
                    height: (self.height * 0.2),
                },
            };

            let (text_colour, background_colour) = if (self.tab == 0) != is_right {
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

            let font: *const Object = msg_send![class!(UIFont), fontWithName: create_ns_string("PricedownGTAVInt") size: 30.0];

            let button: *mut Object = msg_send![class!(UIButton), alloc];
            let button: *mut Object = msg_send![button, initWithFrame: frame];

            let tag = ButtonTag {
                index: if is_right { 1 } else { 0 },
                is_tab_button: true,
                is_cheat_button: false,
                _unused: [0; 2],
            };

            let _: () = msg_send![button, setTag: tag];
            let _: () = msg_send![button, setTitle: create_ns_string(text) forState: 0u64];
            let _: () = msg_send![button, setTitleColor: text_colour forState: 0u64];
            let _: () = msg_send![button, setBackgroundColor: background_colour];
            let _: () = msg_send![button, addTarget: class!(IOSReachability) 
                                                 action: sel!(reachabilityWithHostName:) 
                                       forControlEvents: /* UIControlEventTouchUpInside */ (1 << 6) as c_long];

            let label: *mut Object = msg_send![button, titleLabel];
            let _: () = msg_send![label, setFont: font];
            let _: () = msg_send![label, setAdjustsFontSizeToFitWidth: true];
            let _: () = msg_send![label, setTextAlignment: 1 as c_long];

            button
        }
    }

    fn create_tab_buttons(&mut self) {
        self.scripts_tab_btn = self.create_single_tab_button("Scripts", false);
        self.cheats_tab_btn = self.create_single_tab_button("Cheats", true);
    }

    fn create_single_scroll_view(
        &self,
        top_inset: f64,
        item_height: f64,
        item_count: usize,
    ) -> *mut Object {
        unsafe {
            let scroll_view: *mut Object = msg_send![class!(UIScrollView), alloc];
            let scroll_view: *mut Object = msg_send![scroll_view, initWithFrame: CGRect {
                origin: CGPoint {
                    x: 0.0,
                    y: top_inset + (self.height * 0.2),
                },
                size: CGSize {
                    width: self.width,
                    height: (self.height * 0.8) - top_inset,
                },
            }];

            let background_colour: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.95];
            let _: () = msg_send![scroll_view, setBackgroundColor: background_colour];

            let _: () = msg_send![scroll_view, setBounces: false];
            let _: () = msg_send![scroll_view, setShowsHorizontalScrollIndicator: false];
            let _: () = msg_send![scroll_view, setShowsVerticalScrollIndicator: false];
            let _: () = msg_send![scroll_view, setContentSize: CGSize {
                width: self.width,
                height: item_count as f64 * item_height,
            }];

            scroll_view
        }
    }

    fn create_single_script_button(
        &self,
        index: usize,
        script: &scripts::Script,
        height: f64,
    ) -> *mut Object {
        unsafe {
            let button: *mut Object = msg_send![class!(UIButton), alloc];
            let button: *mut Object = msg_send![button, initWithFrame: CGRect {
                origin: CGPoint {
                    x: self.width * 0.05,
                    y: index as f64 * height,
                },
                size: CGSize {
                    width: self.width * 0.95,
                    height,
                },
            }];

            let button_label: *mut Object = msg_send![button, titleLabel];
            let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("ChaletComprime-CologneSixty") size: 25.0];

            let _: () = msg_send![button_label, setFont: font];

            let tag = ButtonTag {
                index: index as u32,
                is_tab_button: false,
                is_cheat_button: false,
                _unused: [0; 2],
            };

            if std::mem::size_of_val(&tag) != 8 {
                panic!("Size of tag structure must be 8 bytes!");
            }

            trace!("tag = {:?}", tag);

            let _: () = msg_send![button, setTag: tag];
            let _: () = msg_send![button, setContentHorizontalAlignment: 1 as c_long];
            let _: () = msg_send![button, setTitle: create_ns_string(script.display_name.as_str()) forState: /* UIControlStateNormal */ 0 as c_long];

            if !script.is_active() {
                let _: () = msg_send![button, addTarget: class!(IOSReachability) action: sel!(reachabilityWithHostName:) forControlEvents: /* UIControlEventTouchUpInside */ (1 << 6) as c_long];
            } else {
                // Show the button as disabled so the user can't fuck up the script by starting it when
                //  it's already active.
                let _: () = msg_send![button, setEnabled: false];
                let _: () = msg_send![button, setAlpha: 0.4];
            }

            // If we need a red in the future, that's 255, 40, 46.
            let text_colour: *const Object = if script.is_active() {
                msg_send![class!(UIColor), colorWithRed: 78.0 / 255.0 green: 149.0 / 255.0 blue: 64.0 / 255.0 alpha: 1.0]
            } else {
                msg_send![class!(UIColor), whiteColor]
            };

            let _: () = msg_send![button, setTitleColor: text_colour forState: /* UIControlStateNormal */ 0 as c_long];

            let running = create_label(
                CGRect {
                    origin: CGPoint { x: 0.0, y: 0.0 },
                    size: CGSize {
                        width: self.width * 0.9,
                        height,
                    },
                },
                if script.is_active() {
                    "Running"
                } else {
                    "Not running"
                },
                font,
                text_colour,
                2,
            );

            let _: () = msg_send![button, addSubview: running];
            let _: () = msg_send![running, release];

            button
        }
    }

    fn create_single_cheat_button(&self, cheat: &cheats::Cheat, height: f64) -> *mut Object {
        unsafe {
            let index = cheat.index as usize;

            let button: *mut Object = msg_send![class!(UIButton), alloc];
            let button: *mut Object = msg_send![button, initWithFrame: CGRect {
                origin: CGPoint {
                    x: self.width * 0.05,
                    y: index as f64 * height,
                },
                size: CGSize {
                    width: self.width * 0.95,
                    height,
                },
            }];

            let button_label: *mut Object = msg_send![button, titleLabel];
            let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("ChaletComprime-CologneSixty") size: 25.0];

            let _: () = msg_send![button_label, setFont: font];

            let tag = ButtonTag {
                index: index as u32,
                is_tab_button: false,
                is_cheat_button: true,
                _unused: [0; 2],
            };

            if std::mem::size_of_val(&tag) != 8 {
                panic!("Size of tag structure must be 8 bytes!");
            }

            trace!("tag = {:?}", tag);

            let _: () = msg_send![button, setTag: tag];
            let _: () = msg_send![button, setContentHorizontalAlignment: 1 as c_long];

            let title = create_ns_string(if cheat.code.is_empty() {
                "<No code>"
            } else {
                cheat.code
            });

            let _: () =
                msg_send![button, setTitle: title forState: /* UIControlStateNormal */ 0 as c_long];

            #[repr(C)]
            struct UIEdgeInsets {
                top: f64,
                left: f64,
                bottom: f64,
                right: f64,
            }

            let insets = UIEdgeInsets {
                top: 0.0,
                left: 0.0,
                bottom: height * 0.4,
                right: 0.0,
            };

            let _: () = msg_send![button, setTitleEdgeInsets: insets];
            let _: () = msg_send![button, addTarget: class!(IOSReachability) action: sel!(reachabilityWithHostName:) forControlEvents: /* UIControlEventTouchUpInside */ (1 << 6) as c_long];

            // If we need a red in the future, that's 255, 40, 46.
            let text_colour: *const Object = if cheat.is_active() {
                msg_send![class!(UIColor), colorWithRed: 78.0 / 255.0 green: 149.0 / 255.0 blue: 64.0 / 255.0 alpha: 1.0]
            } else {
                msg_send![class!(UIColor), whiteColor]
            };

            let _: () = msg_send![button, setTitleColor: text_colour forState: /* UIControlStateNormal */ 0 as c_long];

            let running = create_label(
                CGRect {
                    origin: CGPoint { x: 0.0, y: 0.0 },
                    size: CGSize {
                        width: self.width * 0.9,
                        height: height * 0.6,
                    },
                },
                if cheat.is_active() {
                    "Active"
                } else {
                    "Inactive"
                },
                font,
                text_colour,
                2,
            );

            let _: () = msg_send![button, addSubview: running];
            let _: () = msg_send![running, release];

            let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("ChaletComprime-CologneSixty") size: 20.0];

            let description = create_label(
                CGRect {
                    origin: CGPoint {
                        x: 0.0,
                        y: height * 0.6,
                    },
                    size: CGSize {
                        width: self.width * 0.9,
                        height: height * 0.4,
                    },
                },
                cheat.description,
                font,
                msg_send![class!(UIColor), whiteColor],
                0,
            );

            let _: () = msg_send![description, sizeToFit];

            let _: () = msg_send![button, addSubview: description];
            let _: () = msg_send![description, release];

            button
        }
    }

    fn create_scroll_views(&mut self) {
        let injected_scripts: Vec<&'static mut scripts::Script> = scripts::loaded_scripts()
            .iter_mut()
            .filter(|s| s.injected)
            .collect();

        self.scripts_scroll_view =
            self.create_single_scroll_view(0.0, self.height * 0.15, injected_scripts.len());

        for (index, item) in injected_scripts.iter().enumerate() {
            let button = self.create_single_script_button(index, item, self.height * 0.15);

            unsafe {
                let _: () = msg_send![self.scripts_scroll_view, addSubview: button];
                let _: () = msg_send![button, release];
            }
        }

        self.cheats_scroll_view = self.create_single_scroll_view(
            self.height * 0.1,
            self.height * 0.25,
            cheats::CHEATS.len(),
        );

        // There are a lot of cheats, so we save how far the user has scrolled so they don't have to
        //  go back to the same point every time.
        unsafe {
            let _: () = msg_send![self.cheats_scroll_view, setContentOffset: self.cheat_scroll_point animated: false];
        }

        let font: *mut Object = unsafe {
            msg_send![class!(UIFont), fontWithName: create_ns_string("Helvetica-Bold") size: 25.0]
        };

        let colour: *mut Object = unsafe { msg_send![class!(UIColor), orangeColor] };

        let warning_label = create_label(
            CGRect {
                origin: CGPoint {
                    x: self.width * 0.05,
                    y: 0.0,
                },
                size: CGSize {
                    width: self.width * 0.9,
                    height: self.height * 0.1,
                },
            },
            r#"Cheats may break your save. It is strongly advised that you save to a different slot before using any cheats.
Additionally, some cheats (especially those without codes) may crash your game in some situations."#,
            font,
            colour,
            1,
        );

        unsafe {
            let _: () = msg_send![warning_label, setNumberOfLines: 2i64];

            self.cheats_warning = msg_send![class!(UIView), alloc];
            self.cheats_warning = msg_send![self.cheats_warning, initWithFrame:CGRect {
                origin: CGPoint {
                    x: 0.0,
                    y: self.height * 0.2,
                },
                size: CGSize {
                    width: self.width,
                    height: self.height * 0.1,
                },
            }];

            let background: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.95];
            let _: () = msg_send![self.cheats_warning, setBackgroundColor: background];

            let _: () = msg_send![self.cheats_warning, addSubview: warning_label];
            let _: () = msg_send![warning_label, release];
        }

        for cheat in cheats::CHEATS.iter() {
            let button = self.create_single_cheat_button(cheat, self.height * 0.25);

            unsafe {
                let _: () = msg_send![self.cheats_scroll_view, addSubview: button];
                let _: () = msg_send![button, release];
            }
        }
    }

    fn switch_to_tab(&mut self, tab: u8) {
        self.tab = tab;

        unsafe {
            for view in self
                .get_views_for_tab(if self.tab == 0 { 1 } else { 0 })
                .iter()
            {
                let _: () = msg_send![*view, setHidden: true];
            }

            for view in self.get_views_for_tab(self.tab).iter() {
                let _: () = msg_send![*view, setHidden: false];
            }
        }

        unsafe {
            let selected_background: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.95];
            let selected_foreground: *const Object = msg_send![class!(UIColor), whiteColor];
            let inactive_background: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.50];
            let inactive_foreground: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.7 alpha: 1.0];

            let (selected, inactive) = if self.tab == 0 {
                (self.scripts_tab_btn, self.cheats_tab_btn)
            } else {
                (self.cheats_tab_btn, self.scripts_tab_btn)
            };

            let _: () = msg_send![selected, setBackgroundColor: selected_background];
            let _: () = msg_send![inactive, setBackgroundColor: inactive_background];
            let _: () = msg_send![selected, setTitleColor: selected_foreground forState: 0u64];
            let _: () = msg_send![inactive, setTitleColor: inactive_foreground forState: 0u64];
        }
    }

    fn create_layout(&mut self) {
        self.create_base_view();
        self.create_tab_buttons();

        unsafe {
            let _: () = msg_send![self.base_view, addSubview: self.scripts_tab_btn];
            let _: () = msg_send![self.base_view, addSubview: self.cheats_tab_btn];
        }

        self.create_scroll_views();

        unsafe {
            let _: () = msg_send![self.base_view, addSubview: self.scripts_scroll_view];
            let _: () = msg_send![self.base_view, addSubview: self.cheats_warning];
            let _: () = msg_send![self.base_view, addSubview: self.cheats_scroll_view];

            self.switch_to_tab(self.tab);

            let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
            let window: *mut Object = msg_send![app, keyWindow];

            let _: () = msg_send![window, addSubview: self.base_view];
        }
    }

    fn show(&mut self) {
        if !self.base_view.is_null() {
            return;
        }

        crate::hook::slide::<fn()>(0x10026ca5c)();
        self.create_layout();
    }

    fn hide(&mut self) {
        if self.base_view.is_null() {
            return;
        }

        unsafe {
            // Save the cheat scroll distance.
            self.cheat_scroll_point = msg_send![self.cheats_scroll_view, contentOffset];

            let _: () = msg_send![self.base_view, removeFromSuperview];

            let _: () = msg_send![self.scripts_tab_btn, release];
            let _: () = msg_send![self.cheats_tab_btn, release];
            let _: () = msg_send![self.scripts_scroll_view, release];
            let _: () = msg_send![self.cheats_warning, release];
            let _: () = msg_send![self.cheats_scroll_view, release];
            let _: () = msg_send![self.base_view, release];
        }

        self.base_view = std::ptr::null_mut();

        crate::hook::slide::<fn()>(0x10026ca6c)();
    }
}

/*
        This hook allows us to handle button presses by giving us a method with a rough
    signature match for a button handler. Normally, this method has nothing to do with
    buttons: it is +[IOSReachability reachabilityWithHostName:(NSString *)], which creates
    an IOSReachability object.

        UIButton handlers are typically defined on objects created by the programmer.
    However, those objects are Objective-C objects; we don't have the ability to easily
    make such objects, especially not by writing our own class out. Given the aim for
    CLEO to be pure Rust, need to find a workaround. The workaround here is using an
    object that already exists - such as the IOSReachability class - and hook a method
    that has the signature we need. We can keep the original functionality of the method
    by checking the class of the parameter: if we have been given a hostname in the form
    of a UIButton, we know that this is actually a button press; otherwise, it probably
    is a hostname.
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

                if let Some(menu) = MENU.as_mut() {
                    menu.switch_to_tab(tag.index as u8);
                }
            } else if tag.is_cheat_button {
                trace!("Cheat button pressed.");
                cheats::CHEATS[tag.index as usize].run();
                
                hide_script_menu();
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

                hide_script_menu();
            }

            std::ptr::null_mut()
        } else {
            trace!("Normal IOSReachability call.");
            call_original!(targets::button_hack, this_class, sel, hostname)
        }
    }
}

/*
        This hook fixes a bug in the game where -[SCAppDelegate persistentStoreCoordinator]
    calls -[SCAppDelegate managedObjectModel], which crashes the game because it attempts
    to call -[NSManagedObjectModel initWithContentsOfURL:] with a nil URL that it gets
    from calling -[NSBundle URLForResource:withExtension:] for the resource "gtasa.momd",
    which does not exist.

        The easiest way to fix this issue is to hook -[SCAppDelegate persistentStoreCoordinator]
    to always return a null pointer, since the method that calls it,
    -[SCAppDelegate managedObjectContext], checks the return value to see if it is null
    before attempting to do anything with it. This seems to be a fairly robust fix since
    everything further up the callstack has decent checks in place to prevent issues with
    null pointers.

        These events only occur when the app is terminated, so the crash
    is fairly insignificant, but on a jailbroken device with crash reporting tools installed,
    the constant crash reports can get annoying.
*/
fn persistent_store_coordinator(_this: *mut Object, _sel: Sel) -> *const Object {
    trace!("-[SCAppDelegate persistentStoreCoordinator] called. Returning null to prevent crash.");
    std::ptr::null()
}

pub fn hook() {
    targets::process_touch::install(process_touch);
    targets::legal_splash::install(legal_splash_did_load);
    targets::store_crash_fix::install(persistent_store_coordinator);
    targets::button_hack::install(reachability_with_hostname);
}
