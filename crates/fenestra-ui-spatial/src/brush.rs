//! Raw color, gradient-stop, and brush records.

use crate::content_key::SpatialBrushKeyV2;
use crate::model::SpatialPointV2;

/// Four raw encoded 8-bit color channels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialRgba8V2 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl SpatialRgba8V2 {
    /// Creates one raw encoded color.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Returns the red channel.
    #[must_use]
    pub const fn r(self) -> u8 {
        self.r
    }

    /// Returns the green channel.
    #[must_use]
    pub const fn g(self) -> u8 {
        self.g
    }

    /// Returns the blue channel.
    #[must_use]
    pub const fn b(self) -> u8 {
        self.b
    }

    /// Returns the alpha channel.
    #[must_use]
    pub const fn a(self) -> u8 {
        self.a
    }
}

/// Raw stop in one linear-gradient stop range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialGradientStopV2 {
    offset: u16,
    color: SpatialRgba8V2,
}

impl SpatialGradientStopV2 {
    /// Creates an unvalidated gradient stop.
    #[must_use]
    pub const fn new(offset: u16, color: SpatialRgba8V2) -> Self {
        Self { offset, color }
    }

    /// Returns the raw normalized offset.
    #[must_use]
    pub const fn offset(self) -> u16 {
        self.offset
    }

    /// Returns the straight encoded color.
    #[must_use]
    pub const fn color(self) -> SpatialRgba8V2 {
        self.color
    }
}

/// Closed vocabulary for one brush discriminant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialBrushKindV2 {
    /// One constant color.
    Solid,
    /// One linear gradient over a stop range.
    LinearGradient,
}

impl SpatialBrushKindV2 {
    /// Every brush kind in deterministic format order.
    pub const ALL: [Self; 2] = [Self::Solid, Self::LinearGradient];
}

/// Raw exhaustively matchable brush payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialBrushContentV2 {
    /// One straight encoded solid color.
    Solid {
        /// Authored straight encoded color.
        color: SpatialRgba8V2,
    },
    /// One linear gradient over a raw stop range.
    LinearGradient {
        /// First stop-table index.
        stop_start: u32,
        /// Number of stops in the raw range.
        stop_length: u32,
        /// Gradient start point.
        start: SpatialPointV2,
        /// Gradient end point.
        end: SpatialPointV2,
    },
}

/// Raw brush record identified by one dense key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialBrushV2 {
    key: SpatialBrushKeyV2,
    content: SpatialBrushContentV2,
}

impl SpatialBrushV2 {
    /// Creates an unvalidated brush record.
    #[must_use]
    pub const fn new(key: SpatialBrushKeyV2, content: SpatialBrushContentV2) -> Self {
        Self { key, content }
    }

    /// Returns the dense brush key.
    #[must_use]
    pub const fn key(self) -> SpatialBrushKeyV2 {
        self.key
    }

    /// Returns the raw brush payload.
    #[must_use]
    pub const fn content(self) -> SpatialBrushContentV2 {
        self.content
    }
}
