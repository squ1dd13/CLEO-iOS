use std::collections::HashSet;

use crate::{call_original, targets};
use cached::proc_macro::cached;
use lazy_static::lazy_static;
use objc::*;
use runtime::Object;
use std::sync::Mutex;

use log::{debug, trace, warn};

#[repr(C)]
#[derive(Debug)]
struct CGSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Debug)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Debug)]
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
}

fn log_zone_statuses(zones: &[bool; 9]) {
    fn textual(b: &bool) -> &'static str {
        if *b {
            " X "
        } else {
            " - "
        }
    }

    // let map_str = zones
    //     .rchunks(3)
    //     // .rev()
    //     .map(|chunk| {
    //         chunk
    //             .iter()
    //             .rev()
    //             .map(textual)
    //             .collect::<Vec<&'static str>>()
    //             .join("")
    //     })
    //     .collect::<Vec<String>>()
    //     .join("\n");

    trace!(
        "\nZones:\n{}{}{}\n{}{}{}\n{}{}{}\n",
        textual(&zones[0]),
        textual(&zones[3]),
        textual(&zones[6]),
        textual(&zones[1]),
        textual(&zones[4]),
        textual(&zones[7]),
        textual(&zones[2]),
        textual(&zones[5]),
        textual(&zones[8]),
    );
}

// Hook the touch handler so we can use touch zones like CLEO Android does.
fn process_touch(x: f32, y: f32, timestamp: f64, force: f32, touch_type: TouchType) {
    if matches!(touch_type, TouchType::Up | TouchType::Down) {
        if let Some(zone) = get_zone(x, y) {
            match TOUCH_ZONES.lock() {
                Ok(mut touch_zones) => {
                    trace!("zone = {}", (zone as usize - 1));
                    touch_zones[zone as usize - 1] = touch_type == TouchType::Down;
                    log_zone_statuses(&touch_zones);
                }

                Err(err) => {
                    warn!("Error when trying to lock touch zone mutex: {}", err);
                }
            }
        }
    }

    call_original!(targets::process_touch, x, y, timestamp, force, touch_type);
}

pub fn install_hooks() {
    targets::process_touch::install(process_touch);
}
