#![forbid(unsafe_code)]

//! Unpublished candidate-neutral hybrid spatial boundary for Fenestra.

mod error;
mod limits;
mod model;
mod topology;
mod vocabulary;

/// Reserved unstable surface for hybrid spatial feasibility work.
#[doc(hidden)]
pub mod prototype {
    pub use crate::error::{
        SpatialContainerErrorKindV2, SpatialDependencyErrorKindV2, SpatialErrorLocationV2,
        SpatialInputErrorKindV2, SpatialLayoutDimensionErrorKindV2,
    };
    pub use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2, SpatialLimitsV2};
    pub use crate::model::{
        Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetKindV2, SpatialAnchorTargetV2,
        SpatialAnchorV2, SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialOffsetV2,
        SpatialPointV2, SpatialScalarV2, SpatialViewportV2,
    };
    pub use crate::topology::{
        SpatialContainerV2, SpatialFreePlacementV2, SpatialLayoutPlacementV2, SpatialNodeV2,
        SpatialPlacementKindV2, SpatialPlacementV2, SpatialTopologyInputV2,
    };
    pub use crate::vocabulary::{SpatialAxisV2, SpatialExtentV2, SpatialNodeFieldV2};
}
