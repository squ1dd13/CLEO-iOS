pub mod data {
    use std::borrow::Cow;

    use crossbeam_channel::Sender;

    use super::view;

    /// Provides data that is displayed within a row.
    pub trait RowData<Msg> {
        /// Returns the title of the row.
        fn title(&self) -> Cow<'_, str>;

        /// Returns a vector of strings to be shown underneath the title of the row. The strings
        /// will be shown downwards in order.
        fn detail(&self) -> Vec<Cow<'_, str>>;

        /// Returns the string to show on the RHS of the row, indicating the current state of
        /// whatever the row represents.
        fn value(&self) -> Cow<'_, str>;

        /// Returns a value representing the selection of colours that should be applied to the
        /// row's UI components. The tint colour should be selected to provide meaning.
        fn tint(&self) -> view::Tint;

        /// Returns a message to send with the sender in the parent `TabData` structure. This
        /// method will be called when the row is tapped in the menu.
        fn tap_msg(&mut self) -> Option<Msg>;
    }

    /// Data for a message shown above the rows in a tab.
    pub struct TabMsg<'s> {
        pub text: Cow<'s, str>,
        pub tint: view::Tint,
    }

    /// Data used to construct a tab for the user to interact with in the menu.
    pub struct TabData<'s, Msg, R: RowData<Msg>> {
        /// The title of the tab. This is shown at the top of the menu.
        pub title: Cow<'s, str>,

        /// A message shown above the rows.
        pub message: Option<TabMsg<'s>>,

        /// The rows in the tab.
        pub rows: Vec<R>,

        /// A sender for reporting UI changes.
        pub sender: Sender<Msg>,
    }
}

pub mod view {
    use objc::{
        runtime::{Object, Sel},
        *,
    };

    use crate::ui::gui;

    /// Colours that are applied to menu information to add extra meaning.
    ///
    /// [coolors.co](https://coolors.co/78c8ff-4e9540-ffffff-ff535e-ff8000-f3b61f)
    pub enum Tint {
        White,
        Red,
        Orange,
        Yellow,
        Green,
        Blue,
    }

    impl Tint {
        /// Returns the RGB components of the tint colour. The alpha used should vary based on what
        /// the colour is being used for.
        fn rgb(self) -> (u8, u8, u8) {
            match self {
                Tint::White => (255, 255, 255),
                Tint::Red => (255, 83, 94),
                Tint::Orange => (255, 128, 0),
                Tint::Yellow => (243, 182, 31),
                Tint::Green => (78, 149, 64),
                Tint::Blue => (120, 200, 255),
            }
        }

        /// Returns the colour that text using this tint should be.
        fn text_colour(self) -> *const Object {
            let (r, g, b) = self.rgb();

            unsafe {
                msg_send![class!(UIColor), colorWithRed: r as f64 / 255.
                                                  green: g as f64 / 255.
                                                   blue: b as f64 / 255.
                                                  alpha: 0.95_f64]
            }
        }

        /// Returns the background colour that should be used for areas of the screen with this
        /// tint.
        fn background_colour(self) -> *const Object {
            let (r, g, b) = self.rgb();

            unsafe {
                msg_send![class!(UIColor), colorWithRed: r as f64 / 255.
                                                  green: g as f64 / 255.
                                                   blue: b as f64 / 255.
                                                  alpha: 0.2_f64]
            }
        }
    }

    /// A wrapper around the Objective-C `UILabel` class.
    struct Label(*mut Object);

    impl Label {
        fn with_frame(frame: gui::CGRect) -> Label {
            unsafe {
                let label: *mut Object = msg_send![class!(UILabel), alloc];
                let label: *mut Object = msg_send![label, initWithFrame: frame];
                Label(label)
            }
        }

        fn set_text(&mut self, text: &impl AsRef<str>) {
            unsafe {
                let c_string = std::ffi::CString::new(text.as_ref()).expect("CString::new failed");
                let ns_string: *const Object =
                    msg_send![class!(NSString), stringWithUTF8String: c_string.as_ptr()];

                let _: () = msg_send![self.0, setText: ns_string];
            }
        }

        fn set_background(&mut self, colour: *const Object) {
            unsafe {
                let _: () = msg_send![self.0, setBackgroundColor: colour];
            }
        }

        fn set_foreground(&mut self, colour: *const Object) {
            unsafe {
                let _: () = msg_send![self.0, setTextColor: colour];
            }
        }

        fn set_size_font_to_fit(&mut self, stf: bool) {
            unsafe {
                let _: () = msg_send![self.0, setAdjustsFontSizeToFitWidth: stf];
            }
        }

        fn set_alignment(&mut self, alignment: u64) {
            unsafe {
                let _: () = msg_send![self.0, setTextAlignment: alignment];
            }
        }
    }

    impl Drop for Label {
        fn drop(&mut self) {
            unsafe {
                let _: () = msg_send![self.0, release];
            }
        }
    }
}
