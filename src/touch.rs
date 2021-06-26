use crate::gui::CGRect;
use crate::{call_original, targets};
use cached::proc_macro::cached;
use lazy_static::lazy_static;
use log::error;
use log::warn;
use objc::{runtime::Object, *};
use std::sync::Mutex;

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
    crate::menu::hide_on_main_thread();

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

                        let is_menu = is_menu_swipe(x1, y1, initial_timestamp, x, y, timestamp);
                        touches.remove(close_index);

                        if is_menu {
                            if !touches.is_empty() {
                                log::info!("Ignoring menu swipe because there are other touches.");
                            } else {
                                log::info!("Detected valid menu swipe.");

                                // Show the menu from a second thread because the touch handlers are typically called
                                //  from the main thread (and we don't want a deadlock).
                                std::thread::spawn(|| crate::menu::show());
                            }
                        }
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

pub fn hook() {
    targets::process_touch::install(process_touch);
}
