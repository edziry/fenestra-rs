use std::fmt;

/// Validation and content-limit failures from owned text editing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditingError {
    /// A selection endpoint exceeds the text's byte length.
    OffsetOutOfBounds {
        /// The rejected UTF-8 byte offset.
        offset: usize,
        /// The buffer's UTF-8 byte length.
        len: usize,
    },
    /// A selection endpoint is inside an extended grapheme or UTF-8 code point.
    InvalidGraphemeBoundary {
        /// The rejected UTF-8 byte offset.
        offset: usize,
    },
    /// The requested content exceeds the inclusive UTF-8 byte limit.
    LimitExceeded {
        /// The configured content byte limit.
        limit: usize,
        /// The resulting content byte length that was rejected.
        actual: usize,
    },
    /// The resulting content byte length cannot be represented.
    CapacityOverflow,
}

impl fmt::Display for EditingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OffsetOutOfBounds { offset, len } => {
                write!(
                    f,
                    "selection byte offset {offset} exceeds text length {len}"
                )
            }
            Self::InvalidGraphemeBoundary { offset } => {
                write!(
                    f,
                    "selection byte offset {offset} is not a grapheme boundary"
                )
            }
            Self::LimitExceeded { limit, actual } => {
                write!(
                    f,
                    "text byte limit exceeded: requested {actual}, maximum {limit}"
                )
            }
            Self::CapacityOverflow => f.write_str("text byte length overflow"),
        }
    }
}

impl std::error::Error for EditingError {}
