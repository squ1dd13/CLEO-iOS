use std::sync::Mutex;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ControllerState {
    left_stick_x: i16,
    left_stick_y: i16,

    right_stick_x: i16,
    right_stick_y: i16,

    left_shoulder_1: i16,
    left_shoulder_2: i16,

    right_shoulder_1: i16,
    right_shoulder_2: i16,

    d_pad_up: i16,
    d_pad_down: i16,
    d_pad_left: i16,
    d_pad_right: i16,

    start: i16,
    select: i16,

    button_square: i16,
    button_triangle: i16,
    button_cross: i16,
    button_circle: i16,

    shock_button_l: i16,
    shock_button_r: i16,

    chat_indicated: i16,
    ped_walk: i16,
    vehicle_mouse_look: i16,
    radio_track_skip: i16,
}

lazy_static::lazy_static! {
    static ref CONTROLLER_STATE: Mutex<Option<ControllerState>> = Mutex::new(None);
}

pub fn with_shared_state<T>(with: &mut impl FnMut(&Option<ControllerState>) -> T) -> T {
    let mut locked = CONTROLLER_STATE.lock();
    with(locked.as_mut().unwrap())
}

fn update_pads() {
    crate::call_original!(crate::targets::update_pads);

    // The first two fields are ControllerStates, so we can just pretend we have an array
    //  of ControllerStates and use that to access the NewState field.
    let state = unsafe {
        let ptr: *const ControllerState = crate::hook::slide(0x1007baf5c);
        *ptr.offset(1)
    };

    let locked = CONTROLLER_STATE.lock();
    *locked.unwrap() = Some(state);
}

pub fn hook() {
    crate::targets::update_pads::install(update_pads);
}
