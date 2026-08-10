/// Version of the experimental authoring input format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthoringFormatVersion(u32);

impl AuthoringFormatVersion {
    /// Creates an authoring format version.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the numeric authoring format version.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Authoring format understood by this experimental compiler boundary.
pub const SUPPORTED_AUTHORING_FORMAT: AuthoringFormatVersion = AuthoringFormatVersion::new(1);
