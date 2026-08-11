//! Dense raw keys for local content tables.

/// Dense pass-local key for one version-2 brush.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpatialBrushKeyV2(u32);

impl SpatialBrushKeyV2 {
    /// Creates an unvalidated brush key.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw brush key.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Dense pass-local key for one version-2 image.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpatialImageKeyV2(u32);

impl SpatialImageKeyV2 {
    /// Creates an unvalidated image key.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw image key.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}
