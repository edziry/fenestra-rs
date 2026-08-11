use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::VIEWPORT;
use super::validated_paint_support;
use crate::content_diagnostic::{
    SpatialClipErrorV2, SpatialContentReferenceV2, SpatialOrderedItemTableV2, SpatialStrokeErrorV2,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::content_item::{SpatialHitV2, SpatialInputPolicyV2};
use crate::coverage::{SpatialCoverageV2, SpatialFillRuleV2};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::item_field::SpatialHitFieldV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{SpatialNodeKeyV2, SpatialScalarV2};
use crate::paint::SpatialPaintV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn fixture(hits: Vec<SpatialHitV2>) -> RawInputFixture {
    fixture_with_paints(Vec::new(), hits)
}

pub(super) fn fixture_with_paints(
    paints: Vec<SpatialPaintV2>,
    hits: Vec<SpatialHitV2>,
) -> RawInputFixture {
    validated_paint_support::fixture(paints).with_hit_items(hits)
}

pub(super) fn fill(
    owner: u32,
    ordinal: u32,
    shape: u32,
    clip: Option<u32>,
    rule: SpatialFillRuleV2,
    policy: SpatialInputPolicyV2,
) -> SpatialHitV2 {
    hit(
        owner,
        ordinal,
        SpatialCoverageV2::Fill {
            shape: SpatialShapeKeyV2::new(shape),
            rule,
        },
        clip,
        policy,
    )
}

pub(super) fn stroke(
    owner: u32,
    ordinal: u32,
    shape: u32,
    width: i64,
    clip: Option<u32>,
    policy: SpatialInputPolicyV2,
) -> SpatialHitV2 {
    hit(
        owner,
        ordinal,
        SpatialCoverageV2::RoundStroke {
            shape: SpatialShapeKeyV2::new(shape),
            width: SpatialScalarV2::new(width),
        },
        clip,
        policy,
    )
}

fn hit(
    owner: u32,
    ordinal: u32,
    coverage: SpatialCoverageV2,
    clip: Option<u32>,
    policy: SpatialInputPolicyV2,
) -> SpatialHitV2 {
    SpatialHitV2::new(
        SpatialNodeKeyV2::new(owner),
        ordinal,
        coverage,
        clip.map(SpatialClipKeyV2::new),
        policy,
    )
}

pub(super) fn limits(maximum: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        if kind == SpatialLimitKindV2::HitItemsPerNode {
            *value = maximum;
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn deferred_limits(maximum: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::HitItemsPerNode => *value = maximum,
            SpatialLimitKindV2::FlattenedSegmentsPerPath
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
    prepare_validated_hit_items!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected validated-hit success, got {error:?}"),
    }
}

pub(super) fn expect_content<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected validated-hit content failure"),
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
    field: SpatialHitFieldV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidReference(reference),
        hit_location(index, field),
    );
}

pub(super) fn expect_order<T>(
    result: Result<T, SpatialResolveErrorV2>,
    index: u32,
    field: SpatialHitFieldV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidOrder(SpatialOrderedItemTableV2::Hit),
        hit_location(index, field),
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
        hit_location(index, SpatialHitFieldV2::StrokeWidth),
    );
}

pub(super) fn expect_clip<T>(result: Result<T, SpatialResolveErrorV2>, index: u32) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ItemOwnerNotDescendant),
        hit_location(index, SpatialHitFieldV2::Clip),
    );
}

pub(super) fn expect_limit<T>(
    result: Result<T, SpatialResolveErrorV2>,
    index: u32,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected hit-item limit failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::HitItemsPerNode)
    );
    assert_eq!(
        error.location(),
        hit_location(index, SpatialHitFieldV2::ItemOrdinal)
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

pub(super) const fn hit_location(index: u32, field: SpatialHitFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Hit { index, field }
}
