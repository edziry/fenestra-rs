use std::error::Error;

use super::fixture::RawInputFixture;
use super::local_transform_support::VIEWPORT;
use super::validated_hit_support;
use crate::content_diagnostic::{
    SpatialClipErrorV2, SpatialContentReferenceV2, SpatialOrderedItemTableV2,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::content_item::{SpatialHitV2, SpatialSemanticGeometryV2};
use crate::coverage::SpatialFillRuleV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::item_field::SpatialSemanticFieldV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::SpatialNodeKeyV2;
use crate::paint::SpatialPaintV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) fn fixture(semantics: Vec<SpatialSemanticGeometryV2>) -> RawInputFixture {
    fixture_with_items(Vec::new(), Vec::new(), semantics)
}

pub(super) fn fixture_with_items(
    paints: Vec<SpatialPaintV2>,
    hits: Vec<SpatialHitV2>,
    semantics: Vec<SpatialSemanticGeometryV2>,
) -> RawInputFixture {
    validated_hit_support::fixture_with_paints(paints, hits).with_semantic_items(semantics)
}

pub(super) const fn semantic(
    owner: u32,
    ordinal: u32,
    shape: u32,
    fill_rule: SpatialFillRuleV2,
    clip: Option<u32>,
) -> SpatialSemanticGeometryV2 {
    SpatialSemanticGeometryV2::new(
        SpatialNodeKeyV2::new(owner),
        ordinal,
        SpatialShapeKeyV2::new(shape),
        fill_rule,
        match clip {
            Some(clip) => Some(SpatialClipKeyV2::new(clip)),
            None => None,
        },
    )
}

pub(super) const fn limits() -> SpatialLimitsV2 {
    SpatialLimitsV2::new([usize::MAX; SpatialLimitKindV2::ALL.len()])
}

pub(super) fn no_item_limits(semantic_total: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; SpatialLimitKindV2::ALL.len()];
    for (value, kind) in values.iter_mut().zip(SpatialLimitKindV2::ALL) {
        match kind {
            SpatialLimitKindV2::SemanticItems => *value = semantic_total,
            SpatialLimitKindV2::PaintItemsPerNode | SpatialLimitKindV2::HitItemsPerNode => {
                *value = 0;
            }
            _ => {}
        }
    }
    SpatialLimitsV2::new(values)
}

pub(super) fn validate(
    fixture: &RawInputFixture,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    prepare_validated_semantic_items!(fixture, VIEWPORT, limits).map(|_| ())
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected validated-semantic success, got {error:?}"),
    }
}

pub(super) fn expect_reference<T>(
    result: Result<T, SpatialResolveErrorV2>,
    reference: SpatialContentReferenceV2,
    index: u32,
    field: SpatialSemanticFieldV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidReference(reference),
        semantic_location(index, field),
    );
}

pub(super) fn expect_order<T>(
    result: Result<T, SpatialResolveErrorV2>,
    index: u32,
    field: SpatialSemanticFieldV2,
) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidOrder(SpatialOrderedItemTableV2::Semantic),
        semantic_location(index, field),
    );
}

pub(super) fn expect_clip<T>(result: Result<T, SpatialResolveErrorV2>, index: u32) {
    expect_content(
        result,
        SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ItemOwnerNotDescendant),
        semantic_location(index, SpatialSemanticFieldV2::Clip),
    );
}

fn expect_content<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected validated-semantic content failure"),
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

pub(super) const fn semantic_location(
    index: u32,
    field: SpatialSemanticFieldV2,
) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Semantic { index, field }
}
