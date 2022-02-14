//! Provides touch information to other modules, and directly controls the showing/hiding
//! of the menu when related to touch events.

use crate::ui::gui::CGRect;
use crate::{call_original, targets};
use cached::proc_macro::cached;
use log::warn;
use objc::{runtime::Object, *};
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
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
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Stage {
    /// The user's finger left the screen.
    Up = 0,

    /// The user has placed their finger on the screen.
    Down = 2,

    /// The user has moved their finger while holding it on the screen.
    Move = 3,
}

#[derive(Clone, Copy)]
struct Pos {
    x: f32,
    y: f32,
}

impl Pos {
    fn dist_squared(self, other: Pos) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

struct Touch {
    start_time: f64,
    start_pos: Pos,
    current_pos: Pos,
}

impl Touch {
    /// Uses the touch movement speed, distance and direction to work out whether it is valid for
    /// summoning the CLEO menu.
    fn is_menu_swipe(&self, time_now: f64) -> bool {
        const MIN_SPEED: f32 = 800.0;
        const MIN_DISTANCE: f32 = 35.0;

        if self.start_time <= 0.0 {
            return false;
        }

        let delta_x = self.current_pos.x - self.start_pos.x;
        let delta_y = self.current_pos.y - self.start_pos.y;
        let delta_time = time_now - self.start_time;

        // todo: Don't root the distance. Adjust the constants to allow for using squared distance.
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

        if distance < MIN_DISTANCE {
            return false;
        }

        let speed = distance / delta_time as f32;

        if speed < MIN_SPEED {
            return false;
        }

        // We only want a downwards swipe, so don't tolerate very much sideways movement.
        let x_is_static = (delta_x / distance).abs() < 0.4;
        let y_is_downwards = delta_y / distance > 0.4;

        x_is_static && y_is_downwards
    }

    /// Calculates the touch zone that this touch falls into. Returns `None` if the calculated
    /// value is not in the range `(0..9)`.
    fn zone(&self) -> Option<usize> {
        let (w, h) = get_screen_size();

        let x = self.current_pos.x / w as f32;
        let y = self.current_pos.y / h as f32;

        fn coordinate_zone(coordinate: f32) -> i64 {
            (coordinate * 3.0).ceil() as i64
        }

        // Minus one at the end to turn the zone number into an index.
        let zone = (coordinate_zone(y) + coordinate_zone(x) * 3 - 3) - 1;

        if (0..9).contains(&zone) {
            Some(zone as usize)
        } else {
            log::trace!(
                "Calculated invalid touch zone {} for ({}, {})",
                zone,
                self.current_pos.x,
                self.current_pos.y
            );

            None
        }
    }
}

pub struct Manager {
    touches: Mutex<Vec<Touch>>,
}

impl Manager {
    /// Returns a reference to the shared touch manager. This method will create the manager if it
    /// doesn't exist already.
    pub fn shared<'mgr>() -> &'mgr Manager {
        static SHARED_MGR: OnceCell<Manager> = OnceCell::new();

        SHARED_MGR.get_or_init(|| Manager {
            touches: Mutex::new(vec![]),
        })
    }

    /// Returns whether or not the user is touching in the specified touch zone.
    pub fn query_zone(&self, zone_idx: usize) -> bool {
        self.touches()
            .iter()
            .any(|touch| touch.zone() == Some(zone_idx))
    }

    /// Locks the touch vector mutex and returns a guard that gives access to the vector. Take care
    /// to not call this twice without dropping the first guard returned.
    fn touches(&self) -> std::sync::MutexGuard<Vec<Touch>> {
        self.touches.lock().unwrap()
    }

    /// Finds the touch closest to the given point. This is useful for tracking touches between
    /// repeated method calls that do not give information to link new touches to old ones.
    fn closest_touch(&self, pos: Pos) -> Option<usize> {
        self.touches()
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                // Find the distance between each touch's position and the target position, then
                // compare them to see which is closer.
                let dist_a = a.current_pos.dist_squared(pos);
                let dist_b = b.current_pos.dist_squared(pos);

                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index)
    }

    /// Shows the menu if the given touch is a menu swipe. Requires ownership of the touch to
    /// ensure that it is not contained within a collection.
    fn handle_swipe(&self, touch: Touch, time: f64) {
        if !touch.is_menu_swipe(time) {
            return;
        }

        if !self.touches().is_empty() {
            log::warn!("Ignoring menu swipe as there are other touches active.");
            return;
        }

        log::info!("Detected menu swipe. Showing menu.");

        // Show the menu.
        super::menu::MenuMessage::Show.send();
    }

    /// Updates the touch manager with information from a new touch event.
    fn proc_event(&self, pos: Pos, time: f64, stage: Stage) {
        match stage {
            Stage::Up => {
                // Find the touch that's ending.
                let index = match self.closest_touch(pos) {
                    Some(index) => index,
                    None => {
                        log::error!("Unable to find touch to terminate.");

                        // todo: Prevent zombie touches accumulating after failed terminations.
                        // Maybe we should have some sort of "lifespan" for touches, whereby if one
                        // has not been updated in a given length of time it is automatically
                        // terminated. That would prevent any "zombie touches" from hanging around.
                        return;
                    }
                };

                // Check if the touch was a menu swipe.
                let touch = self.touches().remove(index);
                self.handle_swipe(touch, time);
            }

            Stage::Down => {
                self.touches().push(Touch {
                    start_time: time,
                    start_pos: pos,
                    current_pos: pos,
                });
            }

            Stage::Move => {
                // Find the version of this touch that we already know about and move it.
                let index = match self.closest_touch(pos) {
                    Some(index) => index,
                    None => {
                        log::error!("Could not find touch to move.");
                        return;
                    }
                };

                self.touches()[index].current_pos = pos;
            }
        }
    }
}

fn proc_touch(x: f32, y: f32, time: f64, force: f32, stage: Stage) {
    Manager::shared().proc_event(Pos { x, y }, time, stage);
    call_original!(targets::process_touch, x, y, time, force, stage);
}

pub fn init() {
    targets::process_touch::install(proc_touch);
}
