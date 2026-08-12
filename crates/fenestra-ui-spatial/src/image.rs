//! Owned normalized image records and raw image rectangles.

use crate::content_key::SpatialImageKeyV2;
use crate::model::SpatialScalarV2;

/// Owned raw version-2 image record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpatialImageV2 {
    key: SpatialImageKeyV2,
    width: u32,
    height: u32,
    stride: u32,
    bytes: Box<[u8]>,
}

impl SpatialImageV2 {
    /// Creates an unvalidated image record and takes ownership of its exact bytes.
    #[must_use]
    pub fn new(
        key: SpatialImageKeyV2,
        width: u32,
        height: u32,
        stride: u32,
        bytes: Box<[u8]>,
    ) -> Self {
        Self {
            key,
            width,
            height,
            stride,
            bytes,
        }
    }

    /// Returns the dense image key.
    #[must_use]
    pub const fn key(&self) -> SpatialImageKeyV2 {
        self.key
    }

    /// Returns the raw image width.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the raw image height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the raw row stride in bytes.
    #[must_use]
    pub const fn stride(&self) -> u32 {
        self.stride
    }

    /// Returns the exact owned byte sequence.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Raw integer source rectangle inside one image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialImageSourceRectV2 {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl SpatialImageSourceRectV2 {
    /// Creates an unvalidated image source rectangle.
    #[must_use]
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the raw source x coordinate.
    #[must_use]
    pub const fn x(self) -> u32 {
        self.x
    }

    /// Returns the raw source y coordinate.
    #[must_use]
    pub const fn y(self) -> u32 {
        self.y
    }

    /// Returns the raw source width.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Returns the raw source height.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }
}

/// Raw local destination rectangle for one image paint item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialImageDestinationRectV2 {
    x: SpatialScalarV2,
    y: SpatialScalarV2,
    width: SpatialScalarV2,
    height: SpatialScalarV2,
}

impl SpatialImageDestinationRectV2 {
    /// Creates an unvalidated image destination rectangle.
    #[must_use]
    pub const fn new(
        x: SpatialScalarV2,
        y: SpatialScalarV2,
        width: SpatialScalarV2,
        height: SpatialScalarV2,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the raw destination x coordinate.
    #[must_use]
    pub const fn x(self) -> SpatialScalarV2 {
        self.x
    }

    /// Returns the raw destination y coordinate.
    #[must_use]
    pub const fn y(self) -> SpatialScalarV2 {
        self.y
    }

    /// Returns the raw destination width.
    #[must_use]
    pub const fn width(self) -> SpatialScalarV2 {
        self.width
    }

    /// Returns the raw destination height.
    #[must_use]
    pub const fn height(self) -> SpatialScalarV2 {
        self.height
    }
}
