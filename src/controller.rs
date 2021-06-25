use std::sync::atomic::AtomicU16;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControllerState {
    pub left_stick_x: i16,
    pub left_stick_y: i16,

    pub right_stick_x: i16,
    pub right_stick_y: i16,

    pub left_shoulder_1: i16,
    pub left_shoulder_2: i16,

    pub right_shoulder_1: i16,
    pub right_shoulder_2: i16,

    pub dpad_up: i16,
    pub dpad_down: i16,
    pub dpad_left: i16,
    pub dpad_right: i16,

    pub start: i16,
    pub select: i16,

    pub button_square: i16,
    pub button_triangle: i16,
    pub button_cross: i16,
    pub button_circle: i16,

    pub shock_button_l: i16,
    pub shock_button_r: i16,

    pub chat_indicated: i16,
    pub ped_walk: i16,
    pub vehicle_mouse_look: i16,
    pub radio_track_skip: i16,
}

static UPDATE_COUNTER: AtomicU16 = AtomicU16::new(0);

fn update_pads() {
    crate::call_original!(crate::targets::update_pads);

    let (previous_state, current_state) = unsafe {
        let ptr: *const ControllerState = crate::hook::slide(0x1007baf5c);

        let refs = (ptr.offset(0).as_ref(), ptr.offset(1).as_ref());

        if refs.0.is_none() || refs.1.is_none() {
            log::error!("Null controller state!");
            return;
        }

        (refs.0.unwrap(), refs.1.unwrap())
    };

    let counter = UPDATE_COUNTER.load(std::sync::atomic::Ordering::Relaxed);

    if previous_state != current_state || counter >= 10 {
        UPDATE_COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
    } else {
        UPDATE_COUNTER.store(counter + 1, std::sync::atomic::Ordering::Relaxed);
        return;
    }

    crate::gui::with_shared_menu(|menu| {
        menu.handle_controller_input(current_state);
    });
}

pub fn hook() {
    crate::targets::update_pads::install(update_pads);
}
