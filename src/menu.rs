use objc::{class, msg_send, runtime::Object, sel, sel_impl};

use crate::gui::{self, create_ns_string, CGPoint, CGRect, CGSize};

pub trait RowData {
    fn title(&self) -> String;
    fn detail(&self) -> Option<String>;
    fn value(&self) -> String;
    fn foreground(&self) -> (u8, u8, u8, u8);
    fn handle_tap(&mut self);
}

pub struct TabData {
    name: String,
    warning: Option<String>,
    row_data: Vec<Box<dyn RowData>>,
}

struct Row {
    data: Box<dyn RowData>,
    detail_label: *mut Object,
    value_label: *mut Object,
    button: *mut Object,
}

// fixme: New states don't incorporate controller stuff.
struct TabState {
    selected: bool,
    scroll_y: f64,
}

struct Tab {
    name: String,
    warning: Option<String>,
    scroll_view: *mut Object,
    rows: Vec<Row>,
    state: TabState,
}

struct Menu {
    tabs: Vec<Tab>,
}

impl Row {
    fn new(data: Box<dyn RowData>, frame: gui::CGRect) -> Row {
        unsafe {
            let button: *mut Object = msg_send![class!(UIButton), alloc];
            let button: *mut Object = msg_send![button, initWithFrame: frame];

            let _: () =
                msg_send![button, setTitle: create_ns_string(data.title().as_str()) forState: 0u64];

            Row {
                data,
                detail_label: std::ptr::null_mut(),
                value_label: std::ptr::null_mut(),
                button,
            }
        }
    }
}

impl Tab {
    fn new(data: TabData, tab_frame: gui::CGRect, state: TabState) -> Tab {
        unsafe {
            let scroll_frame = if data.warning.is_some() {
                // Make the scroll view slightly shorter so we can fit the warning above it.
                CGRect::new(
                    tab_frame.origin.x,
                    tab_frame.origin.y + tab_frame.size.height * 0.05,
                    tab_frame.size.width,
                    tab_frame.size.height * 0.95,
                )
            } else {
                tab_frame
            };

            let scroll_view: *mut Object = msg_send![class!(UIScrollView), alloc];
            let scroll_view: *mut Object = msg_send![scroll_view, initWithFrame: scroll_frame];

            const ROW_HEIGHT: f64 = 85.;
            let row_width = scroll_frame.size.width;

            let make_row = |(index, data)| {
                Row::new(
                    data,
                    CGRect::new(0., ROW_HEIGHT * index as f64, row_width, ROW_HEIGHT),
                )
            };

            // Move all the RowData elements into Row structures.
            let rows: Vec<Row> = data
                .row_data
                .into_iter()
                .enumerate()
                .map(make_row)
                .collect();

            for row in rows.iter() {
                let _: () = msg_send![scroll_view, addSubview: row.button];
            }

            Tab {
                name: data.name,
                warning: data.warning,
                scroll_view,
                rows,
                state,
            }
        }
    }
}

// todo: Re-enable RC calls.
impl Drop for Row {
    fn drop(&mut self) {
        unsafe {
            // let _: () = msg_send![self.detail_label, release];
            // let _: () = msg_send![self.value_label, release];
            // let _: () = msg_send![self.button, release];
        }
    }
}

impl Drop for Tab {
    fn drop(&mut self) {
        unsafe {
            // let _: () = msg_send![self.scroll_view, release];
        }
    }
}
