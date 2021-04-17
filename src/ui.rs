use crate::{call_original, get_log, targets};
use objc::*;
use runtime::{Object, BOOL};

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

// Hook the touch handler so we can use touch zones like CLEO Android does.
// We flip the X and Y coordinates because the game is always in landscape.
// The game's function uses 'x, y, timestamp, force, p5', but we use 'y, x, ...'.
extern "C" fn process_touch(y: f32, x: f32, timestamp: f64, force: f32, p5: f32) {
    get_log().normal(format!(
        "process_touch(x: {}, y: {}, {}, {}, {})",
        x, y, timestamp, force, p5
    ));

    unsafe {
        let cls = class!(UIScreen);

        let screen: *mut Object = msg_send![cls, mainScreen];
        let bounds: CGRect = msg_send![screen, nativeBounds];

        get_log().normal(format!("screen bounds: {:#?}", bounds));

        let (norm_x, norm_y) = (x / bounds.size.width as f32, y / bounds.size.height as f32);
        get_log().normal(format!("({}, {})", norm_x, norm_y));
    }

    call_original!(targets::process_touch, x, y, timestamp, force, p5);
}

pub fn install_hooks() {
    targets::process_touch::install(process_touch);
}
