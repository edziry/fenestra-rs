use crate::{ImeEvent, KeyboardInput, Modifiers, Size};

/// Owned application events, independent of a native window host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    /// A pointer moved to physical viewport coordinates.
    PointerMoved {
        /// Horizontal coordinate.
        x: i32,
        /// Vertical coordinate.
        y: i32,
    },
    /// The left mouse button was pressed over the current committed scene.
    Click {
        /// The topmost input-enabled element name, if any.
        target: Option<String>,
    },
    /// Compatibility notification after a fresh Space press; excludes repeats.
    ///
    /// Text consumers use `KeyboardInput` and must not also insert from this event.
    SpacePressed,
    /// An owned logical key event, including releases and automatic repeats.
    KeyboardInput(KeyboardInput),
    /// The active keyboard modifiers changed.
    ModifiersChanged(Modifiers),
    /// Window focus changed. Discard held keys and preedit on focus loss.
    Focused(bool),
    /// A native input method changed composition state or committed text.
    Ime(ImeEvent),
    /// A valid nonzero viewport size was committed.
    Resized {
        /// The current pixel size.
        size: Size,
    },
    /// The user requested that the native window close.
    CloseRequested,
}
