mod derived;
mod resources;

use std::sync::Arc;

use super::PreparedSpatialV2;
use crate::aabb::SpatialAabbV2;
use crate::owned_input::SpatialOwnedInputV2;

impl PreparedSpatialV2 {
    pub(in crate::input_validation) fn source_arc(&self) -> &Arc<SpatialOwnedInputV2> {
        &self.source
    }

    pub(in crate::input_validation) fn effective_clip_identity(
        &self,
    ) -> (*const SpatialAabbV2, usize) {
        (
            self.state.effective_clip_aabbs.as_ptr(),
            self.state.effective_clip_aabbs.len(),
        )
    }
}
