use std::fmt;

/// Typed text preparation failures without user content or font bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextError {
    /// A view contains text but no adapter was supplied.
    EngineUnavailable,
    /// The supplied adapter does not implement intrinsic measurement.
    MeasurementUnavailable,
    /// A measurement width exceeds the signed pixel coordinate domain.
    InvalidMeasurementWidth {
        /// Rejected wrap width.
        width: u32,
    },
    /// Measurement and raster layout disagree for the same content and width.
    InconsistentMeasurement,
    /// A typography value exceeds the supported domain.
    InvalidStyle {
        /// Invalid typography property.
        property: &'static str,
        /// Rejected value.
        value: u32,
    },
    /// Text preparation exceeded an inclusive bound.
    LimitExceeded {
        /// Bounded resource.
        resource: &'static str,
        /// Requested count.
        actual: usize,
        /// Inclusive maximum.
        limit: usize,
    },
    /// Raster dimensions, bytes or premultiplied channels are invalid.
    InvalidRaster,
    /// Text metrics contain invalid numbers or inconsistent counts.
    InvalidMetrics,
    /// The adapter cannot supply a usable font.
    FontUnavailable,
    /// The configured font set has no glyph for part of the request.
    MissingGlyphs {
        /// Number of missing glyphs.
        count: usize,
    },
    /// A private adapter failed; details must not contain user text or font data.
    Engine(String),
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EngineUnavailable => f.write_str("text requires an application text engine"),
            Self::MeasurementUnavailable => {
                f.write_str("text engine cannot measure intrinsic text")
            }
            Self::InvalidMeasurementWidth { width } => {
                write!(f, "invalid text measurement width: {width}")
            }
            Self::InconsistentMeasurement => {
                f.write_str("text measurement and raster layout disagree")
            }
            Self::InvalidStyle { property, value } => write!(f, "invalid text {property}: {value}"),
            Self::LimitExceeded {
                resource,
                actual,
                limit,
            } => write!(
                f,
                "{resource} limit exceeded: requested {actual}, maximum {limit}"
            ),
            Self::InvalidRaster => f.write_str("invalid text raster"),
            Self::InvalidMetrics => f.write_str("invalid text metrics"),
            Self::FontUnavailable => f.write_str("no usable text font"),
            Self::MissingGlyphs { count } => write!(f, "font set is missing {count} glyphs"),
            Self::Engine(reason) => write!(f, "text engine failed: {reason}"),
        }
    }
}

impl std::error::Error for TextError {}
