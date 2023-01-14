//! Provides touch information to other modules, and directly controls the showing/hiding
//! of the menu when related to touch events.

use crate::meta::gui::CGRect;
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

// BUG: `get_zone` doesn't work properly for coordinate pairs with 0 in them.
fn get_zone(x: f32, y: f32) -> Option<i8> {
    let (w, h) = get_screen_size();

    let x = x / w as f32;
    let y = y / h as f32;

    fn coordinate_zone(coordinate: f32) -> i64 {
        (coordinate * 3.0).ceil() as i64
    }

    let zone = coordinate_zone(y) + coordinate_zone(x) * 3 - 3;

    // Sometimes -2 pops up. Other invalid values are probably possible.
    if (1..=9).contains(&zone) {
        Some(zone as i8)
    } else {
        warn!("Bad touch zone {}", zone);
        None
    }
}

struct Pos {
    x: f32,
    y: f32,
}

struct Touch {
    start_time: f64,
    start_pos: Pos,
    current_pos: Pos,
}

lazy_static! {
    static ref TOUCH_ZONES: Mutex<[bool; 9]> = Mutex::new([false; 9]);
    static ref CURRENT_TOUCHES: Mutex<Vec<Touch>> = Mutex::new(Vec::new());
}

pub fn query_zone(zone: usize) -> Option<bool> {
    if !(1..10).contains(&zone) {
        warn!("Touch zone {} does not lie within 1..10.", zone);
        return None;
    }

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

fn is_menu_swipe(touch: &Touch, current_time: f64) -> bool {
    // todo: Present user with combined "swipe sensitivity" option to configure speed and distance together.
    const MIN_SPEED: f32 = 800.0;
    const MIN_DISTANCE: f32 = 35.0;

    if touch.start_time <= 0.0 {
        return false;
    }

    let delta_x = touch.current_pos.x - touch.start_pos.x;
    let delta_y = touch.current_pos.y - touch.start_pos.y;
    let delta_time = current_time - touch.start_time;

    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

    if distance < MIN_DISTANCE {
        return false;
    }

    let speed = distance / delta_time as f32;

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
// fixme: `process_touch` nests too deeply and needs to be broken up into smaller functions.
fn process_touch(x: f32, y: f32, timestamp: f64, force: f32, touch_type: TouchType) {
    // Find the closest touch to the given position that we know about.
    fn find_closest_index(touches: &[Touch], x: f32, y: f32) -> Option<usize> {
        touches
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                // Compare taxicab distance (in order to avoid square rooting).
                let distance_a = (a.current_pos.x - x).abs() + (a.current_pos.y - y).abs();
                let distance_b = (b.current_pos.x - x).abs() + (b.current_pos.y - y).abs();

                distance_a
                    .partial_cmp(&distance_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index)
    }

    // If we have registered a touch, it means the user has touched outside the menu (because
    //  if they touch the menu, we don't get the event). This is a way of dismissing the menu.
    // crate::menu::hide_on_main_thread();
    // MenuAction::queue(MenuAction::Hide);

    //crate::menu::MenuMessage::Hide.send();

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
                        let is_menu = is_menu_swipe(&touches[close_index], timestamp);
                        touches.remove(close_index);

                        if is_menu {
                            if !touches.is_empty() {
                                log::info!("Ignoring menu swipe because there are other touches.");
                            } else {
                                log::info!("Detected valid menu swipe.");
                                crate::meta::menu::MenuMessage::Show.send();
                                // MenuAction::queue(MenuAction::Show(false));
                            }
                        }
                    } else {
                        error!("Unable to find touch to release!");
                    }
                }

                TouchType::Down => {
                    touches.push(Touch {
                        start_time: timestamp,

                        // We only have a single touch event, so the starting position is the same as the current position.
                        start_pos: Pos { x, y },
                        current_pos: Pos { x, y },
                    });
                }

                TouchType::Move => {
                    if let Some(close_index) = find_closest_index(&touches[..], x, y) {
                        touches[close_index].current_pos = Pos { x, y };
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
                        if let Some(zone) = get_zone(touch.current_pos.x, touch.current_pos.y) {
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

pub fn init() {
    log::info!("installing touch hook...");
    targets::process_touch::install(process_touch);
}
