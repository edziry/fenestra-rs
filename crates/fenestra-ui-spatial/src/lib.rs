#![forbid(unsafe_code)]

//! Unpublished candidate-neutral hybrid spatial boundary for Fenestra.

mod aabb;
mod affine;
mod aggregate_input;
mod brush;
mod content_diagnostic;
mod content_error;
mod content_input;
mod content_item;
mod content_key;
mod coverage;
mod direct_counts;
mod error;
mod geometry_field;
mod geometry_input;
mod geometry_kernel;
mod geometry_key;
mod image;
mod input_validation;
mod item_field;
mod limits;
mod model;
mod numeric;
mod numeric_error;
mod output_aabb;
mod output_field;
mod output_geometry;
mod output_item;
mod output_view;
mod owned_input;
mod paint;
mod paint_kernel;
mod path;
mod reference_raster;
mod resolve_error;
mod shape;
mod topology;
mod vocabulary;

/// Reserved unstable surface for hybrid spatial feasibility work.
#[doc(hidden)]
pub mod prototype {
    pub use crate::aabb::SpatialAabbV2;
    pub use crate::aggregate_input::SpatialInputV2;
    pub use crate::brush::{
        SpatialBrushContentV2, SpatialBrushKindV2, SpatialBrushV2, SpatialGradientStopV2,
        SpatialRgba8V2,
    };
    pub use crate::content_diagnostic::{
        SpatialClipErrorV2, SpatialContentReferenceV2, SpatialGradientErrorV2, SpatialImageErrorV2,
        SpatialKeyedContentTableV2, SpatialOrderedItemTableV2, SpatialPathGrammarErrorV2,
        SpatialPayloadTableV2, SpatialShapeErrorV2, SpatialStrokeErrorV2,
    };
    pub use crate::content_error::SpatialContentErrorKindV2;
    pub use crate::content_input::{SpatialItemInputV2, SpatialResourceInputV2};
    pub use crate::content_item::{SpatialHitV2, SpatialInputPolicyV2, SpatialSemanticGeometryV2};
    pub use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
    pub use crate::coverage::{
        SpatialClipV2, SpatialCoverageKindV2, SpatialCoverageV2, SpatialFillRuleV2,
    };
    pub use crate::direct_counts::preflight_spatial_direct_counts_v2;
    pub use crate::error::{
        SpatialContainerErrorKindV2, SpatialDependencyErrorKindV2, SpatialErrorLocationV2,
        SpatialInputErrorKindV2, SpatialLayoutDimensionErrorKindV2,
    };
    pub use crate::geometry_field::{
        SpatialBrushFieldV2, SpatialColorChannelV2, SpatialGradientStopFieldV2,
        SpatialImageFieldV2, SpatialPathFieldV2, SpatialPathVerbFieldV2,
        SpatialPolygonPointFieldV2, SpatialShapeFieldV2,
    };
    pub use crate::geometry_input::SpatialGeometryInputV2;
    pub use crate::geometry_key::{SpatialClipKeyV2, SpatialPathKeyV2, SpatialShapeKeyV2};
    pub use crate::image::{
        SpatialImageDestinationRectV2, SpatialImageSourceRectV2, SpatialImageV2,
    };
    pub use crate::input_validation::{
        PreparedSpatialV2, SpatialHitResultV2, SpatialResolvedSnapshotV2,
        materialize_reference_spatial_v2, prepare_spatial_v2, resolve_spatial_v2,
        validate_spatial_output_v2,
    };
    pub use crate::item_field::{
        SpatialClipFieldV2, SpatialHitFieldV2, SpatialPaintFieldV2, SpatialSemanticFieldV2,
    };
    pub use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2, SpatialLimitsV2};
    pub use crate::model::{
        Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetKindV2, SpatialAnchorTargetV2,
        SpatialAnchorV2, SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialOffsetV2,
        SpatialPointV2, SpatialScalarV2, SpatialViewportV2,
    };
    pub use crate::numeric::round_ratio_v2;
    pub use crate::numeric_error::{SpatialArithmeticOperationV2, SpatialTransformErrorKindV2};
    pub use crate::output_aabb::SpatialOutputAabbV2;
    pub use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
    pub use crate::output_geometry::{SpatialClipOutputRecordV2, SpatialGeometryOutputRecordV2};
    pub use crate::output_item::{
        SpatialHitOutputRecordV2, SpatialPaintOutputRecordV2, SpatialPaintOutputReferenceV2,
        SpatialSemanticOutputRecordV2,
    };
    pub use crate::output_view::SpatialOutputV2;
    pub use crate::owned_input::SpatialOwnedInputV2;
    pub use crate::paint::{SpatialPaintContentV2, SpatialPaintKindV2, SpatialPaintV2};
    pub use crate::path::{SpatialPathV2, SpatialPathVerbKindV2, SpatialPathVerbV2};
    pub use crate::reference_raster::{
        REGISTERED_REFERENCE_RASTER_LIMITS_V2, ReferenceRasterErrorKindV2, ReferenceRasterErrorV2,
        ReferenceRasterLimitKindV2, ReferenceRasterLimitsV2, ReferenceRasterV2,
    };
    pub use crate::resolve_error::{
        SpatialLayoutErrorKindV2, SpatialOutputErrorKindV2, SpatialResolveErrorKindV2,
        SpatialResolveErrorV2,
    };
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
