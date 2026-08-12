use fenestra_ui_layout::prototype::{LayoutDimensionV1, ReferenceStackEngineV1};
use fenestra_ui_spatial::prototype::{
    SpatialAnchorTargetV2, SpatialBrushContentV2, SpatialBrushKeyV2, SpatialBrushV2,
    SpatialClipKeyV2, SpatialClipV2, SpatialFillRuleV2, SpatialGradientStopV2, SpatialHitV2,
    SpatialImageKeyV2, SpatialImageV2, SpatialInputPolicyV2, SpatialLayoutPlacementV2,
    SpatialLimitKindV2, SpatialLimitsV2, SpatialNodeKeyV2, SpatialNodeV2, SpatialPaintContentV2,
    SpatialPaintV2, SpatialPathKeyV2, SpatialPathV2, SpatialPathVerbV2, SpatialPlacementV2,
    SpatialRgba8V2, SpatialShapeGeometryV2, SpatialShapeKeyV2, SpatialShapeV2, SpatialViewportV2,
    preflight_spatial_direct_counts_v2, resolve_spatial_v2,
};

use super::input::{Parts, container, fill, free, identity, point, root, valid_shape};
use super::types::RawLimitBoundaryV2;

pub(super) fn raw_limits() -> Vec<RawLimitBoundaryV2> {
    SpatialLimitKindV2::ALL
        .into_iter()
        .map(|kind| {
            if SpatialLimitKindV2::DIRECT_ALL.contains(&kind) {
                direct_boundary(kind)
            } else {
                derived_boundary(kind)
            }
        })
        .collect()
}

fn direct_boundary(kind: SpatialLimitKindV2) -> RawLimitBoundaryV2 {
    let index = SpatialLimitKindV2::DIRECT_ALL
        .iter()
        .position(|value| *value == kind)
        .expect("direct kind has an ordinal");
    let maximum = 2_usize;
    let mut values = [usize::MAX; 30];
    values[index] = maximum;
    let limits = SpatialLimitsV2::new(values);
    let mut equal = [0_u128; 12];
    equal[index] = maximum as u128;
    preflight_spatial_direct_counts_v2(equal, limits).expect("direct equality is admitted");
    let mut over = equal;
    over[index] += 1;
    let error = preflight_spatial_direct_counts_v2(over, limits)
        .expect_err("direct one-over must be rejected");
    boundary(kind, error)
}

fn derived_boundary(kind: SpatialLimitKindV2) -> RawLimitBoundaryV2 {
    let maximum = derived_observed(kind);
    let equal = derived_parts(kind);
    resolve_spatial_v2(
        &ReferenceStackEngineV1::new(),
        equal.owned(SpatialViewportV2::new(8, 8)),
        limits(kind, maximum),
    )
    .expect("derived equality is admitted");
    let over = derived_parts(kind);
    let error = resolve_spatial_v2(
        &ReferenceStackEngineV1::new(),
        over.owned(SpatialViewportV2::new(8, 8)),
        limits(kind, maximum - 1),
    )
    .err()
    .expect("derived one-over must be rejected");
    boundary(kind, error)
}

fn boundary(
    kind: SpatialLimitKindV2,
    error: fenestra_ui_spatial::prototype::SpatialResolveErrorV2,
) -> RawLimitBoundaryV2 {
    let observed = error.observed().expect("limit error has observed evidence");
    let maximum = error.maximum().expect("limit error has maximum evidence");
    assert_eq!(
        error.kind(),
        fenestra_ui_spatial::prototype::SpatialResolveErrorKindV2::LimitExceeded(kind)
    );
    assert_eq!(observed, maximum + 1);
    RawLimitBoundaryV2 {
        kind,
        equality_passes: true,
        one_over_kind: error.kind(),
        location: fenestra_ui_spatial::prototype::SpatialErrorLocationV2::Input,
        observed,
        maximum,
    }
}

fn limits(kind: SpatialLimitKindV2, maximum: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; 30];
    let index = SpatialLimitKindV2::ALL
        .iter()
        .position(|value| *value == kind)
        .expect("registered kind has an ordinal");
    values[index] = maximum;
    SpatialLimitsV2::new(values)
}

fn derived_observed(kind: SpatialLimitKindV2) -> usize {
    match kind {
        SpatialLimitKindV2::Depth
        | SpatialLimitKindV2::ChildrenPerNode
        | SpatialLimitKindV2::Islands
        | SpatialLimitKindV2::PathSubpathsTotal
        | SpatialLimitKindV2::ImageEdge
        | SpatialLimitKindV2::ImagePixelsTotal
        | SpatialLimitKindV2::ClipDepth
        | SpatialLimitKindV2::PaintItemsPerNode
        | SpatialLimitKindV2::HitItemsPerNode
        | SpatialLimitKindV2::FlattenedSegmentsPerPath
        | SpatialLimitKindV2::FlattenedSegmentsTotal
        | SpatialLimitKindV2::DependencyVertices
        | SpatialLimitKindV2::DependencyEdges => 1,
        SpatialLimitKindV2::LayoutInputRecordsPerIsland
        | SpatialLimitKindV2::LayoutInputRecordsTotal
        | SpatialLimitKindV2::PathVerbsPerPath
        | SpatialLimitKindV2::GradientStopsPerBrush => 2,
        SpatialLimitKindV2::PolygonPointsPerShape => 3,
        _ => unreachable!("direct kind is handled separately"),
    }
}

fn derived_parts(kind: SpatialLimitKindV2) -> Parts {
    match kind {
        SpatialLimitKindV2::Depth => root_only(),
        SpatialLimitKindV2::ChildrenPerNode => one_free(),
        SpatialLimitKindV2::Islands
        | SpatialLimitKindV2::LayoutInputRecordsPerIsland
        | SpatialLimitKindV2::LayoutInputRecordsTotal => one_layout(),
        SpatialLimitKindV2::PathVerbsPerPath
        | SpatialLimitKindV2::PathSubpathsTotal
        | SpatialLimitKindV2::FlattenedSegmentsPerPath
        | SpatialLimitKindV2::FlattenedSegmentsTotal => one_path(),
        SpatialLimitKindV2::PolygonPointsPerShape => one_polygon(),
        SpatialLimitKindV2::GradientStopsPerBrush => one_gradient(),
        SpatialLimitKindV2::ImageEdge | SpatialLimitKindV2::ImagePixelsTotal => one_image(),
        SpatialLimitKindV2::ClipDepth => one_clip(),
        SpatialLimitKindV2::PaintItemsPerNode => one_paint(),
        SpatialLimitKindV2::HitItemsPerNode => one_hit(),
        SpatialLimitKindV2::DependencyVertices => one_free(),
        SpatialLimitKindV2::DependencyEdges => one_dependency_edge(),
        _ => unreachable!("direct kind is handled separately"),
    }
}

fn root_only() -> Parts {
    Parts {
        nodes: vec![root()],
        ..Parts::default()
    }
}

fn one_free() -> Parts {
    Parts {
        nodes: vec![
            root(),
            free(1, 0, SpatialAnchorTargetV2::Viewport, identity()),
        ],
        ..Parts::default()
    }
}

fn one_layout() -> Parts {
    let layout = SpatialNodeV2::new(
        SpatialNodeKeyV2::new(1),
        Some(SpatialNodeKeyV2::new(0)),
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            LayoutDimensionV1::new(4, 4, 4),
            LayoutDimensionV1::new(4, 4, 4),
            identity(),
        )),
        container(),
    );
    Parts {
        nodes: vec![root(), layout],
        ..Parts::default()
    }
}

fn one_path() -> Parts {
    let mut parts = root_only();
    parts.verbs = vec![
        SpatialPathVerbV2::MoveTo { to: point(0, 0) },
        SpatialPathVerbV2::LineTo {
            to: point(65_536, 0),
        },
    ];
    parts.paths = vec![SpatialPathV2::new(SpatialPathKeyV2::new(0), 0, 2)];
    parts
}

fn one_polygon() -> Parts {
    let mut parts = one_free();
    parts.points = vec![point(0, 0), point(65_536, 0), point(0, 65_536)];
    parts.shapes = vec![SpatialShapeV2::new(
        SpatialShapeKeyV2::new(0),
        SpatialNodeKeyV2::new(1),
        SpatialShapeGeometryV2::Polygon {
            point_start: 0,
            point_length: 3,
        },
    )];
    parts
}

fn one_gradient() -> Parts {
    let mut parts = root_only();
    parts.stops = vec![
        SpatialGradientStopV2::new(0, SpatialRgba8V2::new(0, 0, 0, 255)),
        SpatialGradientStopV2::new(u16::MAX, SpatialRgba8V2::new(255, 255, 255, 255)),
    ];
    parts.brushes = vec![SpatialBrushV2::new(
        SpatialBrushKeyV2::new(0),
        SpatialBrushContentV2::LinearGradient {
            stop_start: 0,
            stop_length: 2,
            start: point(0, 0),
            end: point(65_536, 0),
        },
    )];
    parts
}

fn one_image() -> Parts {
    let mut parts = root_only();
    parts.images = vec![SpatialImageV2::new(
        SpatialImageKeyV2::new(0),
        1,
        1,
        4,
        vec![0; 4].into_boxed_slice(),
    )];
    parts
}

fn one_clip() -> Parts {
    let mut parts = one_free();
    parts.shapes.push(valid_shape());
    parts.clips.push(SpatialClipV2::new(
        SpatialClipKeyV2::new(0),
        SpatialNodeKeyV2::new(1),
        None,
        SpatialShapeKeyV2::new(0),
        SpatialFillRuleV2::NonZero,
    ));
    parts
}

fn one_paint() -> Parts {
    let mut parts = one_free();
    parts.shapes.push(valid_shape());
    parts.brushes.push(SpatialBrushV2::new(
        SpatialBrushKeyV2::new(0),
        SpatialBrushContentV2::Solid {
            color: SpatialRgba8V2::new(0, 0, 0, 255),
        },
    ));
    parts.paints.push(SpatialPaintV2::new(
        SpatialNodeKeyV2::new(1),
        0,
        SpatialPaintContentV2::CoveragePaint {
            coverage: fill(0),
            brush: SpatialBrushKeyV2::new(0),
            opacity: 255,
            clip: None,
        },
    ));
    parts
}

fn one_hit() -> Parts {
    let mut parts = one_free();
    parts.shapes.push(valid_shape());
    parts.hits.push(SpatialHitV2::new(
        SpatialNodeKeyV2::new(1),
        0,
        fill(0),
        None,
        SpatialInputPolicyV2::Accept,
    ));
    parts
}

fn one_dependency_edge() -> Parts {
    Parts {
        nodes: vec![
            root(),
            free(
                1,
                0,
                SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(2)),
                identity(),
            ),
            free(2, 0, SpatialAnchorTargetV2::Viewport, identity()),
        ],
        ..Parts::default()
    }
}
