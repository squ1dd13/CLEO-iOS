//! Hooks the splash screen to display our "CLEO" numberplate, and also provides a Rust interface for some
//! common UIKit code.

use crate::{call_original, targets};
use log::trace;
use objc::{
    runtime::{Object, Sel},
    *,
};
use std::os::raw::c_long;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CGSize {
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}

impl CGRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> CGRect {
        CGRect {
            origin: CGPoint { x, y },
            size: CGSize { width, height },
        }
    }

    pub fn rounded(self) -> Self {
        CGRect {
            origin: CGPoint {
                x: self.origin.x.round(),
                y: self.origin.y.round(),
            },
            size: CGSize {
                width: self.size.width.round(),
                height: self.size.height.round(),
            },
        }
    }
}

#[repr(C)]
pub struct UIEdgeInsets {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

impl UIEdgeInsets {
    pub fn new(top: f64, left: f64, bottom: f64, right: f64) -> UIEdgeInsets {
        UIEdgeInsets {
            top,
            left,
            bottom,
            right,
        }
    }
}

pub type Rgb = (u8, u8, u8);

pub mod colours {
    use super::*;

    pub const RED: Rgb = (255, 83, 94);
    pub const ORANGE: Rgb = (255, 128, 0);
    pub const GREEN: Rgb = (78, 149, 64);
    pub const BLUE: Rgb = (120, 200, 255);

    pub fn get(colour: Rgb, alpha: f64) -> *const Object {
        unsafe {
            msg_send![class!(UIColor), colorWithRed: colour.0 as f64 / 255. green: colour.1 as f64 / 255. blue: colour.2 as f64 / 255. alpha: alpha]
        }
    }

    pub fn white_with_alpha(white: f64, alpha: f64) -> *const Object {
        unsafe { msg_send![class!(UIColor), colorWithWhite: white alpha: alpha] }
    }
}

pub fn get_font(name: &str, size: f64) -> *const Object {
    unsafe { msg_send![class!(UIFont), fontWithName: create_ns_string(name) size: size] }
}

pub fn create_ns_string(rust_string: &str) -> *const Object {
    unsafe {
        let c_string = std::ffi::CString::new(rust_string).expect("CString::new failed");
        let ns_string: *const Object =
            msg_send![class!(NSString), stringWithUTF8String: c_string.as_ptr()];

        ns_string
    }
}

pub fn exit_to_homescreen() {
    unsafe {
        dispatch::Queue::main().exec_sync(|| {
            let control: *mut Object = msg_send![class!(UIControl), new];
            let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
            let _: () = msg_send![control, sendAction: sel!(suspend) to: app forEvent: 0usize];

            dispatch::Queue::main().exec_after(std::time::Duration::from_millis(200), || {
                std::process::exit(0);
            })
        });
    }
}

fn legal_splash_did_load(this: *mut Object, sel: Sel) {
    log::info!("Showing splash screen.");

    // All of this code draws the numberplate splash screen. I'm too lazy to embed an image
    //  and use a UIImageView, so the numberplate is made from scratch with UIViews and UILabels.
    unsafe {
        let view: *mut Object = msg_send![this, view];
        let bounds: CGRect = msg_send![view, bounds];

        let background_view: *mut Object = msg_send![class!(UIView), alloc];
        let background_view: *mut Object = msg_send![background_view, initWithFrame: bounds];

        let background_colour: *const Object = msg_send![class!(UIColor), blackColor];
        let _: () = msg_send![background_view, setBackgroundColor: background_colour];

        let state_label = {
            let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("GTALICENSE-REGULAR") size: 23.0];
            let text_colour: *const Object =
                msg_send![class!(UIColor), colorWithRed: 0.77 green: 0.089 blue: 0.102 alpha: 1.0];

            let state_label: *mut Object = create_label(
                bounds,
                &format!(
                    "{} {}",
                    if cfg!(feature = "debug") {
                        "BETA"
                    } else {
                        "RELEASE"
                    },
                    env!("CARGO_PKG_VERSION")
                ),
                font,
                text_colour,
                1,
            );
            let _: () = msg_send![state_label, sizeToFit];

            state_label
        };

        let state_frame: CGRect = msg_send![state_label, frame];

        let text = {
            let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("GTALICENSE-REGULAR") size: 70.0];
            let text_colour: *const Object =
                msg_send![class!(UIColor), colorWithRed: 0.14 green: 0.37 blue: 0.62 alpha: 1.0];

            let plate_label: *mut Object = create_label(
                CGRect {
                    origin: CGPoint {
                        x: 0.0,
                        y: state_frame.size.height,
                    },
                    ..bounds
                },
                "CLEO",
                font,
                text_colour,
                1,
            );

            let _: () = msg_send![plate_label, sizeToFit];

            plate_label
        };

        let text_frame: CGRect = msg_send![text, frame];

        let backing_size = CGSize {
            width: text_frame.size.width * 2.3,
            height: text_frame.size.height * 1.9,
        };

        let (backing, backing_outer) = {
            let outer_frame = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize {
                    width: backing_size.width + 8.0,
                    height: backing_size.height + 8.0,
                },
            };

            let backing_view_outer: *mut Object = msg_send![class!(UIView), alloc];
            let backing_view_outer: *mut Object =
                msg_send![backing_view_outer, initWithFrame: outer_frame];

            let backing_view: *mut Object = msg_send![class!(UIView), alloc];
            let backing_view: *mut Object = msg_send![backing_view, initWithFrame: CGRect {
                origin: CGPoint {
                    x: 0.0,
                    y: 0.0,
                },
                size: backing_size,
            }];

            let white: *const Object = msg_send![class!(UIColor), whiteColor];
            let _: () = msg_send![backing_view_outer, setBackgroundColor: white];

            let _: () = msg_send![backing_view_outer, setCenter: CGPoint {
                x: bounds.size.width / 2.0,
                y: bounds.size.height / 2.0,
            }];

            let _: () = msg_send![backing_view, setCenter: CGPoint {
                x: outer_frame.size.width / 2.0,
                y: outer_frame.size.height / 2.0,
            }];

            let border_colour: *const Object =
                msg_send![class!(UIColor), colorWithWhite: 0.0 alpha: 0.27];
            let border_colour: *const Object = msg_send![border_colour, CGColor];

            let layer: *mut Object = msg_send![backing_view, layer];
            let _: () = msg_send![layer, setCornerRadius: 10.0];
            let _: () = msg_send![layer, setBorderWidth: 2.0];
            let _: () = msg_send![layer, setBorderColor: border_colour];

            let layer: *mut Object = msg_send![backing_view_outer, layer];
            let _: () = msg_send![layer, setCornerRadius: 12.0];

            let _: () = msg_send![backing_view_outer, addSubview: backing_view];
            let _: () = msg_send![backing_view, release];

            (backing_view, backing_view_outer)
        };

        // Calculate the gap between the elements and the edge of the plate on the top and bottom.
        let y_gap =
            (backing_size.height - (text_frame.size.height + state_frame.size.height)) / 2.0;

        let state_centre = CGPoint {
            x: backing_size.width / 2.0,
            y: (state_frame.size.height / 2.0) + y_gap,
        };

        let text_centre = CGPoint {
            x: backing_size.width / 2.0,
            y: backing_size.height - ((text_frame.size.height / 2.0) + y_gap),
        };

        let _: () = msg_send![state_label, setCenter: state_centre];
        let _: () = msg_send![text, setCenter: text_centre];

        if !crate::hook::is_german_game() {
            call_original!(targets::legal_splash, this, sel);
        } else {
            call_original!(targets::legal_splash_german, this, sel);
        }

        let _: () = msg_send![backing, addSubview: state_label];
        let _: () = msg_send![state_label, release];
        let _: () = msg_send![backing, addSubview: text];
        let _: () = msg_send![text, release];
        let _: () = msg_send![background_view, addSubview: backing_outer];
        let _: () = msg_send![backing, release];

        let bottom_text_frame = CGRect {
            origin: CGPoint {
                x: 0.0,
                y: bounds.size.height * 0.9,
            },
            size: CGSize {
                width: bounds.size.width,
                height: bounds.size.height * 0.1,
            },
        };

        let copyright = "Copyright © 2020-2022 squ1dd13. Code licenced under the MIT License.\nMade with love in the United Kingdom. Have fun!";

        let label: *mut Object = msg_send![class!(UILabel), alloc];
        let label: *mut Object = msg_send![label, initWithFrame: bottom_text_frame];
        let font = get_font("HelveticaNeue", 10.);
        let colour = colours::white_with_alpha(0.5, 0.7);
        let _: () = msg_send![label, setTextColor: colour];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![label, setText: create_ns_string(copyright)];
        let _: () = msg_send![label, setTextAlignment: 1u64];
        let _: () = msg_send![label, setNumberOfLines: 2u64];

        let _: () = msg_send![view, addSubview: background_view];
        let _: () = msg_send![background_view, release];
        let _: () = msg_send![view, addSubview: label];
        let _: () = msg_send![label, release];
    }
}

pub fn create_label(
    frame: CGRect,
    text: &str,
    font: *const Object,
    colour: *const Object,
    alignment: u32,
) -> *mut Object {
    unsafe {
        let running: *mut Object = msg_send![class!(UILabel), alloc];
        let label: *mut Object = msg_send![running, initWithFrame: frame];

        let _: () = msg_send![label, setText: create_ns_string(text)];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![label, setTextColor: colour];
        let _: () = msg_send![label, setAdjustsFontSizeToFitWidth: true];
        let _: () = msg_send![label, setTextAlignment: alignment as c_long];

        label
    }
}

fn show_update_prompt(screen: *mut u8) {
    use crate::{hook, text};

    unsafe {
        screen.offset(0x75).write(0);

        // eq: MobileMenu::Load(...)
        hook::slide::<fn(*mut u8)>(0x100339838)(screen);

        // Add our custom strings so we can use them in the menu.
        text::set_kv("CL_UPT", "Update Available");
        text::set_kv(
            "CL_UPM",
            "A new CLEO update is available. Do you want to go to GitHub to download it?",
        );

        // eq: nag_menu = operator.new(0x80)
        let menu = hook::slide::<fn(u64) -> u64>(0x1004f9be0)(0x80);

        let on_yes = |_: u64| {
            const GITHUB_URL: &str = "https://github.com/squ1dd13/CLEO-iOS/releases/latest";

            let url: *const Object =
                msg_send![class!(NSURL), URLWithString: create_ns_string(GITHUB_URL)];

            let shared_app: *const Object = msg_send![class!(UIApplication), sharedApplication];

            // eq: [[UIApplication sharedApplication] openURL: [NSURL URLWithString: ...]]
            let _: () = msg_send![shared_app, openURL: url];
        };

        // eq: MobileMenu::InitForNag(...)
        hook::slide::<fn(u64, *const u8, *const u8, fn(u64), u64, u64, bool) -> u64>(0x100348964)(
            menu,                 // Menu structure (uninitialised)
            b"CL_UPT\0".as_ptr(), // Title
            b"CL_UPM\0".as_ptr(), // Message
            on_yes,               // "Yes" function
            0,                    // Callback argument
            0,                    // "No" function
            false,                // Enable 'back' button
        );

        // We could create a repl(C) struct, but the fields we need are at fairly large
        //  offsets, so it's easiest just to mess with pointers.
        let u64_ptr: *mut u64 = screen.cast();

        // Offset is 6 * u64, so 48 bytes (0x30).
        if u64_ptr.offset(6).read() != 0 {
            // eq: MobileMenu::ProcessPending(...)
            hook::slide::<fn(*mut u64)>(0x100338f5c)(u64_ptr);
        }

        u64_ptr.offset(6).write(menu);
    }
}

// This function is responsible for setting up the main flow screen, so we use it to
//  show our update prompt when the game loads.
fn init_for_title(screen: *mut u8) {
    // Set up the title menu.
    call_original!(crate::targets::init_for_title, screen);

    if crate::update::was_update_found() {
        // Create our prompt afterwards, so it's above the title menu.
        show_update_prompt(screen);
    }
}

// Fixes an annoying crash that happens just before the game exits. Normal users don't notice this crash (since
// it's only when the game is killed) but jailbroken users that get notified when processes crash may find the
// crash alerts annoying.
fn persistent_store_coordinator(_this: *mut Object, _sel: Sel) -> *const Object {
    trace!("-[SCAppDelegate persistentStoreCoordinator] called. Returning null to prevent crash.");
    std::ptr::null()
}

pub fn init() {
    if !crate::hook::is_german_game() {
        targets::legal_splash::install(legal_splash_did_load);
    } else {
        trace!("Correcting splash address for German game.");
        targets::legal_splash_german::install(legal_splash_did_load);
    }

    targets::init_for_title::install(init_for_title);
    targets::store_crash_fix::install(persistent_store_coordinator);
}
