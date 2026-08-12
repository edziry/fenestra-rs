use std::sync::Arc;

use super::super::fixture::RawInputFixture;
use super::super::world_aabb_support::owner_node;
use super::super::world_transform_support::{identity, root};
pub(super) use super::ReferenceRasterRedBridge;
use super::support::{requested_limits, zero_call_engine};
use super::validator_support::CandidateTables;
use super::*;
use crate::brush::{SpatialBrushContentV2, SpatialBrushV2, SpatialGradientStopV2, SpatialRgba8V2};
use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
use crate::coverage::{SpatialClipV2, SpatialCoverageV2, SpatialFillRuleV2};
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2, SpatialImageV2};
use crate::model::{SpatialNodeKeyV2, SpatialPointV2, SpatialScalarV2, SpatialViewportV2};
use crate::owned_input::SpatialOwnedInputV2;
use crate::paint::{SpatialPaintContentV2, SpatialPaintV2};
use crate::path::{SpatialPathV2, SpatialPathVerbV2};
use crate::shape::SpatialShapeV2;
use crate::topology::SpatialNodeV2;

pub(super) const S: i64 = SpatialScalarV2::SCALE;

#[allow(clippy::too_many_arguments)]
pub(super) fn owned_fixture(
    viewport: SpatialViewportV2,
    nodes: Vec<SpatialNodeV2>,
    shapes: Vec<SpatialShapeV2>,
    polygon_points: Vec<SpatialPointV2>,
    paths: Vec<SpatialPathV2>,
    path_verbs: Vec<SpatialPathVerbV2>,
    brushes: Vec<SpatialBrushV2>,
    gradient_stops: Vec<SpatialGradientStopV2>,
    images: Vec<SpatialImageV2>,
    clips: Vec<SpatialClipV2>,
    paints: Vec<SpatialPaintV2>,
) -> Arc<SpatialOwnedInputV2> {
    Arc::new(
        RawInputFixture::with_nodes(nodes)
            .with_paths(paths, path_verbs)
            .with_shapes(shapes, polygon_points)
            .with_brushes(brushes, gradient_stops)
            .with_images(images)
            .with_clips(clips)
            .with_paint_items(paints)
            .with_hit_items(Vec::new())
            .with_semantic_items(Vec::new())
            .into_owned(viewport),
    )
}

pub(super) fn empty_owned(viewport: SpatialViewportV2) -> Arc<SpatialOwnedInputV2> {
    owned_fixture(
        viewport,
        vec![root()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

pub(super) fn root_and_owners(count: u32, width: i32, height: i32) -> Vec<SpatialNodeV2> {
    let mut nodes = vec![root()];
    nodes.extend((1..=count).map(|key| owner_node(key, identity(), width, height)));
    nodes
}

pub(super) fn snapshot(source: Arc<SpatialOwnedInputV2>) -> SpatialResolvedSnapshotV2 {
    let prepared = prepare_spatial_v2(&zero_call_engine(), source, requested_limits())
        .expect("raster fixture prepares");
    materialize_reference_spatial_v2(prepared)
}

pub(super) fn candidate_case(
    source: Arc<SpatialOwnedInputV2>,
) -> (PreparedSpatialV2, CandidateTables) {
    let prepared = prepare_spatial_v2(&zero_call_engine(), source.clone(), requested_limits())
        .expect("raster fixture prepares");
    let reference = snapshot(source);
    (prepared, CandidateTables::from_snapshot(&reference))
}

pub(super) const fn limits(pixels: usize) -> ReferenceRasterLimitsV2 {
    ReferenceRasterLimitsV2::new(pixels)
}

pub(super) const fn viewport(width: i32, height: i32) -> SpatialViewportV2 {
    SpatialViewportV2::new(width, height)
}

pub(super) const fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(SpatialScalarV2::new(x), SpatialScalarV2::new(y))
}

pub(super) const fn color(r: u8, g: u8, b: u8, a: u8) -> SpatialRgba8V2 {
    SpatialRgba8V2::new(r, g, b, a)
}

pub(super) const fn solid(key: u32, value: SpatialRgba8V2) -> SpatialBrushV2 {
    SpatialBrushV2::new(
        SpatialBrushKeyV2::new(key),
        SpatialBrushContentV2::Solid { color: value },
    )
}

pub(super) const fn gradient(
    key: u32,
    start: u32,
    length: u32,
    from: SpatialPointV2,
    to: SpatialPointV2,
) -> SpatialBrushV2 {
    SpatialBrushV2::new(
        SpatialBrushKeyV2::new(key),
        SpatialBrushContentV2::LinearGradient {
            stop_start: start,
            stop_length: length,
            start: from,
            end: to,
        },
    )
}

pub(super) const fn stop(offset: u16, value: SpatialRgba8V2) -> SpatialGradientStopV2 {
    SpatialGradientStopV2::new(offset, value)
}

pub(super) fn image(key: u32, width: u32, height: u32, bytes: Vec<u8>) -> SpatialImageV2 {
    SpatialImageV2::new(
        SpatialImageKeyV2::new(key),
        width,
        height,
        width.checked_mul(4).expect("test stride"),
        bytes.into_boxed_slice(),
    )
}

pub(super) const fn source(x: u32, y: u32, width: u32, height: u32) -> SpatialImageSourceRectV2 {
    SpatialImageSourceRectV2::new(x, y, width, height)
}

pub(super) const fn destination(
    x: i64,
    y: i64,
    width: i64,
    height: i64,
) -> SpatialImageDestinationRectV2 {
    SpatialImageDestinationRectV2::new(
        SpatialScalarV2::new(x),
        SpatialScalarV2::new(y),
        SpatialScalarV2::new(width),
        SpatialScalarV2::new(height),
    )
}

pub(super) const fn fill(
    owner: u32,
    ordinal: u32,
    shape: u32,
    brush: u32,
    opacity: u8,
    clip: Option<u32>,
    rule: SpatialFillRuleV2,
) -> SpatialPaintV2 {
    coverage(
        owner,
        ordinal,
        SpatialCoverageV2::Fill {
            shape: crate::geometry_key::SpatialShapeKeyV2::new(shape),
            rule,
        },
        brush,
        opacity,
        clip,
    )
}

pub(super) const fn stroke(
    owner: u32,
    ordinal: u32,
    shape: u32,
    width: i64,
    brush: u32,
    opacity: u8,
    clip: Option<u32>,
) -> SpatialPaintV2 {
    coverage(
        owner,
        ordinal,
        SpatialCoverageV2::RoundStroke {
            shape: crate::geometry_key::SpatialShapeKeyV2::new(shape),
            width: SpatialScalarV2::new(width),
        },
        brush,
        opacity,
        clip,
    )
}

const fn coverage(
    owner: u32,
    ordinal: u32,
    coverage: SpatialCoverageV2,
    brush: u32,
    opacity: u8,
    clip: Option<u32>,
) -> SpatialPaintV2 {
    SpatialPaintV2::new(
        SpatialNodeKeyV2::new(owner),
        ordinal,
        SpatialPaintContentV2::CoveragePaint {
            coverage,
            brush: SpatialBrushKeyV2::new(brush),
            opacity,
            clip: match clip {
                Some(key) => Some(crate::geometry_key::SpatialClipKeyV2::new(key)),
                None => None,
            },
        },
    )
}

pub(super) const fn image_paint(
    owner: u32,
    ordinal: u32,
    image: u32,
    source: SpatialImageSourceRectV2,
    destination: SpatialImageDestinationRectV2,
    opacity: u8,
    clip: Option<u32>,
) -> SpatialPaintV2 {
    SpatialPaintV2::new(
        SpatialNodeKeyV2::new(owner),
        ordinal,
        SpatialPaintContentV2::ImagePaint {
            image: SpatialImageKeyV2::new(image),
            source,
            destination,
            opacity,
            clip: match clip {
                Some(key) => Some(crate::geometry_key::SpatialClipKeyV2::new(key)),
                None => None,
            },
        },
    )
}

pub(super) fn assert_raster(raster: &ReferenceRasterV2, width: u32, height: u32, expected: &[u8]) {
    assert_eq!(raster.width(), width);
    assert_eq!(raster.height(), height);
    assert_eq!(raster.stride(), u64::from(width) * 4);
    assert_eq!(raster.bytes(), expected);
}
