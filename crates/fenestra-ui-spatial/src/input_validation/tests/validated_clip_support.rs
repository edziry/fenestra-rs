use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::{VIEWPORT, free_node, identity, input, root};
use super::validated_shape_support::rect;
use crate::content_diagnostic::{
    SpatialClipErrorV2, SpatialContentReferenceV2, SpatialKeyedContentTableV2,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::coverage::{SpatialClipV2, SpatialFillRuleV2};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::item_field::SpatialClipFieldV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::SpatialNodeKeyV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::shape::SpatialShapeV2;
use crate::topology::SpatialNodeV2;

pub(super) fn standard_fixture(clips: Vec<SpatialClipV2>) -> RawInputFixture {
    let transform = identity();
    fixture_with(
        vec![
            root(),
            free_node(1, 0, 10, 10, transform),
            free_node(2, 0, 10, 10, transform),
        ],
        vec![rect(0, 1), rect(1, 2)],
        clips,
    )
}

pub(super) fn fixture_with(
    nodes: Vec<SpatialNodeV2>,
    shapes: Vec<SpatialShapeV2>,
    clips: Vec<SpatialClipV2>,
) -> RawInputFixture {
    input(nodes)
        .with_paths(Vec::new(), Vec::new())
        .with_shapes(shapes, Vec::new())
        .with_brushes(Vec::new(), Vec::new())
        .with_images(Vec::new())
        .with_clips(clips)
}

pub(super) const fn clip(
    key: u32,
    owner: u32,
    parent: Option<u32>,
    shape: u32,
    fill_rule: SpatialFillRuleV2,
) -> SpatialClipV2 {
    SpatialClipV2::new(
        SpatialClipKeyV2::new(key),
        SpatialNodeKeyV2::new(owner),
        match parent {
            Some(parent) => Some(SpatialClipKeyV2::new(parent)),
            None => None,
        },
        SpatialShapeKeyV2::new(shape),
        fill_rule,
    )
}

pub(super) const fn root_clip(key: u32, owner: u32, shape: u32) -> SpatialClipV2 {
    clip(key, owner, None, shape, SpatialFillRuleV2::NonZero)
}

pub(super) fn limits(maximum_depth: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        if kind == SpatialLimitKindV2::ClipDepth {
            *value = maximum_depth;
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn deferred_limits(maximum_depth: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::ClipDepth => *value = maximum_depth,
            SpatialLimitKindV2::PaintItemsPerNode
            | SpatialLimitKindV2::HitItemsPerNode
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
    prepare_validated_clips!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected validated-clip success, got {error:?}"),
    }
}

pub(super) fn expect_non_dense<T>(
    result: Result<T, SpatialResolveErrorV2>,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Clip),
        location,
    );
}

pub(super) fn expect_reference<T>(
    result: Result<T, SpatialResolveErrorV2>,
    reference: SpatialContentReferenceV2,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidReference(reference),
        location,
    );
}

pub(super) fn expect_clip<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialClipErrorV2,
    location: SpatialErrorLocationV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidClip(kind),
        location,
    );
}

fn expect_content<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected validated-clip content failure"),
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

pub(super) fn expect_depth<T>(
    result: Result<T, SpatialResolveErrorV2>,
    clip: u32,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected clip-depth failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::ClipDepth)
    );
    assert_eq!(
        error.location(),
        clip_location(clip, SpatialClipFieldV2::Parent)
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

pub(super) const fn clip_location(index: u32, field: SpatialClipFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Clip { index, field }
}
