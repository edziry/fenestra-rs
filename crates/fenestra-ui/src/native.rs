//! Optional native Windows and Linux Wayland window presentation.
//!
//! Enable `native` to run a [`WindowContent`](crate::native::WindowContent) implementation. Input,
//! resize callbacks, and raster sizes use physical window pixels consistently.

use std::error::Error;
use std::fmt;

use winit::event_loop::EventLoop;

use crate::{Raster, Size};

mod input;
mod presentation;
mod shell;
#[cfg(test)]
mod tests;

/// Application input delivered by the native shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WindowEvent {
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
/// Receiving these events requires opting in with `WindowOptions::ime_allowed`.
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

/// Application callbacks required by the native raster window.
pub trait WindowContent {
    /// Error returned by application callbacks.
    type Error: Error;

    /// Commits a nonzero physical window size before the next frame.
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Self::Error>;

    /// Handles one input or closure event.
    fn event(&mut self, event: WindowEvent) -> Result<(), Self::Error>;

    /// Produces premultiplied RGBA8 pixels at the last committed window size.
    fn frame(&self) -> Result<Raster, Self::Error>;

    /// Observes a frame only after native presentation succeeds.
    fn presented(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Window title, initial logical size, IME opt-in, and one-frame smoke mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowOptions {
    title: String,
    size: Size,
    smoke: bool,
    ime_allowed: bool,
}

impl WindowOptions {
    /// Creates an interactive window with a 640 by 420 logical-pixel size.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            size: Size::new(640, 420),
            smoke: false,
            ime_allowed: false,
        }
    }

    /// Sets the initial nonzero size in logical window pixels.
    #[must_use]
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Size::new(width, height);
        self
    }

    /// Exits after the first successful presentation when enabled.
    #[must_use]
    pub const fn smoke(mut self, enabled: bool) -> Self {
        self.smoke = enabled;
        self
    }

    /// Allows native IME notifications for this window. Disabled by default.
    ///
    /// This is a window-wide opt-in, not a focused text control lifecycle.
    /// Candidate-window positioning and platform qualification are deferred.
    #[must_use]
    pub const fn ime_allowed(mut self, allowed: bool) -> Self {
        self.ime_allowed = allowed;
        self
    }
}

/// Failure to create a native window, present pixels, or execute its application.
#[derive(Debug)]
pub enum NativeError<E> {
    /// The native event loop could not be created or completed.
    EventLoop,
    /// The window could not be created or its initial size was zero.
    Window,
    /// The native surface, raster dimensions, or input coordinates were invalid.
    Presenter,
    /// An application callback rejected an event or frame.
    Application(E),
}

impl<E: fmt::Display> fmt::Display for NativeError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EventLoop => formatter.write_str("native event loop failed"),
            Self::Window => formatter.write_str("native window creation failed"),
            Self::Presenter => formatter.write_str("native presentation failed"),
            Self::Application(error) => write!(formatter, "native application failed: {error}"),
        }
    }
}

impl<E: Error + 'static> Error for NativeError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Application(error) => Some(error),
            _ => None,
        }
    }
}

/// Runs one native window on the calling thread until closure or a failure.
///
/// The application remains available to the caller after the window closes.
/// A minimized window suspends resize and frame callbacks until restored.
/// Smoke mode exits after presentation without synthesizing user input.
pub fn run<C: WindowContent>(
    content: &mut C,
    options: WindowOptions,
) -> Result<(), NativeError<C::Error>> {
    if options.size.width() == 0 || options.size.height() == 0 {
        return Err(NativeError::Window);
    }
    let event_loop = EventLoop::new().map_err(|_| NativeError::EventLoop)?;
    let mut application = shell::NativeApplication::new(content, options);
    let result = event_loop.run_app(&mut application);
    if let Some(error) = application.failure {
        return Err(error);
    }
    result.map_err(|_| NativeError::EventLoop)
}
