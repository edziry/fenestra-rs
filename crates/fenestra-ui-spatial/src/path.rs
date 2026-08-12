//! Raw local path records.

use crate::geometry_key::SpatialPathKeyV2;
use crate::model::SpatialPointV2;

/// Closed vocabulary for one path verb discriminant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPathVerbKindV2 {
    /// Starts a new subpath.
    MoveTo,
    /// Adds one straight segment.
    LineTo,
    /// Adds one quadratic Bezier segment.
    QuadraticTo,
    /// Adds one cubic Bezier segment.
    CubicTo,
    /// Closes the active subpath.
    Close,
}

impl SpatialPathVerbKindV2 {
    /// Every path verb kind in deterministic format order.
    pub const ALL: [Self; 5] = [
        Self::MoveTo,
        Self::LineTo,
        Self::QuadraticTo,
        Self::CubicTo,
        Self::Close,
    ];
}

/// Raw payload for one local path verb.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPathVerbV2 {
    /// Starts a subpath at one point.
    MoveTo {
        /// Destination point.
        to: SpatialPointV2,
    },
    /// Draws a straight segment to one point.
    LineTo {
        /// Destination point.
        to: SpatialPointV2,
    },
    /// Draws a quadratic Bezier segment.
    QuadraticTo {
        /// Quadratic control point.
        control: SpatialPointV2,
        /// Destination point.
        to: SpatialPointV2,
    },
    /// Draws a cubic Bezier segment.
    CubicTo {
        /// First cubic control point.
        control1: SpatialPointV2,
        /// Second cubic control point.
        control2: SpatialPointV2,
        /// Destination point.
        to: SpatialPointV2,
    },
    /// Closes the active subpath.
    Close,
}

/// Raw range selecting the verbs owned by one path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialPathV2 {
    key: SpatialPathKeyV2,
    verb_start: u32,
    verb_length: u32,
}

impl SpatialPathV2 {
    /// Creates an unvalidated path record.
    #[must_use]
    pub const fn new(key: SpatialPathKeyV2, verb_start: u32, verb_length: u32) -> Self {
        Self {
            key,
            verb_start,
            verb_length,
        }
    }

    /// Returns the dense path key.
    #[must_use]
    pub const fn key(self) -> SpatialPathKeyV2 {
        self.key
    }

    /// Returns the first supplied verb ordinal.
    #[must_use]
    pub const fn verb_start(self) -> u32 {
        self.verb_start
    }

    /// Returns the supplied verb count.
    #[must_use]
    pub const fn verb_length(self) -> u32 {
        self.verb_length
    }
}
