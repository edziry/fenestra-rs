//! Replaceable text shaping and outline rasterization for owned Fenestra views.
//!
//! Applications supply the complete font set as owned bytes. No host fonts are
//! discovered. The first font is preferred and the remaining fonts are tried in
//! caller order. Unsupported coverage fails explicitly. See ADR 0001 for the
//! provisional dependency admission and input-scope limitations.

#![forbid(unsafe_code)]

mod error;
mod fonts;
mod raster;
mod renderer;

pub use error::FontError;
pub use renderer::TextRenderer;
