#![forbid(unsafe_code)]

//! Unpublished candidate-neutral hybrid spatial boundary for Fenestra.

mod aabb;
mod affine;
mod coverage;
mod error;
mod geometry_input;
mod geometry_kernel;
mod geometry_key;
mod limits;
mod model;
mod numeric;
mod numeric_error;
mod path;
mod shape;
mod topology;
mod vocabulary;

/// Reserved unstable surface for hybrid spatial feasibility work.
#[doc(hidden)]
pub mod prototype {
    pub use crate::aabb::SpatialAabbV2;
    pub use crate::coverage::{
        SpatialClipV2, SpatialCoverageKindV2, SpatialCoverageV2, SpatialFillRuleV2,
    };
    pub use crate::error::{
        SpatialContainerErrorKindV2, SpatialDependencyErrorKindV2, SpatialErrorLocationV2,
        SpatialInputErrorKindV2, SpatialLayoutDimensionErrorKindV2,
    };
    pub use crate::geometry_input::SpatialGeometryInputV2;
    pub use crate::geometry_key::{SpatialClipKeyV2, SpatialPathKeyV2, SpatialShapeKeyV2};
    pub use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2, SpatialLimitsV2};
    pub use crate::model::{
        Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetKindV2, SpatialAnchorTargetV2,
        SpatialAnchorV2, SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialOffsetV2,
        SpatialPointV2, SpatialScalarV2, SpatialViewportV2,
    };
    pub use crate::numeric::round_ratio_v2;
    pub use crate::numeric_error::{SpatialArithmeticOperationV2, SpatialTransformErrorKindV2};
    pub use crate::path::{SpatialPathV2, SpatialPathVerbKindV2, SpatialPathVerbV2};
    pub use crate::shape::{SpatialShapeGeometryV2, SpatialShapeKindV2, SpatialShapeV2};
    pub use crate::topology::{
        SpatialContainerV2, SpatialFreePlacementV2, SpatialLayoutPlacementV2, SpatialNodeV2,
        SpatialPlacementKindV2, SpatialPlacementV2, SpatialTopologyInputV2,
    };
    pub use crate::vocabulary::{
        SpatialAffineComponentV2, SpatialAxisV2, SpatialExtentV2, SpatialNodeFieldV2,
        SpatialTransformScalarFieldV2, SpatialTransformStageV2,
    };
}
