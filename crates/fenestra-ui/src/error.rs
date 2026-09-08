use std::fmt;

/// Failures from view validation, resource limits or application updates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A view or element name is not a nonempty ASCII identifier.
    InvalidName {
        /// The invalid authored name.
        name: String,
    },
    /// Two elements use the same application name.
    DuplicateName {
        /// The repeated name.
        name: String,
    },
    /// A numeric style value is outside its supported domain.
    InvalidStyle {
        /// The element whose style is invalid.
        node: String,
        /// The invalid property.
        property: &'static str,
        /// The rejected value.
        value: i32,
    },
    /// An element contains an unsupported child or style combination.
    InvalidElement {
        /// The element whose structure is invalid.
        node: String,
        /// The violated element rule.
        reason: &'static str,
    },
    /// A caller-supplied resource bound would be exceeded.
    LimitExceeded {
        /// The bounded resource.
        resource: &'static str,
        /// The inclusive bound.
        limit: usize,
        /// The requested amount.
        actual: usize,
    },
    /// Derived capacities cannot be represented by the runtime.
    CapacityOverflow,
    /// Internal typed program validation rejected the view.
    InvalidProgram(String),
    /// The runtime rejected an operation without publishing partial state.
    Runtime(String),
    /// A mutation named an element absent from this application.
    UnknownNode {
        /// The requested element name.
        name: String,
    },
    /// The viewport is empty or exceeds signed coordinate representation.
    InvalidViewport {
        /// Requested pixel width.
        width: u32,
        /// Requested pixel height.
        height: u32,
    },
    /// Raster data does not contain one RGBA8 value for every pixel.
    InvalidRaster,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { name } => write!(f, "invalid element or view name {name:?}"),
            Self::DuplicateName { name } => write!(f, "duplicate element name {name:?}"),
            Self::InvalidStyle {
                node,
                property,
                value,
            } => {
                write!(f, "element {node:?}: {property} cannot be {value}")
            }
            Self::InvalidElement { node, reason } => write!(f, "element {node:?}: {reason}"),
            Self::LimitExceeded {
                resource,
                limit,
                actual,
            } => {
                write!(
                    f,
                    "{resource} limit exceeded: requested {actual}, maximum {limit}"
                )
            }
            Self::CapacityOverflow => f.write_str("application capacity overflow"),
            Self::InvalidProgram(reason) => write!(f, "invalid application program: {reason}"),
            Self::Runtime(reason) => write!(f, "application update failed: {reason}"),
            Self::UnknownNode { name } => write!(f, "unknown element {name:?}"),
            Self::InvalidViewport { width, height } => {
                write!(f, "invalid viewport {width}x{height}")
            }
            Self::InvalidRaster => f.write_str("raster byte count does not match its size"),
        }
    }
}

impl std::error::Error for Error {}
