/// Owned application input, independent of a native window host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputEvent {
    /// The pointer moved to physical window pixel coordinates.
    PointerMoved {
        /// Horizontal pixel coordinate.
        x: i32,
        /// Vertical pixel coordinate.
        y: i32,
    },
    /// The primary mouse button was pressed.
    PointerPressed,
    /// Compatibility notification after a fresh physical Space press.
    ///
    /// Excludes repeats, synthetic input, and active IME composition. Text
    /// consumers must use `KeyboardInput` and must not also insert from this event.
    SpacePressed,
    /// An owned logical key event, including releases and automatic repeats.
    KeyboardInput(KeyboardInput),
    /// The active keyboard modifiers changed.
    ModifiersChanged(Modifiers),
    /// Window focus changed. On loss, discard held keys and preedit state.
    ///
    /// The host clears its modifier snapshot before delivering focus loss.
    Focused(bool),
    /// A native input method changed its composition state or committed text.
    Ime(ImeEvent),
    /// The user requested normal window closure.
    CloseRequested,
}

/// A logical key, independent of the private platform event types.
///
/// This initial vocabulary covers text editing and common modifiers. Other
/// platform named keys are represented as `Unidentified`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Key {
    /// Layout-dependent key text; use `KeyboardInput::text` for insertion.
    Character(String),
    /// A dead key that may combine with a later key.
    Dead(Option<char>),
    /// The Space key.
    Space,
    /// The Enter key.
    Enter,
    /// The Tab key.
    Tab,
    /// The Backspace key.
    Backspace,
    /// The Delete key.
    Delete,
    /// The Escape key.
    Escape,
    /// The left arrow key.
    ArrowLeft,
    /// The right arrow key.
    ArrowRight,
    /// The up arrow key.
    ArrowUp,
    /// The down arrow key.
    ArrowDown,
    /// The Home key.
    Home,
    /// The End key.
    End,
    /// The Page Up key.
    PageUp,
    /// The Page Down key.
    PageDown,
    /// The Insert key.
    Insert,
    /// A Shift key.
    Shift,
    /// A Control key.
    Control,
    /// An Alt key.
    Alt,
    /// A Super or Windows key.
    Super,
    /// An unknown key or a named key outside this vocabulary.
    Unidentified,
}

/// A snapshot of modifiers reported by the window system.
///
/// These flags do not define shortcut policy or distinguish AltGr from other
/// Control/Alt combinations. Clients must not infer text solely from them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    /// Shift is active.
    pub shift: bool,
    /// Control is active.
    pub control: bool,
    /// Alt is active.
    pub alt: bool,
    /// Super or Windows is active.
    pub super_key: bool,
}

/// Whether a logical key was pressed or released.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyState {
    /// The key was pressed, possibly as an automatic repeat.
    Pressed,
    /// The key was released.
    Released,
}

/// An owned key event delivered without platform library types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyboardInput {
    /// The logical key after the keyboard layout is applied.
    pub key: Key,
    /// Press or release state.
    pub state: KeyState,
    /// The most recently reported modifier snapshot.
    pub modifiers: Modifiers,
    /// Whether this press is an automatic repeat.
    pub repeat: bool,
    /// Text reported for this press, which can differ from the logical key.
    ///
    /// May contain control characters. No shortcut or insertion policy is
    /// applied. Absent on releases, synthetic events, unfocused input, and
    /// during preedit. IME commits arrive only through `ImeEvent::Commit`.
    pub text: Option<String>,
    /// Whether the window system synthesized this event during focus changes.
    pub is_synthetic: bool,
}

/// Owned input method notifications; platform availability varies.
///
/// The optional native host requires opting in with `WindowOptions::ime_allowed`.
/// Preedit is provisional and must not be inserted as committed text. These
/// notifications do not provide candidate-window positioning or qualify a
/// platform's IME behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImeEvent {
    /// The input method became available.
    Enabled,
    /// The input method was disabled; discard any preedit state.
    Disabled,
    /// Provisional composition text and its optional selection or caret.
    Preedit {
        /// The entire provisional string, including an empty clearing update.
        text: String,
        /// Byte offsets into `text`, as reported by the platform.
        ///
        /// `None` means the preedit cursor is hidden. Consumers must validate
        /// boundaries before slicing this string.
        cursor: Option<(usize, usize)>,
    },
    /// Text committed by the input method, delivered once by this bridge.
    Commit(String),
}
