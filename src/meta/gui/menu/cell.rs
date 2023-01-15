use super::elements::Colour;
use crate::meta::language::Message;

/// Types of interaction that the user can have with a cell.
pub enum InteractionType {
    /// An interaction with no associated direction, such as a single tap on the screen.
    Undirected,

    /// An interaction in the left direction, such as the left button on a controller dpad.
    Left,

    /// An interaction in the right direction, such as the right button on a controller dpad.
    Right,
}

/// Different types of refresh that the menu can do.
pub enum RefreshScope {
    /// Reloads a single cell.
    Cell,

    /// Reloads a single tab.
    Tab,

    /// Reloads the whole menu.
    Menu,
}

/// Trait providing data for cells which respond to user interaction.
pub trait InteractableData {
    /// Returns the title of the cell.
    fn title(&self) -> Message;

    /// Returns the description of the cell.
    fn description(&self) -> Message;

    /// Returns the tint colour to be applied to the cell's text.
    fn text_tint(&self) -> Colour;

    /// Returns the background tint colour.
    fn background_tint(&self) -> Colour;

    /// Called when the cell representing this data is interacted with. Returns how much of the
    /// menu needs to be reloaded, or `None` if no refresh is required.
    fn on_interaction(&mut self, interaction: InteractionType) -> Option<RefreshScope>;
}

/// Types of data that can be displayed in a cell.
pub enum Data {
    /// Centre-aligned text.
    CentredText {
        /// The message to display.
        message: Message,

        /// The colour of the displayed text.
        text_colour: Colour,
    },

    /// Text aligned to either the left or right, depending on the language.
    NaturalText {
        /// The message to display.
        message: Message,

        /// The colour of the displayed text.
        text_colour: Colour,

        /// The background tint of the cell.
        background_tint: Colour,
    },

    /// Data which can change based on user interaction.
    Interactable(Box<dyn InteractableData>),
}

pub type View = ();
