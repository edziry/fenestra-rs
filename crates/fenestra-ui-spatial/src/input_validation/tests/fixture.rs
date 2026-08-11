use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutPaddingV1};

use crate::aggregate_input::SpatialInputV2;
use crate::brush::{SpatialBrushContentV2, SpatialBrushV2, SpatialGradientStopV2, SpatialRgba8V2};
use crate::content_input::{SpatialItemInputV2, SpatialResourceInputV2};
use crate::content_item::{SpatialHitV2, SpatialInputPolicyV2, SpatialSemanticGeometryV2};
use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
use crate::coverage::{SpatialClipV2, SpatialCoverageV2, SpatialFillRuleV2};
use crate::geometry_input::SpatialGeometryInputV2;
use crate::geometry_key::{SpatialClipKeyV2, SpatialPathKeyV2, SpatialShapeKeyV2};
use crate::image::SpatialImageV2;
use crate::model::{SpatialNodeKeyV2, SpatialPointV2, SpatialScalarV2, SpatialViewportV2};
use crate::paint::{SpatialPaintContentV2, SpatialPaintV2};
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::shape::{SpatialShapeGeometryV2, SpatialShapeV2};
use crate::topology::{
    SpatialContainerV2, SpatialNodeV2, SpatialPlacementV2, SpatialTopologyInputV2,
};

use super::DIRECT_COUNT;

pub(super) struct RawInputFixture {
    nodes: Vec<SpatialNodeV2>,
    shapes: Vec<SpatialShapeV2>,
    brushes: Vec<SpatialBrushV2>,
    clips: Vec<SpatialClipV2>,
    paint_items: Vec<SpatialPaintV2>,
    hit_items: Vec<SpatialHitV2>,
    semantic_items: Vec<SpatialSemanticGeometryV2>,
    paths: Vec<SpatialPathV2>,
    path_verbs: Vec<SpatialPathVerbV2>,
    polygon_points: Vec<SpatialPointV2>,
    gradient_stops: Vec<SpatialGradientStopV2>,
    images: Vec<SpatialImageV2>,
}

impl RawInputFixture {
    pub(super) fn new(counts: [usize; DIRECT_COUNT]) -> Self {
        Self {
            nodes: vec![malformed_node(); counts[0]],
            shapes: vec![malformed_shape(); counts[1]],
            brushes: vec![malformed_brush(); counts[2]],
            clips: vec![malformed_clip(); counts[3]],
            paint_items: vec![malformed_paint(); counts[4]],
            hit_items: vec![malformed_hit(); counts[5]],
            semantic_items: vec![malformed_semantic(); counts[6]],
            paths: vec![malformed_path(); counts[7]],
            path_verbs: vec![SpatialPathVerbV2::Close; counts[8]],
            polygon_points: vec![malformed_point(); counts[9]],
            gradient_stops: vec![malformed_stop(); counts[10]],
            images: (0..counts[11]).map(|_| malformed_image()).collect(),
        }
    }

    pub(super) fn with_nodes(nodes: Vec<SpatialNodeV2>) -> Self {
        let mut fixture = Self::new([1; DIRECT_COUNT]);
        fixture.nodes = nodes;
        fixture
    }

    pub(super) fn input(&self) -> SpatialInputV2<'_> {
        SpatialInputV2::new(
            SpatialTopologyInputV2::new(SpatialViewportV2::new(-1, -1), &self.nodes),
            SpatialGeometryInputV2::new(
                &self.polygon_points,
                &self.path_verbs,
                &self.paths,
                &self.shapes,
                &self.clips,
            ),
            SpatialResourceInputV2::new(&self.gradient_stops, &self.brushes, &self.images),
            SpatialItemInputV2::new(&self.paint_items, &self.hit_items, &self.semantic_items),
        )
    }
}

fn malformed_point() -> SpatialPointV2 {
    SpatialPointV2::new(
        SpatialScalarV2::new(i64::MAX),
        SpatialScalarV2::new(i64::MIN),
    )
}

fn malformed_node() -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(u32::MAX),
        Some(SpatialNodeKeyV2::new(u32::MAX)),
        SpatialPlacementV2::Root,
        SpatialContainerV2::new(
            LayoutAxisV1::Column,
            LayoutPaddingV1::new(-1, -1, -1, -1),
            -1,
        ),
    )
}

fn malformed_shape() -> SpatialShapeV2 {
    SpatialShapeV2::new(
        SpatialShapeKeyV2::new(u32::MAX),
        SpatialNodeKeyV2::new(0),
        SpatialShapeGeometryV2::Rect {
            origin: malformed_point(),
            width: SpatialScalarV2::new(-1),
            height: SpatialScalarV2::new(-1),
        },
    )
}

fn malformed_brush() -> SpatialBrushV2 {
    SpatialBrushV2::new(
        SpatialBrushKeyV2::new(u32::MAX),
        SpatialBrushContentV2::Solid {
            color: SpatialRgba8V2::new(255, 255, 255, 0),
        },
    )
}

fn malformed_clip() -> SpatialClipV2 {
    SpatialClipV2::new(
        SpatialClipKeyV2::new(u32::MAX),
        SpatialNodeKeyV2::new(0),
        Some(SpatialClipKeyV2::new(u32::MAX)),
        SpatialShapeKeyV2::new(u32::MAX),
        SpatialFillRuleV2::NonZero,
    )
}

fn malformed_coverage() -> SpatialCoverageV2 {
    SpatialCoverageV2::Fill {
        shape: SpatialShapeKeyV2::new(u32::MAX),
        rule: SpatialFillRuleV2::NonZero,
    }
}

fn malformed_paint() -> SpatialPaintV2 {
    SpatialPaintV2::new(
        SpatialNodeKeyV2::new(0),
        u32::MAX,
        SpatialPaintContentV2::CoveragePaint {
            coverage: malformed_coverage(),
            brush: SpatialBrushKeyV2::new(u32::MAX),
            opacity: 255,
            clip: Some(SpatialClipKeyV2::new(u32::MAX)),
        },
    )
}

fn malformed_hit() -> SpatialHitV2 {
    SpatialHitV2::new(
        SpatialNodeKeyV2::new(0),
        u32::MAX,
        malformed_coverage(),
        Some(SpatialClipKeyV2::new(u32::MAX)),
        SpatialInputPolicyV2::Accept,
    )
}

fn malformed_semantic() -> SpatialSemanticGeometryV2 {
    SpatialSemanticGeometryV2::new(
        SpatialNodeKeyV2::new(0),
        u32::MAX,
        SpatialShapeKeyV2::new(u32::MAX),
        SpatialFillRuleV2::NonZero,
        Some(SpatialClipKeyV2::new(u32::MAX)),
    )
}

fn malformed_path() -> SpatialPathV2 {
    SpatialPathV2::new(SpatialPathKeyV2::new(u32::MAX), u32::MAX, u32::MAX)
}

fn malformed_stop() -> SpatialGradientStopV2 {
    SpatialGradientStopV2::new(1, SpatialRgba8V2::new(255, 255, 255, 0))
}

fn malformed_image() -> SpatialImageV2 {
    SpatialImageV2::new(
        SpatialImageKeyV2::new(u32::MAX),
        0,
        0,
        u32::MAX,
        Vec::new().into_boxed_slice(),
    )
}
