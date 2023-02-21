//! Provides touch information to other modules, and directly controls the showing/hiding
//! of the menu when related to touch events.

use crate::meta::gui::CGRect;
use crate::{call_original, targets};
use cached::proc_macro::cached;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::error;
use log::warn;
use objc::{runtime::Object, *};
use std::sync::Mutex;

/// Information about an isolated touch event.
#[derive(Debug, Clone, Copy)]
struct EventInfo {
    /// The horizontal position at which the event ended.
    x: f32,

    /// The vertical position at which the event ended.
    y: f32,

    /// The time at which the event occurred.
    timestamp: f32,
}

/// Different touch events that can take place.
#[derive(Debug, Clone, Copy)]
enum TouchEvent {
    /// The user lifted their finger off the screen.
    Up(EventInfo),

    /// The user placed their finger on the screen.
    Down(EventInfo),

    /// The user kept their finger on the screen, but moved it.
    Move(EventInfo),
}

impl TouchEvent {
    /// Returns the event information regardless of the event type.
    fn info(self) -> EventInfo {
        match self {
            TouchEvent::Up(info) | TouchEvent::Down(info) | TouchEvent::Move(info) => info,
        }
    }
}

/// Areas of the screen that scripts can query for touch information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Zone {
    TopLeft,
    TopMiddle,
    TopRight,

    MiddleLeft,
    Middle,
    MiddleRight,

    BottomLeft,
    BottomMiddle,
    BottomRight,
}

/// Touch event information with zone information included.
#[derive(Debug, Clone, Copy)]
struct ZonedEventInfo {
    /// The raw touch information.
    event_info: EventInfo,

    /// The zone that the event is in, or `None` if the event's position doesn't place it in any
    /// valid zone.
    zone: Option<Zone>,
}

/// Information that can be used to track a touch across multiple events.
#[derive(Debug, Clone, Copy)]
struct TrackedTouch {
    /// The initial "touch down" event information for this touch.
    down_event: ZonedEventInfo,

    /// The most recent event information for this touch, apart from the initial "touch down"
    /// event.
    last_event: Option<ZonedEventInfo>,
}

impl TrackedTouch {
    /// Returns the information about the latest event for this touch.
    fn most_recent_event(&self) -> ZonedEventInfo {
        self.last_event.unwrap_or(self.down_event)
    }

    /// Returns the timestamp of the last change to the touch.
    fn timestamp(&self) -> f32 {
        self.most_recent_event().event_info.timestamp
    }

    /// Returns the current position of the touch.
    fn position(&self) -> (f32, f32) {
        let event = self.most_recent_event().event_info;

        (event.x, event.y)
    }

    /// Returns the zone that this touch is within, or `None` if the touch doesn't fit within a
    /// single zone or any valid zone.
    fn zone(&self) -> Option<Zone> {
        if self.most_recent_event().zone == self.down_event.zone {
            self.down_event.zone
        } else {
            // Don't report any zone information if the touch is currently in a different zone from
            // where it started.
            None
        }
    }
}

/// Returns the width and height of `[[UIScreen mainScreen] nativeBounds]`.
fn uiscreen_size() -> (f64, f64) {
    let cls = class!(UIScreen);

    let bounds: CGRect = unsafe {
        let screen: *mut Object = msg_send![cls, mainScreen];
        msg_send![screen, nativeBounds]
    };

    (bounds.size.width, bounds.size.height)
}

/// Tracks and provides touch information.
struct TouchInterface {
    /// The width of the game window.
    viewport_width: f32,

    /// The height of the game window.
    viewport_height: f32,

    /// The touches currently being tracked. Touches should be removed once they end (with a "touch
    /// up" event).
    tracked_touches: Vec<TrackedTouch>,

    /// Previously tracked touches that have now finished.
    finished_touches: Vec<TrackedTouch>,
}

impl TouchInterface {
    /// Returns the index of the tracked touch that matches best with the event described by
    /// `event_info`, or `None` if there are no realistic candidates.
    fn tracked_index(&self, event_info: EventInfo) -> Option<usize> {
        let (event_x, event_y) = (event_info.x, event_info.y);

        // Obtain an iterator over the indices of the tracked touches and the squared distance of
        // each from the new event's location.
        let indexed_distances = self
            .tracked_touches
            .iter()
            // We're going to filter, so enumerate now so we keep the original index information.
            .enumerate()
            .filter_map(|(index, tracked)| {
                // Only map if the tracked touch was last updated before the new event. Otherwise,
                // we could end up applying two events from the same frame to a single tracked
                // touch.
                (tracked.timestamp() < event_info.timestamp).then_some((index, {
                    let (tracked_x, tracked_y) = tracked.position();

                    // Don't bother square rooting, since the comparison result is the same for
                    // squared and unsquared distances.
                    (tracked_x - event_x).abs() + (tracked_y - event_y).abs()
                }))
            });

        // Return the index of the tracked touch that is closest to the new event.
        indexed_distances
            .min_by(|(_, dist_a), (_, dist_b)| dist_a.total_cmp(dist_b))
            .map(|(index, _)| index)
    }

    /// Updates the tracked touch at `index` with `event_info`.
    fn update_touch(&mut self, index: usize, event_info: ZonedEventInfo) {
        self.tracked_touches[index].last_event = Some(event_info);
    }

    /// Ends the tracked touch at `index`.
    fn end_touch(&mut self, index: usize) {
        self.finished_touches
            .push(self.tracked_touches.swap_remove(index));
    }

    /// Returns the zone that `(x, y)` falls into.
    fn zone_for_position(&self, x: f32, y: f32) -> Option<Zone> {
        // When we divide each coordinate by the applicable screen dimension, we get a fraction of
        // the way across the screen that the coordinate lies. Anything in the left column, for
        // example, will have an X fraction between 0 and 1/3 and anything in the middle column
        // will have an X fraction between 1/3 and 2/3. We multiply these fractions by 3 so we can
        // get just the row/column number, and then floor the value so it is exactly 0, 1 or 2.
        // Invalid positions will not be in (0..=2, 0..=2).

        let horizontal_index = ((x / self.viewport_width) * 3.0).floor() as i8;
        let vertical_index = ((y / self.viewport_height) * 3.0).floor() as i8;

        Some(match (horizontal_index, vertical_index) {
            (0, 0) => Zone::TopLeft,
            (1, 0) => Zone::TopMiddle,
            (2, 0) => Zone::TopRight,

            (0, 1) => Zone::MiddleLeft,
            (1, 1) => Zone::Middle,
            (2, 1) => Zone::MiddleRight,

            (0, 2) => Zone::BottomLeft,
            (1, 2) => Zone::BottomMiddle,
            (2, 2) => Zone::BottomRight,

            _ => return None,
        })
    }

    /// Determines the zone that `event_info` falls into and returns a `ZonedEventInfo` with that
    /// zone.
    fn zoned_info(&self, event_info: EventInfo) -> ZonedEventInfo {
        ZonedEventInfo {
            event_info,
            zone: self.zone_for_position(event_info.x, event_info.y),
        }
    }

    /// Receives and handles `event` in the context of previous touches.
    fn handle_event(&mut self, event: TouchEvent) {
        let zoned_event_info = self.zoned_info(event.info());

        match event {
            TouchEvent::Down(_) => self.tracked_touches.push(TrackedTouch {
                down_event: zoned_event_info,
                last_event: None,
            }),

            TouchEvent::Up(event_info) | TouchEvent::Move(event_info) => {
                let index = self.tracked_index(event_info);

                let index = match index {
                    Some(i) => i,
                    None => {
                        log::warn!(
                            "unable to match event {event:?} to any tracked touch in {:?}",
                            self.tracked_touches
                        );
                        return;
                    }
                };

                self.update_touch(index, zoned_event_info);

                if let TouchEvent::Up(_) = event {
                    self.end_touch(index);
                }
            }
        }
    }

    /// Fetches and stores the viewport size.
    fn fetch_viewport_size(&mut self) {
        let (screen_w, screen_h) = uiscreen_size();

        // The viewport is always in landscape, but the screen size is measured in portrait, so we
        // need to swap the width and height of the screen to get the viewport size
        self.viewport_width = screen_h as f32;
        self.viewport_height = screen_w as f32;
    }
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
