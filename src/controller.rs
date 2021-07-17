//! Adapts and uses the game's controller system to allow CLEO to take advantage of controllers.

use objc::{runtime::Object, *};
use std::sync::atomic::AtomicU16;

use crate::old_menu::MenuAction;

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

impl ControllerState {
    pub fn has_input(&self) -> bool {
        // eq: CControllerState::CheckForInput(...)
        crate::hook::slide::<fn(*const ControllerState) -> bool>(0x100244118)(self)
    }
}

static UPDATE_COUNTER: AtomicU16 = AtomicU16::new(0);

pub fn request_update() {
    // Set the update counter really high so the next call to update_pads triggers a menu update.
    UPDATE_COUNTER.store(1000, std::sync::atomic::Ordering::Relaxed);
}

fn update_pads() {
    crate::call_original!(crate::targets::update_pads);

    let (current_state, previous_state) = unsafe {
        let ptr: *mut ControllerState = crate::hook::slide(0x1007baf5c);

        let refs = (ptr.offset(0).as_mut(), ptr.offset(1).as_ref());

        if refs.0.is_none() || refs.1.is_none() {
            log::error!("Null controller state!");
            return;
        }

        (refs.0.unwrap(), refs.1.unwrap())
    };

    let counter = UPDATE_COUNTER.load(std::sync::atomic::Ordering::Relaxed);

    unsafe {
        // The game's controller structure has fields for 'start' and 'select', but doesn't actually
        //  assign them values, so we need to check for those buttons.
        let controllers: *const Object = msg_send![class!(GCController), controllers];
        let count: usize = msg_send![controllers, count];

        for i in 0..count {
            let controller: *const Object = msg_send![controllers, objectAtIndex: i];
            let extended_gamepad: *const Object = msg_send![controller, extendedGamepad];

            if extended_gamepad.is_null() {
                continue;
            }

            // fixme: We need to change our approach to getting controller input based on the iOS version.

            let select_pressed = {
                let responds: bool =
                    msg_send![extended_gamepad, respondsToSelector: sel!(buttonOptions)];

                if responds {
                    let button: *const Object = msg_send![extended_gamepad, buttonOptions];

                    if !button.is_null() {
                        let pressed: f32 = msg_send![button, value];
                        pressed > 0.125
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if select_pressed {
                current_state.select = 255;
                break;
            }
        }
    }

    if previous_state != current_state || counter >= 10 {
        UPDATE_COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
    } else {
        UPDATE_COUNTER.store(counter + 1, std::sync::atomic::Ordering::Relaxed);
        return;
    }

    // If the counter is 1000, the update was requested by the menu. In this case, we
    //  don't want to act on the state of the select button because it is likely that the user is still
    //  holding it down, so if the menu has only just been shown then it will be removed again.
    if counter != 1000 && current_state.select != 0 {
        MenuAction::queue(MenuAction::Toggle(true));
    }

    crate::old_menu::queue_controller_input(current_state);
}

pub fn hook() {
    crate::targets::update_pads::install(update_pads);
}
