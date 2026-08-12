//! Bounded deterministic CPU reference-raster values.

use std::error::Error;
use std::fmt;

use crate::error::SpatialErrorLocationV2;

/// Closed capacity vocabulary for version-2 reference rasterization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceRasterLimitKindV2 {
    /// Total output pixels.
    Pixels,
}

impl ReferenceRasterLimitKindV2 {
    /// Every reference-raster limit in validation order.
    pub const ALL: [Self; 1] = [Self::Pixels];

    const fn index(self) -> usize {
        match self {
            Self::Pixels => 0,
        }
    }
}

/// Caller-supplied inclusive capacities for one reference rasterization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReferenceRasterLimitsV2 {
    values: [usize; 1],
}

impl ReferenceRasterLimitsV2 {
    /// Creates a caller-supplied inclusive output-pixel capacity.
    #[must_use]
    pub const fn new(pixels: usize) -> Self {
        Self { values: [pixels] }
    }

    /// Returns the inclusive capacity for one reference-raster limit kind.
    #[must_use]
    pub const fn limit(self, kind: ReferenceRasterLimitKindV2) -> usize {
        self.values[kind.index()]
    }
}

/// Registered bounded conformance profile for version-2 reference rasterization.
///
/// This experiment profile is neither a runtime default nor a product capacity.
pub const REGISTERED_REFERENCE_RASTER_LIMITS_V2: ReferenceRasterLimitsV2 =
    ReferenceRasterLimitsV2::new(4_194_304);

/// Closed failure vocabulary for version-2 reference rasterization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceRasterErrorKindV2 {
    /// A checked caller or allocation capacity was exceeded.
    LimitExceeded(ReferenceRasterLimitKindV2),
}

impl ReferenceRasterErrorKindV2 {
    /// Every reference-raster failure in validation order.
    pub const ALL: [Self; 1] = [Self::LimitExceeded(ReferenceRasterLimitKindV2::Pixels)];
}

/// Stored redacted diagnostic for one failed reference rasterization.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ReferenceRasterErrorV2 {
    kind: ReferenceRasterErrorKindV2,
    location: SpatialErrorLocationV2,
    observed: Option<u128>,
    maximum: Option<u128>,
}

impl ReferenceRasterErrorV2 {
    /// Creates the single closed limit diagnostic with widened evidence.
    #[must_use]
    pub(crate) const fn limit_exceeded(observed: u128, maximum: u128) -> Self {
        Self {
            kind: ReferenceRasterErrorKindV2::LimitExceeded(ReferenceRasterLimitKindV2::Pixels),
            location: SpatialErrorLocationV2::Input,
            observed: Some(observed),
            maximum: Some(maximum),
        }
    }

    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> ReferenceRasterErrorKindV2 {
        self.kind
    }

    /// Returns the trusted diagnostic location.
    #[must_use]
    pub const fn location(self) -> SpatialErrorLocationV2 {
        self.location
    }

    /// Returns the observed widened pixel count.
    #[must_use]
    pub const fn observed(self) -> Option<u128> {
        self.observed
    }

    /// Returns the effective widened maximum pixel count.
    #[must_use]
    pub const fn maximum(self) -> Option<u128> {
        self.maximum
    }
}

impl fmt::Display for ReferenceRasterErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ReferenceRasterErrorKindV2::LimitExceeded(_) => {
                formatter.write_str("reference-raster-error(limit-exceeded)")
            }
        }
    }
}

impl fmt::Debug for ReferenceRasterErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ReferenceRasterErrorV2({self})")
    }
}

impl Error for ReferenceRasterErrorV2 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Owned packed premultiplied encoded-sRGB RGBA8 reference raster.
pub struct ReferenceRasterV2 {
    width: u32,
    height: u32,
    stride: u64,
    bytes: Box<[u8]>,
}

impl ReferenceRasterV2 {
    /// Creates an owned raster from trusted packed bytes.
    #[must_use]
    pub(crate) fn from_bytes(width: u32, height: u32, bytes: Box<[u8]>) -> Self {
        Self {
            width,
            height,
            stride: u64::from(width) * 4,
            bytes,
        }
    }

    /// Returns the logical output width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the logical output height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the packed row stride in bytes.
    #[must_use]
    pub const fn stride(&self) -> u64 {
        self.stride
    }

    /// Returns the owned packed premultiplied encoded-sRGB RGBA8 bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
