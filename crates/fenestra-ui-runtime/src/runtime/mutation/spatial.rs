use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

const SPATIAL_RESIZE_INVALIDATION: InvalidationSet = InvalidationSet::NONE
    .union(InvalidationSet::from_class(InvalidationClass::Layout))
    .union(InvalidationSet::from_class(InvalidationClass::Semantics))
    .union(InvalidationSet::from_class(InvalidationClass::HitTest))
    .union(InvalidationSet::from_class(InvalidationClass::Paint))
    .union(InvalidationSet::from_class(InvalidationClass::Composition));

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpatialViewportChange {
    pub(crate) old_viewport: SpatialViewportV2,
    pub(crate) new_viewport: SpatialViewportV2,
}

impl SpatialViewportChange {
    pub(crate) const fn invalidation(self) -> InvalidationSet {
        SPATIAL_RESIZE_INVALIDATION
    }
}

/// Borrowed payload for one runtime spatial viewport change.
#[derive(Clone, Copy)]
pub struct SpatialViewportChangeViewV2<'a> {
    change: &'a SpatialViewportChange,
}

impl<'a> SpatialViewportChangeViewV2<'a> {
    pub(crate) const fn new(change: &'a SpatialViewportChange) -> Self {
        Self { change }
    }

    /// Returns the viewport extent before the transaction.
    #[must_use]
    pub const fn old_viewport(self) -> SpatialViewportV2 {
        self.change.old_viewport
    }

    /// Returns the final coalesced viewport extent.
    #[must_use]
    pub const fn new_viewport(self) -> SpatialViewportV2 {
        self.change.new_viewport
    }
}
