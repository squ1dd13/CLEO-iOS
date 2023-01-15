use super::{
    cell,
    elements::{Button, ScrollView},
};
use crate::meta::language::Message;

/// A trait for objects that can provide the data for a menu tab.
trait DataProvider {
    /// Returns the tab title.
    fn title(&self) -> Message;

    /// Returns the data from which the tab's rows are constructed.
    fn row_data(&self) -> !;
}

/// The state of a tab. This includes its data provider and navigation information.
struct State {
    /// The object that provides the data for this tab. This will be `None` when the menu is not
    /// showing, because data providers only exist while the menu is active.
    data_provider: Option<Box<dyn DataProvider>>,

    /// Data for cells that are always visible to the user.
    pinned: Vec<cell::Data>,

    /// Data for cells that can be scrolled through.
    scrolling: Vec<cell::Data>,

    /// Touch/controller navigation information. This is retained across menu loads so that the
    /// user can continue from where they left off (such as by switching a setting back on after
    /// turning it off).
    nav_state: NavState,
}

/// Tab navigation information. This can be only touch _or_ controller information - never both.
enum NavState {
    /// Touch information.
    Touch(TouchNavState),

    /// Controller information.
    Controller(ControllerNavState),
}

/// Navigation state created when the user touches the screen.
struct TouchNavState {}

/// Navigation state created when the user manipulates the menu with the controller.
struct ControllerNavState {}

/// Manages the button and content views for a single tab.
struct ViewSystem {
    /// The button that switches the user to this tab.
    button: TabButton,

    /// The view that contains the tab's content.
    content: ContentView,
}

/// The views that make up a single tab's content. This does not include the button.
struct ContentView {
    /// The cell views which are always visible. These are placed in a stack above the scroll view.
    /// The more pinned views there are, the less space there is for the scroll view.
    pinned_views: Vec<cell::View>,

    /// The scroll view containing the scrollable cells.
    scroll_view: ScrollView,
}

/// The button used to switch to a particular tab.
struct TabButton {
    /// The button component.
    button_view: Button,
}
