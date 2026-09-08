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
    /// The primary mouse button was released over the current committed scene.
    PointerReleased {
        /// The topmost input-enabled element name at the last pointer position.
        target: Option<String>,
    },
    /// The pointer left the window, invalidating its last position.
    PointerLeft,
    /// A control was activated after its state was committed.
    Activated {
        /// The activated control's element name.
        target: String,
    },
    /// A checkbox changed its committed checked state.
    CheckedChanged {
        /// The checkbox's element name.
        target: String,
        /// The newly committed checked state.
        checked: bool,
    },
    /// Keyboard focus changed after its state was committed.
    FocusChanged {
        /// The focused control's element name, or `None` when focus was cleared.
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
