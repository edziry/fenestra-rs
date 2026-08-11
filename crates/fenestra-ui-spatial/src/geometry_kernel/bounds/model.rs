use crate::aabb::SpatialAabbV2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DerivedLocalBoundsK3 {
    base: SpatialAabbV2,
    empty_fill_clip: bool,
}

impl DerivedLocalBoundsK3 {
    pub(super) const fn new(base: SpatialAabbV2, empty_fill_clip: bool) -> Self {
        Self {
            base,
            empty_fill_clip,
        }
    }

    pub(crate) const fn base_bounds(self) -> SpatialAabbV2 {
        self.base
    }

    pub(super) const fn fill_clip_bounds(self) -> SpatialAabbV2 {
        if self.empty_fill_clip {
            SpatialAabbV2::empty()
        } else {
            self.base
        }
    }
}
