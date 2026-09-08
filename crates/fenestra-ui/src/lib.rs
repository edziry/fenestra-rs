#![forbid(unsafe_code)]

//! Typed native application views backed by the Fenestra runtime.
//!
//! This unpublished API is experimental. Construction and style are expressed
//! with named elements; internal schema and spatial identities stay private.

mod application;
mod editing;
mod error;
mod frame;
mod limits;
mod lower;
mod model;
mod style;
mod text;

pub use application::Application;
pub use editing::{EditingError, Selection, TextBuffer};
pub use error::Error;
pub use frame::{Bounds, Raster, Size};
pub use limits::Limits;
pub use model::{Element, View};
pub use style::{Color, Style};
pub use text::{
    TextEngine, TextError, TextLayout, TextLimits, TextMeasureRequest, TextMetrics, TextRequest,
    TextStyle,
};

/// Compiles an authored view into the public application constructors.
pub use fenestra_ui_macros::ui;

/// Optional native window hosting for application content.
#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
pub mod native;

#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
mod window;

#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
pub use window::Event;
