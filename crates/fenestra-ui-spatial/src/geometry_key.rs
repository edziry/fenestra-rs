//! Dense raw keys for local geometry tables.

/// Dense pass-local key for one version-2 path.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpatialPathKeyV2(u32);

impl SpatialPathKeyV2 {
    /// Creates an unvalidated path key.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw path key.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Dense pass-local key for one version-2 shape.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpatialShapeKeyV2(u32);

impl SpatialShapeKeyV2 {
    /// Creates an unvalidated shape key.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw shape key.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Dense pass-local key for one version-2 clip.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpatialClipKeyV2(u32);

impl SpatialClipKeyV2 {
    /// Creates an unvalidated clip key.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw clip key.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}
