use crate::{call_original, targets};
use cached::proc_macro::cached;
use lazy_static::lazy_static;
use objc::*;
use runtime::Object;
use std::sync::Mutex;

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

fn log_zone_statuses(zones: &[bool; 9]) {
    fn textual(b: bool) -> &'static str {
        if b {
            " X "
        } else {
            " - "
        }
    }

    trace!(
        "\nZones:\n{}{}{}\n{}{}{}\n{}{}{}\n",
        textual(zones[0]),
        textual(zones[3]),
        textual(zones[6]),
        textual(zones[1]),
        textual(zones[4]),
        textual(zones[7]),
        textual(zones[2]),
        textual(zones[5]),
        textual(zones[8]),
    );
}

// Hook the touch handler so we can use touch zones like CLEO Android does.
fn process_touch(x: f32, y: f32, timestamp: f64, force: f32, touch_type: TouchType) {
    trace!(
        "touch({:?}, {:?}, {:?}, {:?}, {:?})",
        x,
        y,
        timestamp,
        force,
        touch_type
    );

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

                    log_zone_statuses(&touch_zones);
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

pub fn install_hooks() {
    targets::process_touch::install(process_touch);
}
