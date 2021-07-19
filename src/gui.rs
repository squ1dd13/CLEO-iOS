//! Hooks the splash screen to display our "CLEO" numberplate, and also provides a Rust interface for some
//! common UIKit code.

use crate::{call_original, targets};
use log::trace;
use objc::runtime::{Class, Sel};
use objc::{runtime::Object, *};
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

    pub const RED: Rgb = (255, 40, 46);
    pub const ORANGE: Rgb = (255, 128, 0);
    pub const GREEN: Rgb = (78, 149, 64);

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

pub fn create_with_frame(class: &Class, frame: CGRect) -> *mut Object {
    unsafe {
        let allocated: *mut Object = msg_send![class, alloc];
        msg_send![allocated, initWithFrame: frame]
    }
}

pub fn create_ns_string(rust_string: &str) -> *const Object {
    unsafe {
        let c_string = std::ffi::CString::new(rust_string).expect("CString::new failed");
        let ns_string: *const Object =
            msg_send![class!(NSString), stringWithUTF8String: c_string.as_ptr()];

        ns_string
    }
}

fn legal_splash_did_load(this: *mut Object, sel: Sel) {
    trace!("splish splash splosh");

    // All of this code draws the numberplate splash screen. I'm too lazy to embed an image
    //  and use a UIImageView, so the numberplate is made from scratch with UIViews and UILabels.
    unsafe {
        let view: *mut Object = msg_send![this, view];
        let bounds: CGRect = msg_send![view, bounds];

        let background_view: *mut Object = msg_send![class!(UIView), alloc];
        let background_view: *mut Object = msg_send![background_view, initWithFrame: bounds];

        let background_colour: *const Object = msg_send![class!(UIColor), blackColor];
        let _: () = msg_send![background_view, setBackgroundColor: background_colour];

        let exempt = {
            let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("GTALICENSE-REGULAR") size: 23.0];
            let text_colour: *const Object =
                msg_send![class!(UIColor), colorWithRed: 0.77 green: 0.089 blue: 0.102 alpha: 1.0];

            let exempt_label: *mut Object = create_label(bounds, "SA EXEMPT", font, text_colour, 1);
            let _: () = msg_send![exempt_label, sizeToFit];

            exempt_label
        };

        let exempt_frame: CGRect = msg_send![exempt, frame];

        let text = {
            let font: *mut Object = msg_send![class!(UIFont), fontWithName: create_ns_string("GTALICENSE-REGULAR") size: 70.0];
            let text_colour: *const Object =
                msg_send![class!(UIColor), colorWithRed: 0.14 green: 0.37 blue: 0.62 alpha: 1.0];

            let plate_label: *mut Object = create_label(
                CGRect {
                    origin: CGPoint {
                        x: 0.0,
                        y: exempt_frame.size.height,
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
            (backing_size.height - (text_frame.size.height + exempt_frame.size.height)) / 2.0;

        let exempt_centre = CGPoint {
            x: backing_size.width / 2.0,
            y: (exempt_frame.size.height / 2.0) + y_gap,
        };

        let text_centre = CGPoint {
            x: backing_size.width / 2.0,
            y: backing_size.height - ((text_frame.size.height / 2.0) + y_gap),
        };

        let _: () = msg_send![exempt, setCenter: exempt_centre];
        let _: () = msg_send![text, setCenter: text_centre];

        if !crate::hook::is_german_game() {
            call_original!(targets::legal_splash, this, sel);
        } else {
            call_original!(targets::legal_splash_german, this, sel);
        }

        let _: () = msg_send![backing, addSubview: exempt];
        let _: () = msg_send![exempt, release];
        let _: () = msg_send![backing, addSubview: text];
        let _: () = msg_send![text, release];
        let _: () = msg_send![background_view, addSubview: backing_outer];
        let _: () = msg_send![backing, release];

        let _: () = msg_send![view, addSubview: background_view];
        let _: () = msg_send![background_view, release];
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

/*
        This hook fixes a bug in the game where -[SCAppDelegate persistentStoreCoordinator]
    calls -[SCAppDelegate managedObjectModel], which crashes the game because it attempts
    to call -[NSManagedObjectModel initWithContentsOfURL:] with a nil URL that it gets
    from calling -[NSBundle URLForResource:withExtension:] for the resource "gtasa.momd",
    which does not exist.

        The easiest way to fix this issue is to hook -[SCAppDelegate persistentStoreCoordinator]
    to always return a null pointer, since the method that calls it,
    -[SCAppDelegate managedObjectContext], checks the return value to see if it is null
    before attempting to do anything with it. This seems to be a fairly robust fix since
    everything further up the callstack has decent checks in place to prevent issues with
    null pointers.

        These events only occur when the app is terminated, so the crash
    is fairly insignificant, but on a jailbroken device with crash reporting tools installed,
    the constant crash reports can get annoying.
*/
fn persistent_store_coordinator(_this: *mut Object, _sel: Sel) -> *const Object {
    trace!("-[SCAppDelegate persistentStoreCoordinator] called. Returning null to prevent crash.");
    std::ptr::null()
}

pub fn hook() {
    if !crate::hook::is_german_game() {
        targets::legal_splash::install(legal_splash_did_load);
    } else {
        trace!("Correcting splash address for German game.");
        targets::legal_splash_german::install(legal_splash_did_load);
    }

    targets::store_crash_fix::install(persistent_store_coordinator);
}
