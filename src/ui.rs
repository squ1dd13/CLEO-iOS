use crate::{call_original, targets};
use cached::proc_macro::cached;
use lazy_static::lazy_static;
use objc::runtime::Sel;
use runtime::Object;
use std::{os::raw::c_long, sync::Mutex};

use log::{error, warn};

#[repr(C)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
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
    static ref CURRENT_TOUCHES: Mutex<Vec<(f32, f32)>> = Mutex::new(Vec::new());
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

// Hook the touch handler so we can use touch zones like CLEO Android does.
fn process_touch(x: f32, y: f32, timestamp: f64, force: f32, touch_type: TouchType) {
    // Find the closest touch to the given position that we know about.
    fn find_closest_index(touches: &[(f32, f32)], x: f32, y: f32) -> Option<usize> {
        touches
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                // Compare taxicab distance (in order to avoid square rooting).
                let distance_a = (a.0 - x).abs() + (a.1 - y).abs();
                let distance_b = (b.0 - x).abs() + (b.1 - y).abs();

                distance_a
                    .partial_cmp(&distance_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index)
    }

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
                        touches.remove(close_index);
                    } else {
                        error!("Unable to find touch to release!");
                    }
                }

                TouchType::Down => {
                    touches.push((x, y));
                }

                TouchType::Move => {
                    if let Some(close_index) = find_closest_index(&touches[..], x, y) {
                        touches[close_index] = (x, y);
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
                        if let Some(zone) = get_zone(touch.0, touch.1) {
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

use objc::*;

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
        // todo: Check if we need to add any reference counting calls here.
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

fn _show_script_menu() {
    unsafe {
        let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
        let window: *mut Object = msg_send![app, keyWindow];
        let window_bounds: CGRect = msg_send![window, bounds];

        let menu_width = window_bounds.size.width * 0.7;
        let menu_height = window_bounds.size.height * 0.7;

        let menu: *mut Object = msg_send![class!(UIView), alloc];
        let menu: *mut Object = msg_send![menu, initWithFrame: CGRect {
            origin: CGPoint {
                x: (window_bounds.size.width * 0.15).round(),
                y: (window_bounds.size.height * 0.15).round(),
            },
            size: CGSize {
                width: menu_width,
                height: menu_height,
            },
        }];

        let background_colour: *const Object =
            msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.95];
        let _: () = msg_send![menu, setBackgroundColor: background_colour];

        let title_label: *mut Object = msg_send![class!(UILabel), alloc];
        let title_label: *mut Object = msg_send![title_label, initWithFrame: CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: menu_width,
                height: (menu_height * 0.2).round(),
            },
        }];

        let text_colour: *const Object = msg_send![class!(UIColor), whiteColor];
        let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("PricedownGTAVInt") size: 35.0];

        let _: () = msg_send![title_label, setText: create_ns_string("Scripts")];
        let _: () = msg_send![title_label, setFont: font];
        let _: () = msg_send![title_label, setTextColor: text_colour];
        let _: () = msg_send![title_label, setAdjustsFontSizeToFitWidth: true];
        let _: () = msg_send![title_label, setTextAlignment: 1 as c_long];

        let scroll_view: *mut Object = msg_send![class!(UIScrollView), alloc];
        let scroll_view: *mut Object = msg_send![scroll_view, initWithFrame: CGRect {
            origin: CGPoint {
                x: 0.0,
                y: (menu_height * 0.2).round(),
            },
            size: CGSize {
                width: menu_width,
                height: (menu_height * 0.8).round(),
            },
        }];

        let button_height = (menu_height * 0.15).round();
        let sample_data = &[
            "Item 1", "Item 2", "Item 3", "Item 4", "Item 5", "Item 6", "Item 7", "Item 8",
            "Item 9", "Item 10",
        ];

        let _: () = msg_send![scroll_view, setBounces: false];
        let _: () = msg_send![scroll_view, setShowsHorizontalScrollIndicator: false];
        let _: () = msg_send![scroll_view, setShowsVerticalScrollIndicator: false];
        let _: () = msg_send![scroll_view, setContentSize: CGSize {
            width: menu_width,
            height: sample_data.len() as f64 * button_height,
        }];

        // Add the entries to the scroll view.
        for (index, item) in sample_data.iter().enumerate() {
            let button: *mut Object = msg_send![class!(UIButton), alloc];
            let button: *mut Object = msg_send![button, initWithFrame: CGRect {
                origin: CGPoint {
                    x: 0.0,
                    y: index as f64 * button_height,
                },
                size: CGSize {
                    width: menu_width,
                    height: button_height,
                },
            }];

            let button_label: *mut Object = msg_send![button, titleLabel];
            let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("ChaletComprime-CologneSixty") size: 25.0];

            let _: () = msg_send![button_label, setFont: font];

            let _: () = msg_send![button, setTitle: create_ns_string(*item) forState: /* UIControlStateNormal */ 0];

            // todo: Touch handler.

            let _: () = msg_send![scroll_view, addSubview: button];
            let _: () = msg_send![button, release];
        }

        let _: () = msg_send![menu, addSubview: title_label];
        let _: () = msg_send![title_label, release];
        let _: () = msg_send![menu, addSubview: scroll_view];
        let _: () = msg_send![scroll_view, release];
        let _: () = msg_send![window, addSubview: menu];
        let _: () = msg_send![menu, release];
    }
}

pub fn hook() {
    targets::process_touch::install(process_touch);
    targets::legal_splash::install(legal_splash_did_load);
}
