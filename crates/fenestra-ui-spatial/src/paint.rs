//! Raw coverage and image paint items.

use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
use crate::coverage::SpatialCoverageV2;
use crate::geometry_key::SpatialClipKeyV2;
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::model::SpatialNodeKeyV2;

/// Closed vocabulary for one paint discriminant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPaintKindV2 {
    /// Paints shape coverage with one brush.
    CoveragePaint,
    /// Paints one image into a local destination rectangle.
    ImagePaint,
}

impl SpatialPaintKindV2 {
    /// Every paint kind in deterministic format order.
    pub const ALL: [Self; 2] = [Self::CoveragePaint, Self::ImagePaint];
}

/// Raw exhaustively matchable paint payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPaintContentV2 {
    /// Paints independent local coverage with one referenced brush.
    CoveragePaint {
        /// Local geometry coverage.
        coverage: SpatialCoverageV2,
        /// Referenced brush key.
        brush: SpatialBrushKeyV2,
        /// Raw opacity.
        opacity: u8,
        /// Optional referenced clip key.
        clip: Option<SpatialClipKeyV2>,
    },
    /// Paints a source region from one referenced image.
    ImagePaint {
        /// Referenced image key.
        image: SpatialImageKeyV2,
        /// Raw image-space source rectangle.
        source: SpatialImageSourceRectV2,
        /// Raw local destination rectangle.
        destination: SpatialImageDestinationRectV2,
        /// Raw opacity.
        opacity: u8,
        /// Optional referenced clip key.
        clip: Option<SpatialClipKeyV2>,
    },
}

/// Raw paint item owned by one spatial node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialPaintV2 {
    owner: SpatialNodeKeyV2,
    item_ordinal: u32,
    content: SpatialPaintContentV2,
}

impl SpatialPaintV2 {
    /// Creates an unvalidated paint item.
    #[must_use]
    pub const fn new(
        owner: SpatialNodeKeyV2,
        item_ordinal: u32,
        content: SpatialPaintContentV2,
    ) -> Self {
        Self {
            owner,
            item_ordinal,
            content,
        }
    }

    /// Returns the owning spatial node key.
    #[must_use]
    pub const fn owner(self) -> SpatialNodeKeyV2 {
        self.owner
    }

    /// Returns the raw owner-local item ordinal.
    #[must_use]
    pub const fn item_ordinal(self) -> u32 {
        self.item_ordinal
    }

    /// Returns the raw paint payload.
    #[must_use]
    pub const fn content(self) -> SpatialPaintContentV2 {
        self.content
    }
}
