//! Hooks the splash screen to display our "CLEO" numberplate, and also provides a Rust interface for some
//! common UIKit code.

use crate::meta::language::MessageKey;
use log::trace;
use objc::{
    class, msg_send,
    runtime::{Object, Sel},
    sel,
};
use std::os::raw::c_long;

/// Fonts that we can use in CLEO's GUI. These have been selected to support all the different
/// devices and languages we support.
#[derive(Clone, Copy, Debug)]
pub enum Font {
    AvenirNextCondensed,
    AvenirNextHeavy,
    AvenirNextMedium,
    PingFangLight,
    PingFangSemibold,
    PingFangMedium,
    ChaletComprime,
    KhmerSangam,
    Pricedown,
    GtaLicense,
}

impl Font {
    /// Returns the name of the font. This can be used with UIKit.
    fn name(self) -> &'static str {
        match self {
            Font::AvenirNextCondensed => "AvenirNextCondensed-Regular",
            Font::AvenirNextHeavy => "AvenirNext-Heavy",
            Font::AvenirNextMedium => "AvenirNext-Medium",

            Font::PingFangLight => "PingFangSC-Light",
            Font::PingFangSemibold => "PingFangSC-Semibold",
            Font::PingFangMedium => "PingFangSC-Medium",

            Font::ChaletComprime => "ChaletComprime-CologneSixty",
            Font::KhmerSangam => "KhmerSangamMN",
            Font::Pricedown => "PricedownGTAVInt",
            Font::GtaLicense => "GTALICENSE-REGULAR",
        }
    }

    /// Creates a `UIFont` object for the font at a particular size.
    pub fn uifont(self, size: f64) -> *mut Object {
        unsafe { msg_send![class!(UIFont), fontWithName: ns_string(self.name()) size: size] }
    }

    /// Converts a pair containing a font and a size into a `UIFont`.
    pub fn pair_uifont((font, size): (Font, f64)) -> *mut Object {
        font.uifont(size)
    }
}

/// Represents a group of fonts used for the CLEO GUI. Different font sets are used for different
/// languages in order to improve readability and appearance.
#[derive(Clone, Copy)]
pub struct FontSet {
    /// The font used for large, bold titles.
    pub title_font: Font,

    /// The size used for the title font.
    pub title_size: f64,

    /// The font used for small but important text.
    pub small_font: Font,

    /// The size used for the small font.
    pub small_size: f64,

    /// The font used for normal text.
    pub text_font: Font,

    /// The size used for the normal text font.
    pub text_size: f64,

    /// The font used for fairly small subtitles.
    pub subtitle_font: Font,

    /// The size used for the subtitle font.
    pub subtitle_size: f64,
}

impl FontSet {
    /// Returns the `UIFont` to be used for titles.
    pub fn title_uifont(self) -> *mut Object {
        self.title_font.uifont(self.title_size)
    }

    /// Returns the `UIFont` to be used for small text.
    pub fn small_uifont(self) -> *mut Object {
        self.small_font.uifont(self.small_size)
    }

    /// Returns the `UIFont` to be used for normal text.
    pub fn text_uifont(self) -> *mut Object {
        self.text_font.uifont(self.text_size)
    }

    /// Returns the `UIFont` to be used for subtitles.
    pub fn subtitle_uifont(self) -> *mut Object {
        self.subtitle_font.uifont(self.subtitle_size)
    }
}

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

    /// The CLEO GUI colour palette.
    #[derive(Clone, Copy)]
    pub enum Colour {
        Red,
        Orange,
        Green,
        Blue,
    }

    impl Colour {
        /// Returns the RGB values for this colour.
        pub const fn rgb(self) -> Rgb {
            match self {
                Colour::Red => (255, 59, 76),
                Colour::Orange => (255, 120, 0),
                Colour::Green => (78, 183, 64),
                Colour::Blue => (100, 200, 255),
            }
        }
    }

    pub const RED: Rgb = Colour::Red.rgb();
    pub const ORANGE: Rgb = Colour::Orange.rgb();
    pub const GREEN: Rgb = Colour::Green.rgb();
    pub const BLUE: Rgb = Colour::Blue.rgb();

    pub fn get(colour: Rgb, alpha: f64) -> *const Object {
        unsafe {
            msg_send![class!(UIColor), colorWithRed: colour.0 as f64 / 255. green: colour.1 as f64 / 255. blue: colour.2 as f64 / 255. alpha: alpha]
        }
    }

    pub fn white_with_alpha(white: f64, alpha: f64) -> *const Object {
        unsafe { msg_send![class!(UIColor), colorWithWhite: white alpha: alpha] }
    }
}

pub fn ns_string(rust_string: impl AsRef<str>) -> *const Object {
    unsafe {
        let c_string = std::ffi::CString::new(rust_string.as_ref()).expect("CString::new failed");
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
            });
        });
    }
}

fn legal_splash_did_load(this: *mut Object, _sel: Sel) {
    log::info!("Setting up splash screen.");

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
            let font: *mut Object =
                msg_send![class!(UIFont), fontWithName: ns_string("GTALICENSE-REGULAR") size: 23.0];
            let text_colour: *const Object =
                msg_send![class!(UIColor), colorWithRed: 0.77 green: 0.089 blue: 0.102 alpha: 1.0];

            let ver_string = crate::meta::github::current_version().to_string();

            let state_label: *mut Object = create_label(bounds, &ver_string, font, text_colour, 1);
            let _: () = msg_send![state_label, sizeToFit];

            state_label
        };

        let state_frame: CGRect = msg_send![state_label, frame];

        let text = {
            let font: *mut Object =
                msg_send![class!(UIFont), fontWithName: ns_string("GTALICENSE-REGULAR") size: 70.0];
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

        // Call the original implementation of viewDidLoad.
        let _: () = msg_send![this, origViewDidLoad];

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

        let copyright = {
            let legal = MessageKey::SplashLegal
                .format(super::language::msg_args!["copyright_names" => super::COPYRIGHT_NAMES])
                .translate();

            let fun = MessageKey::SplashFun.to_message().translate();

            format!("{legal}\n{fun}")
        };

        let label: *mut Object = msg_send![class!(UILabel), alloc];
        let label: *mut Object = msg_send![label, initWithFrame: bottom_text_frame];
        let font = super::language::current().font_set().small_uifont();
        let colour = colours::white_with_alpha(0.5, 0.7);
        let _: () = msg_send![label, setTextColor: colour];
        let _: () = msg_send![label, setFont: font];
        let _: () = msg_send![label, setText: ns_string(copyright)];
        let _: () = msg_send![label, setTextAlignment: 1u64];
        let _: () = msg_send![label, setNumberOfLines: 2u64];
        let _: () = msg_send![label, setAdjustsFontSizeToFitWidth: true];

        let _: () = msg_send![view, addSubview: background_view];
        let _: () = msg_send![background_view, release];
        let _: () = msg_send![view, addSubview: label];
        let _: () = msg_send![label, release];
    }

    log::info!("Finished setting up splash screen.");
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

        let _: () = msg_send![label, setText: ns_string(text)];
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

pub fn init() {
    log::info!("installing GUI hooks...");

    crate::hook::hook_objc(
        "LegalSplash",
        "viewDidLoad",
        "origViewDidLoad",
        legal_splash_did_load as *const (),
    );

    crate::hook::hook_objc(
        "SCAppDelegate",
        "persistentStoreCoordinator",
        "origPersistentStoreCoordinator",
        persistent_store_coordinator as *const (),
    );
}
