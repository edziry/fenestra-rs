use std::fmt;

/// Font registration failures containing no font data, paths or user text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FontError {
    /// At least one explicit font is required.
    EmptySet,
    /// A constructor resource budget was exceeded before font copying.
    LimitExceeded {
        /// Resource whose inclusive maximum was exceeded.
        resource: &'static str,
        /// Requested quantity.
        actual: usize,
        /// Inclusive maximum.
        limit: usize,
    },
    /// A font has invalid structure or cannot supply usable metadata.
    InvalidFont {
        /// Zero-based position in the supplied font set.
        index: usize,
    },
    /// Font collections are outside the single-face admission.
    CollectionUnsupported {
        /// Zero-based position in the supplied font set.
        index: usize,
    },
    /// Color and bitmap font tables are outside the outline-only admission.
    ColorOrBitmapUnsupported {
        /// Zero-based position in the supplied font set.
        index: usize,
    },
    /// The font has no supported monochrome outline tables.
    OutlinesUnavailable {
        /// Zero-based position in the supplied font set.
        index: usize,
    },
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySet => f.write_str("at least one explicit text font is required"),
            Self::LimitExceeded {
                resource,
                actual,
                limit,
            } => write!(
                f,
                "{resource} limit exceeded: requested {actual}, maximum {limit}"
            ),
            Self::InvalidFont { index } => write!(f, "font {index} is invalid or unusable"),
            Self::CollectionUnsupported { index } => {
                write!(f, "font {index} is an unsupported collection")
            }
            Self::ColorOrBitmapUnsupported { index } => {
                write!(f, "font {index} uses unsupported color or bitmap tables")
            }
            Self::OutlinesUnavailable { index } => {
                write!(f, "font {index} has no supported outlines")
            }
        }
    }
}

impl std::error::Error for FontError {}
