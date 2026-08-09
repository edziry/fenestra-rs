/// Bootstrap input handling policy carried as fixture data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputPolicy {
    /// Accept input for the authored element.
    Accept,
    /// Ignore input for the authored element.
    Ignore,
}

/// Closed value types supported by the provisional schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueType {
    /// Boolean fixture data.
    Bool,
    /// Signed 32-bit scalar without units.
    ScalarI32,
    /// Four color bytes without color-space semantics.
    Rgba8,
    /// Bootstrap input policy fixture data.
    InputPolicy,
}

/// Closed property values supported by the provisional schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyValue {
    /// Boolean value.
    Bool(bool),
    /// Signed 32-bit scalar value.
    ScalarI32(i32),
    /// Four uninterpreted color bytes.
    Rgba8([u8; 4]),
    /// Bootstrap input policy value.
    InputPolicy(InputPolicy),
}

impl PropertyValue {
    /// Returns the closed type represented by this value.
    #[must_use]
    pub const fn value_type(&self) -> ValueType {
        match self {
            Self::Bool(_) => ValueType::Bool,
            Self::ScalarI32(_) => ValueType::ScalarI32,
            Self::Rgba8(_) => ValueType::Rgba8,
            Self::InputPolicy(_) => ValueType::InputPolicy,
        }
    }
}
