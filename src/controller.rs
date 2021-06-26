use objc::{runtime::Object, *};
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

    let (current_state, previous_state) = unsafe {
        let ptr: *mut ControllerState = crate::hook::slide(0x1007baf5c);

        let refs = (ptr.offset(0).as_mut(), ptr.offset(1).as_ref());

        if refs.0.is_none() || refs.1.is_none() {
            log::error!("Null controller state!");
            return;
        }

        (refs.0.unwrap(), refs.1.unwrap())
    };

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

            // fn pressed(button: *const Object) -> bool {
            //     if !button.is_null() {
            //         let pressed: f32 = unsafe { msg_send![button, value] };
            //         pressed > 0.125
            //     } else {
            //         false
            //     }
            // }

            // let button_0 = pressed(msg_send![extended_gamepad, button0]);
            // let button_1 = pressed(msg_send![extended_gamepad, button1]);
            // let button_2 = pressed(msg_send![extended_gamepad, button2]);

            // log::trace!("0: {:?}, 1: {:?}, 2: {:?}", button_0, button_1, button_2);

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

    let counter = UPDATE_COUNTER.load(std::sync::atomic::Ordering::Relaxed);

    if previous_state != current_state || counter >= 10 {
        UPDATE_COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
    } else {
        UPDATE_COUNTER.store(counter + 1, std::sync::atomic::Ordering::Relaxed);
        return;
    }

    if current_state.select != 0 {
        // If with_shared_menu returns None, then we know that there isn't a menu currently.
        // In order to let the player use the button to toggle the menu, we change what we
        //  do based on whether a menu exists.
        if crate::gui::with_shared_menu(|_| {}).is_none() {
            // No menu, so create and show one.
            crate::gui::show_menu();
        } else {
            // There is a menu, so remove it.
            dispatch::Queue::main().exec_sync(|| crate::gui::hide_menu_on_main_thread());
        }
    }

    crate::gui::with_shared_menu(|menu| {
        menu.handle_controller_input(current_state);
    });
}

pub fn hook() {
    crate::targets::update_pads::install(update_pads);
}
