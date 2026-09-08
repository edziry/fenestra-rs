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

pub use crate::{ImeEvent, InputEvent as WindowEvent, Key, KeyState, KeyboardInput, Modifiers};

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
