use std::ops::Range;

use crate::aabb::SpatialAabbV2;
use crate::brush::SpatialRgba8V2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::geometry_kernel::{
    FlattenedPathK2, ValidatedCircleK1, ValidatedRectK1, ValidatedStrokeK1,
};
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::limits::SpatialLimitsV2;
use crate::model::{Affine2V2, SpatialPointV2, SpatialScalarV2, SpatialViewportV2};

pub(super) struct PreparedSpatialState {
    pub(super) viewport: SpatialViewportV2,
    pub(super) limits: SpatialLimitsV2,
    pub(super) topology: Box<[PreparedTopologyNode]>,
    pub(super) paths: Box<[PreparedPathPlan]>,
    pub(super) shapes: Box<[PreparedShapePlan]>,
    pub(super) brushes: Box<[PreparedBrushPlan]>,
    pub(super) images: Box<[PreparedImagePlan]>,
    pub(super) clips: Box<[PreparedClipPlan]>,
    pub(super) paints: Box<[PreparedPaintPlan]>,
    pub(super) hits: Box<[PreparedHitPlan]>,
    pub(super) semantics: Box<[PreparedSemanticPlan]>,
    pub(super) base_geometry: Box<[PreparedBaseGeometry]>,
    pub(super) world_transforms: Box<[Affine2V2]>,
    pub(super) world_aabbs: PreparedWorldAabbs,
    pub(super) effective_clip_aabbs: Box<[SpatialAabbV2]>,
}

pub(super) struct PreparedTopologyNode {
    pub(super) parent: Option<u32>,
    pub(super) depth: usize,
}

pub(super) struct PreparedPathPlan {
    pub(super) verb_range: Range<usize>,
    pub(super) verb_count: usize,
    pub(super) subpath_count: usize,
    pub(super) flattened: FlattenedPathK2,
}

pub(super) struct PreparedShapePlan {
    pub(super) owner: u32,
    pub(super) geometry: PreparedShapeGeometry,
    pub(super) base_bounds: SpatialAabbV2,
    pub(super) fill_clip_bounds: SpatialAabbV2,
}

pub(super) enum PreparedShapeGeometry {
    Rect { rect: ValidatedRectK1 },
    Circle { circle: ValidatedCircleK1 },
    Polygon { point_range: Range<usize> },
    Path { path: u32 },
}

pub(super) struct PreparedBrushPlan {
    pub(super) gradient_range: Option<Range<usize>>,
    pub(super) content: PreparedBrushContent,
}

pub(super) enum PreparedBrushContent {
    Solid(SpatialRgba8V2),
    LinearGradient {
        start: SpatialPointV2,
        end: SpatialPointV2,
        stops: Box<[(u16, SpatialRgba8V2)]>,
    },
}

pub(super) struct PreparedImagePlan {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) stride: u32,
}

pub(super) struct PreparedClipPlan {
    pub(super) owner: u32,
    pub(super) parent: Option<u32>,
    pub(super) shape: u32,
    pub(super) fill_rule: SpatialFillRuleV2,
    pub(super) depth: usize,
}

pub(super) struct PreparedPaintPlan {
    pub(super) owner: u32,
    pub(super) item_ordinal: u32,
    pub(super) content: PreparedPaintContent,
    pub(super) local_bounds: SpatialAabbV2,
}

pub(super) enum PreparedPaintContent {
    Coverage {
        coverage: PreparedCoverage,
        brush: u32,
        opacity: u8,
        clip: Option<u32>,
    },
    Image {
        image: u32,
        source: SpatialImageSourceRectV2,
        destination: SpatialImageDestinationRectV2,
        opacity: u8,
        clip: Option<u32>,
    },
}

pub(super) enum PreparedCoverage {
    Fill {
        shape: u32,
        rule: SpatialFillRuleV2,
    },
    RoundStroke {
        shape: u32,
        stroke: ValidatedStrokeK1,
    },
}

pub(super) struct PreparedHitPlan {
    pub(super) owner: u32,
    pub(super) item_ordinal: u32,
    pub(super) coverage: PreparedCoverage,
    pub(super) input_policy: SpatialInputPolicyV2,
    pub(super) clip: Option<u32>,
    pub(super) local_bounds: SpatialAabbV2,
}

pub(super) struct PreparedSemanticPlan {
    pub(super) owner: u32,
    pub(super) item_ordinal: u32,
    pub(super) shape: u32,
    pub(super) fill_rule: SpatialFillRuleV2,
    pub(super) clip: Option<u32>,
}

pub(super) struct PreparedBaseGeometry {
    pub(super) x: SpatialScalarV2,
    pub(super) y: SpatialScalarV2,
    pub(super) width: i32,
    pub(super) height: i32,
}

pub(super) struct PreparedWorldAabbs {
    pub(super) geometry: Box<[SpatialAabbV2]>,
    pub(super) clips: Box<[SpatialAabbV2]>,
    pub(super) paints: Box<[SpatialAabbV2]>,
    pub(super) hits: Box<[SpatialAabbV2]>,
    pub(super) semantics: Box<[SpatialAabbV2]>,
}
