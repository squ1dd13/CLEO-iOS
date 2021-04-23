use crate::{call_original, targets};
use cached::proc_macro::cached;
use objc::*;
use runtime::Object;

use log::trace;

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
#[derive(std::fmt::Debug)]
pub enum TouchType {
    Up = 0,
    Down = 2,
    Move = 3,
}

// Hook the touch handler so we can use touch zones like CLEO Android does.
fn process_touch(x: f32, y: f32, timestamp: f64, force: f32, p5: TouchType) {
    log::trace!(
        "process_touch(x: {}, y: {}, {}, {}, {:?})",
        x,
        y,
        timestamp,
        force,
        p5
    );

    let (width, height) = get_screen_size();
    let (norm_x, norm_y) = (x as f64 / width, y as f64 / height);

    trace!("({}, {})", norm_x, norm_y);
    let x_segment = (norm_x * 3.0).ceil() as i64;
    let y_segment = (norm_y * 3.0).ceil() as i64;
    let zone = (y_segment + (3 * x_segment)) - 3;
    trace!("touch zone {}", zone);

    call_original!(targets::process_touch, x, y, timestamp, force, p5);
}

pub fn install_hooks() {
    targets::process_touch::install(process_touch);
}
