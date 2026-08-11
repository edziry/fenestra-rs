use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::{
    VIEWPORT, fixed, free_node, identity, input, layout_node, root,
};
use super::prepared_brush_support::{color, solid_color};
use super::validated_clip_support::{clip, root_clip};
use super::validated_image_support::{blank_image, image};
use super::validated_shape_support::rect;
use crate::content_diagnostic::{
    SpatialClipErrorV2, SpatialContentReferenceV2, SpatialImageErrorV2, SpatialOrderedItemTableV2,
    SpatialStrokeErrorV2,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
use crate::coverage::{SpatialCoverageV2, SpatialFillRuleV2};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::item_field::SpatialPaintFieldV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{SpatialNodeKeyV2, SpatialScalarV2};
use crate::paint::{SpatialPaintContentV2, SpatialPaintV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn fixture(paints: Vec<SpatialPaintV2>) -> RawInputFixture {
    let transform = identity();
    input(vec![
        root(),
        free_node(1, 0, 10, 10, transform),
        layout_node(2, 1, fixed(10), fixed(10), transform),
        free_node(3, 2, 10, 10, transform),
        free_node(4, 0, 10, 10, transform),
    ])
    .with_paths(Vec::new(), Vec::new())
    .with_shapes(
        vec![rect(0, 1), rect(1, 2), rect(2, 3), rect(3, 4)],
        Vec::new(),
    )
    .with_brushes(
        vec![
            solid_color(0, color(10, 20, 30, 255)),
            solid_color(1, color(40, 50, 60, 255)),
        ],
        Vec::new(),
    )
    .with_images(vec![
        blank_image(0, 4, 4),
        image(1, 2, 2, 8, second_image_bytes()),
    ])
    .with_clips(vec![
        root_clip(0, 1, 0),
        clip(1, 2, Some(0), 1, SpatialFillRuleV2::EvenOdd),
        clip(2, 3, Some(1), 2, SpatialFillRuleV2::NonZero),
        root_clip(3, 4, 3),
    ])
    .with_paint_items(paints)
}

pub(super) fn second_image_bytes() -> Vec<u8> {
    [1, 0, 0, 1].repeat(4)
}

pub(super) fn fill(
    owner: u32,
    ordinal: u32,
    shape: u32,
    brush: u32,
    clip: Option<u32>,
    rule: SpatialFillRuleV2,
) -> SpatialPaintV2 {
    coverage(
        owner,
        ordinal,
        SpatialCoverageV2::Fill {
            shape: SpatialShapeKeyV2::new(shape),
            rule,
        },
        brush,
        clip,
    )
}

pub(super) fn stroke(
    owner: u32,
    ordinal: u32,
    shape: u32,
    width: i64,
    brush: u32,
    clip: Option<u32>,
) -> SpatialPaintV2 {
    coverage(
        owner,
        ordinal,
        SpatialCoverageV2::RoundStroke {
            shape: SpatialShapeKeyV2::new(shape),
            width: SpatialScalarV2::new(width),
        },
        brush,
        clip,
    )
}

fn coverage(
    owner: u32,
    ordinal: u32,
    coverage: SpatialCoverageV2,
    brush: u32,
    clip: Option<u32>,
) -> SpatialPaintV2 {
    SpatialPaintV2::new(
        SpatialNodeKeyV2::new(owner),
        ordinal,
        SpatialPaintContentV2::CoveragePaint {
            coverage,
            brush: SpatialBrushKeyV2::new(brush),
            opacity: 173,
            clip: clip.map(SpatialClipKeyV2::new),
        },
    )
}

pub(super) fn image_paint(
    owner: u32,
    ordinal: u32,
    image: u32,
    source: SpatialImageSourceRectV2,
    destination: SpatialImageDestinationRectV2,
    clip: Option<u32>,
) -> SpatialPaintV2 {
    SpatialPaintV2::new(
        SpatialNodeKeyV2::new(owner),
        ordinal,
        SpatialPaintContentV2::ImagePaint {
            image: SpatialImageKeyV2::new(image),
            source,
            destination,
            opacity: 211,
            clip: clip.map(SpatialClipKeyV2::new),
        },
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

pub(super) const fn valid_source() -> SpatialImageSourceRectV2 {
    source(0, 0, 1, 1)
}

pub(super) const fn valid_destination() -> SpatialImageDestinationRectV2 {
    destination(0, 0, 1, 1)
}

pub(super) fn limits(maximum: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        if kind == SpatialLimitKindV2::PaintItemsPerNode {
            *value = maximum;
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn deferred_limits(maximum: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::PaintItemsPerNode => *value = maximum,
            SpatialLimitKindV2::HitItemsPerNode
            | SpatialLimitKindV2::FlattenedSegmentsPerPath
            | SpatialLimitKindV2::FlattenedSegmentsTotal
            | SpatialLimitKindV2::DependencyVertices
            | SpatialLimitKindV2::DependencyEdges => *value = 0,
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_validated_paint_items!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected validated-paint success, got {error:?}"),
    }
}

pub(super) fn expect_content<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected validated-paint content failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::Content(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), "spatial-resolve-error(content)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(content))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) fn expect_reference<T>(
    result: Result<T, SpatialResolveErrorV2>,
    reference: SpatialContentReferenceV2,
    index: u32,
    field: SpatialPaintFieldV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidReference(reference),
        paint_location(index, field),
    );
}

pub(super) fn expect_order<T>(
    result: Result<T, SpatialResolveErrorV2>,
    index: u32,
    field: SpatialPaintFieldV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidOrder(SpatialOrderedItemTableV2::Paint),
        paint_location(index, field),
    );
}

pub(super) fn expect_image<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialImageErrorV2,
    index: u32,
    field: SpatialPaintFieldV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidImage(kind),
        paint_location(index, field),
    );
}

pub(super) fn expect_stroke<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialStrokeErrorV2,
    index: u32,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidStroke(kind),
        paint_location(index, SpatialPaintFieldV2::StrokeWidth),
    );
}

pub(super) fn expect_clip<T>(result: Result<T, SpatialResolveErrorV2>, index: u32) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ItemOwnerNotDescendant),
        paint_location(index, SpatialPaintFieldV2::Clip),
    );
}

pub(super) fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    index: u32,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected paint-item limit failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::PaintItemsPerNode)
    );
    assert_eq!(
        error.location(),
        paint_location(index, SpatialPaintFieldV2::ItemOrdinal)
    );
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
    assert_eq!(error.to_string(), "spatial-resolve-error(limit-exceeded)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(limit-exceeded))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) const fn paint_location(
    index: u32,
    field: SpatialPaintFieldV2,
) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Paint { index, field }
}
