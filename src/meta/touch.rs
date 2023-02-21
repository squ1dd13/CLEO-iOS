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
use serde::{Deserialize, Serialize};
use std::{
    sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Duration,
};

/// 2D vector type for storing position/offset information.
type Vec2d = vector2d::Vector2D<f32>;

/// Information about an isolated touch event.
#[derive(Debug, Clone, Copy)]
struct EventInfo {
    /// The position of the event on the screen.
    position: Vec2d,

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
pub enum Zone {
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

impl Zone {
    /// Returns the `Zone` that corresponds to the given number, or `None` if the number is
    /// invalid. These numbers are carried over from Android and are only used by scripts; they
    /// have no relevance anywhere else in CLEO.
    pub fn by_number(index: usize) -> Option<Zone> {
        Some(match index {
            1 => Zone::TopLeft,
            2 => Zone::MiddleLeft,
            3 => Zone::BottomLeft,

            4 => Zone::TopMiddle,
            5 => Zone::Middle,
            6 => Zone::BottomMiddle,

            7 => Zone::TopRight,
            8 => Zone::MiddleRight,
            9 => Zone::BottomRight,

            _ => return None,
        })
    }
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

    /// Returns the time at which the touch began.
    fn start_time(&self) -> f32 {
        self.down_event.event_info.timestamp
    }

    /// Returns the timestamp of the last change to the touch.
    fn last_changed(&self) -> f32 {
        self.most_recent_event().event_info.timestamp
    }

    /// Returns the position of the initial "touch down" event.
    fn initial_position(&self) -> Vec2d {
        self.down_event.event_info.position
    }

    /// Returns the latest known position of the touch.
    fn current_position(&self) -> Vec2d {
        self.most_recent_event().event_info.position
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

    /// Returns the distance between the starting position of the touch and the current position.
    fn move_distance(&self) -> f32 {
        (self.initial_position() - self.current_position()).length()
    }

    /// Returns `true` if `self` and `other_touch` were active at the same time for some period.
    fn time_overlaps_with(&self, other_touch: &TrackedTouch) -> bool {
        // If `self` started before `other_touch`, an overlap occurred if `other_touch` started
        // before `self` ended.
        if self.start_time() < other_touch.start_time() {
            other_touch.start_time() < self.last_changed()
        } else {
            // If `other_touch` started before `self`, an overlap occurred if `self` started before
            // `other_touch` ended.
            self.start_time() < other_touch.last_changed()
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
#[derive(Default)]
pub struct TouchInterface {
    /// The width of the game window.
    viewport_width: f32,

    /// The height of the game window.
    viewport_height: f32,

    /// The touches currently being tracked. Touches should be removed once they end (with a "touch
    /// up" event).
    tracked_touches: Vec<TrackedTouch>,

    /// Previously tracked touches that have now finished.
    finished_touches: Vec<TrackedTouch>,

    /// The gesture that the player can use to trigger the CLEO menu.
    menu_gesture: MenuGesture,
}

lazy_static! {
    static ref TOUCH_INTERFACE: RwLock<TouchInterface> = RwLock::new(TouchInterface::default());
}

impl TouchInterface {
    /// Returns a reference to the shared touch interface.
    pub fn shared() -> RwLockReadGuard<'static, TouchInterface> {
        TOUCH_INTERFACE.read().unwrap()
    }

    /// Returns a mutable reference to the shared touch interface.
    pub fn shared_mut() -> RwLockWriteGuard<'static, TouchInterface> {
        TOUCH_INTERFACE.write().unwrap()
    }

    /// Returns `true` if `zone` is currently being touched.
    pub fn is_zone_pressed(&self, zone: Zone) -> bool {
        self.tracked_touches
            .iter()
            .any(|touch| touch.zone() == Some(zone))
    }

    /// Sets the gesture that the player must use to activate the menu.
    pub fn set_menu_gesture(&mut self, gesture: MenuGesture) {
        self.menu_gesture = gesture;
    }

    /// Returns the index of the tracked touch that matches best with the event described by
    /// `event_info`, or `None` if there are no realistic candidates.
    fn tracked_index(&self, event_info: EventInfo) -> Option<usize> {
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
                (tracked.last_changed() < event_info.timestamp).then_some((index, {
                    // Don't bother square rooting the distance here, because the comparison result
                    // is the same whether or not we square root, and sqrt is slow.
                    (event_info.position - tracked.current_position()).length_squared()
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

    /// Returns the zone that `position` falls into.
    fn zone_for_position(&self, position: Vec2d) -> Option<Zone> {
        // When we divide each coordinate by the applicable screen dimension, we get a fraction of
        // the way across the screen that the coordinate lies. Anything in the left column, for
        // example, will have an X fraction between 0 and 1/3 and anything in the middle column
        // will have an X fraction between 1/3 and 2/3. We multiply these fractions by 3 so we can
        // get just the row/column number, and then floor the value so it is exactly 0, 1 or 2.
        // Invalid positions will not be in (0..=2, 0..=2).

        let horizontal_index = ((position.x / self.viewport_width) * 3.0).floor() as i8;
        let vertical_index = ((position.y / self.viewport_height) * 3.0).floor() as i8;

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
            zone: self.zone_for_position(event_info.position),
        }
    }

    /// Receives and handles `event` in the context of previous touches.
    fn handle_event(&mut self, event: TouchEvent) {
        // Use the new timestamp as a reference point for removing old touch information.
        self.remove_stale_touches(event.info().timestamp);

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

    /// Clears all touch information from the interface. This should be used if there is going to
    /// be a break in touch reporting.
    fn clear_touches(&mut self) {
        self.tracked_touches.clear();
        self.finished_touches.clear();
    }

    /// Fetches and stores the viewport size.
    fn fetch_viewport_size(&mut self) {
        let (screen_w, screen_h) = uiscreen_size();

        // The viewport is always in landscape, but the screen size is measured in portrait, so we
        // need to swap the width and height of the screen to get the viewport size
        self.viewport_width = screen_h as f32;
        self.viewport_height = screen_w as f32;
    }

    /// Removes any touches that are too old, relative to the given timestamp, to be relevant.
    fn remove_stale_touches(&mut self, current_timestamp: f32) {
        /// The number of seconds that can pass before we consider a finished touch stale.
        const STALE_CUTOFF: f32 = 2.0;

        self.finished_touches
            .drain_filter(|touch| (current_timestamp - touch.last_changed()) > STALE_CUTOFF);
    }

    /// Checks if the user has carried out the menu trigger gesture. If they have, this will return
    /// `true` after clearing the touch state.
    pub fn check_menu_trigger(&mut self) -> bool {
        let detected = self.menu_gesture.detect(self);

        // If the trigger was detected, clear the touches so that we don't detect it again.
        // bug: Clearing the touches results in any unfinished touches failing to track.
        if detected {
            self.clear_touches();
        }

        detected
    }
}

/// Gestures that can be used to trigger the menu. The player can select which to use in the
/// settings menu.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum MenuGesture {
    /// The classic single-finger downwards swipe.
    OneFingerSwipeDown,

    /// A downwards swipe with two fingers.
    DoubleSwipeDown,

    /// A single tap with two fingers.
    TwoFingerTap,

    /// A single tap with three fingers.
    ThreeFingerTap,
}

impl Default for MenuGesture {
    fn default() -> Self {
        MenuGesture::OneFingerSwipeDown
    }
}

impl MenuGesture {
    /// Returns `true` if the gesture is detected for the given touch interface.
    fn detect(self, interface: &TouchInterface) -> bool {
        match self {
            MenuGesture::OneFingerSwipeDown => Self::detect_single_swipe(interface),
            MenuGesture::DoubleSwipeDown => Self::detect_double_swipe(interface),
            MenuGesture::TwoFingerTap => Self::detect_two_finger_tap(interface),
            MenuGesture::ThreeFingerTap => Self::detect_three_finger_tap(interface),
        }
    }

    fn is_touch_swipe(touch: &TrackedTouch) -> bool {
        const MIN_SPEED: f32 = 800.0;
        const MIN_DISTANCE: f32 = 100.0;

        let displacement = touch.current_position() - touch.initial_position();
        let duration = touch.last_changed() - touch.start_time();

        let distance = displacement.length();

        if distance < MIN_DISTANCE {
            return false;
        }

        let speed = distance / duration;

        if speed < MIN_SPEED {
            return false;
        }

        // Only allow a downwards swipe, so don't tolerate very much sideways movement.
        let x_is_static = (displacement.x / distance).abs() < 0.4;
        let y_is_downwards = displacement.y / distance > 0.4;

        x_is_static && y_is_downwards
    }

    /// Returns `true` if a single swipe gesture is detected in `interface`.
    fn detect_single_swipe(interface: &TouchInterface) -> bool {
        interface.finished_touches.iter().any(Self::is_touch_swipe)
    }

    /// Returns `true` if a two-finger swipe gesture is detected in `interface`.
    fn detect_double_swipe(interface: &TouchInterface) -> bool {
        interface
            .finished_touches
            .iter()
            .tuple_windows::<(_, _)>()
            .any(|(touch_a, touch_b)| {
                Self::is_touch_swipe(touch_a)
                    && Self::is_touch_swipe(touch_b)
                    && touch_a.time_overlaps_with(touch_b)
            })
    }

    fn is_pair_two_finger_tap(touch_a: &TrackedTouch, touch_b: &TrackedTouch) -> bool {
        let gesture_width_squared =
            (touch_a.current_position() - touch_b.current_position()).length_squared();

        const MAX_WIDTH: f32 = 500.0;

        if gesture_width_squared > MAX_WIDTH * MAX_WIDTH {
            return false;
        }

        let a_dist = touch_a.move_distance();
        let b_dist = touch_b.move_distance();

        let dist_difference = (a_dist - b_dist).abs();

        if dist_difference > 5.0 || a_dist > 20.0 {
            return false;
        }

        let a_start = touch_a.start_time();
        let b_start = touch_b.start_time();

        let start_difference = (a_start - b_start).abs();

        if start_difference > 0.1 {
            return false;
        }

        let a_end = touch_a.last_changed();
        let b_end = touch_b.last_changed();

        let end_difference = (a_end - b_end).abs();

        if end_difference > 0.1 {
            return false;
        }

        let a_duration = a_end - a_start;

        if a_duration > 0.5 {
            return false;
        }

        true
    }

    /// Returns `true` if a two-finger tap gesture is detected in `interface`.
    fn detect_two_finger_tap(interface: &TouchInterface) -> bool {
        // Find two subsequently-completed relatively static touch events with short durations and
        // close timestamps.

        interface
            .finished_touches
            .iter()
            .tuple_windows::<(&TrackedTouch, &TrackedTouch)>()
            .any(|(touch_a, touch_b)| Self::is_pair_two_finger_tap(touch_a, touch_b))
    }

    /// Returns `true` if a three-finger tap gesture is detected in `interface`.
    fn detect_three_finger_tap(interface: &TouchInterface) -> bool {
        interface
            .finished_touches
            .iter()
            .tuple_windows::<(&TrackedTouch, &TrackedTouch, &TrackedTouch)>()
            .any(|(a, b, c)| {
                let a_pos = a.current_position();

                // Swap `b` and `c` if necessary to ensure that `b` is closer to `a` than `c`.
                let (b, c) = if (a_pos - b.current_position()).length_squared()
                    < (a_pos - c.current_position()).length_squared()
                {
                    (b, c)
                } else {
                    (c, b)
                };

                Self::is_pair_two_finger_tap(a, b) && Self::is_pair_two_finger_tap(b, c)
            })
    }
}

// Hook the touch handler so we can use touch zones like CLEO Android does.
// todo: Don't pick up touches that have been handled by a non-joypad control.
// fixme: `process_touch` nests too deeply and needs to be broken up into smaller functions.
fn process_touch(x: f32, y: f32, timestamp: f64, force: f32, touch_type: u64) {
    let event_type = match touch_type {
        0 => TouchEvent::Up,
        2 => TouchEvent::Down,
        3 => TouchEvent::Move,

        other => {
            log::warn!("unhandled touch type {other}");
            return;
        }
    };

    let event = event_type(EventInfo {
        position: Vec2d::new(x, y),
        timestamp: timestamp as f32,
    });

    TouchInterface::shared_mut().handle_event(event);

    call_original!(targets::process_touch, x, y, timestamp, force, touch_type);
}

pub fn init() {
    log::info!("installing touch hook...");
    targets::process_touch::install(process_touch);
}
