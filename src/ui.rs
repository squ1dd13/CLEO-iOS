use crate::{call_original, hook, targets};
use cached::proc_macro::cached;
use lazy_static::lazy_static;
use objc::runtime::Sel;
use runtime::Object;
use std::{
    os::raw::c_long,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

use log::{error, trace, warn};

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
    const MIN_SPEED: f32 = 700.0;
    const MIN_DISTANCE: f32 = 25.0;

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
                            show_script_menu();
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

lazy_static! {
    static ref SHOWING_MENU: AtomicBool = AtomicBool::new(false);
}

static mut MENU: Option<*mut Object> = None;

fn hide_script_menu() {
    if !SHOWING_MENU.load(Ordering::Relaxed) {
        // Menu is not showing.
        return;
    }

    unsafe {
        // Return the game to normal speed.
        *hook::slide::<*mut f32>(0x1007d3b18) = 1.0;

        // Hide the menu if it exists.
        if let Some(menu) = MENU {
            let _: () = msg_send![menu, setHidden: true];
        }
    }

    // Allow new menus to be shown.
    SHOWING_MENU.store(false, Ordering::Relaxed);
}

fn show_script_menu() {
    if SHOWING_MENU.load(Ordering::Relaxed) {
        // Menu is already being shown, so ignore the request.
        return;
    }

    // Make sure we don't show the menu again until this menu is gone.
    SHOWING_MENU.store(true, Ordering::Relaxed);

    unsafe {
        // Slow the game down.
        // todo: Stop game completely while menu is showing.
        *hook::slide::<*mut f32>(0x1007d3b18) = 0.0;

        // If we already have a menu constructed, use that one.
        if let Some(menu) = MENU {
            let _: () = msg_send![menu, setHidden: false];
            return;
        }
    }

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

            let _: () = msg_send![button, setTitle: create_ns_string(*item) forState: /* UIControlStateNormal */ 0 as c_long];

            let handler_class: *const Object = msg_send![class!(IOSReachability), class];
            let handler_sel = sel!(reachabilityWithHostName:);
            let _: () = msg_send![button, addTarget: handler_class action: handler_sel forControlEvents: (1 <<  6) as c_long];

            let _: () = msg_send![scroll_view, addSubview: button];
            let _: () = msg_send![button, release];
        }

        let _: () = msg_send![menu, addSubview: title_label];
        let _: () = msg_send![title_label, release];
        let _: () = msg_send![menu, addSubview: scroll_view];
        let _: () = msg_send![scroll_view, release];
        let _: () = msg_send![window, addSubview: menu];

        // Remember this menu so we can use it in the future.
        MENU = Some(menu);
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
        let button_class: *const Object = msg_send![class!(UIButton), class];
        let is_button: bool = msg_send![hostname, isKindOfClass: button_class];

        if is_button {
            trace!("Button pressed!");

            hide_script_menu();
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
