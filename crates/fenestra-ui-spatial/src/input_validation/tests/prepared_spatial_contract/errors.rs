use std::sync::Arc;

use fenestra_ui_layout::prototype::LayoutEngineErrorKindV1;

use super::support::{
    direct_limit_limits, direct_limit_owned, late_failure_owned, layout_failure_owned,
    rejected_layout_engine, requested_limits, successful_layout_engine,
};
use super::*;
use crate::error::SpatialErrorLocationV2;
use crate::limits::SpatialLimitKindV2;
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::resolve_error::{
    SpatialLayoutErrorKindV2, SpatialResolveErrorKindV2, SpatialResolveErrorV2,
};

#[test]
fn preparation_preserves_the_first_direct_limit_before_malformed_later_tables() {
    let source = direct_limit_owned();
    let weak = Arc::downgrade(&source);
    let engine = successful_layout_engine();

    let error = prepare_error(prepare_spatial_v2(&engine, source, direct_limit_limits()));

    expect_limit(error, SpatialLimitKindV2::Nodes, 1, 0);
    assert_eq!(engine.call_count(), 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn preparation_preserves_layout_failures_and_drops_owned_input() {
    let source = layout_failure_owned();
    let weak = Arc::downgrade(&source);
    let engine = rejected_layout_engine();

    let error = prepare_error(prepare_spatial_v2(&engine, source, requested_limits()));

    expect_error(
        error,
        SpatialResolveErrorKindV2::Layout(SpatialLayoutErrorKindV2::Engine(
            LayoutEngineErrorKindV1::RejectedInput,
        )),
        SpatialErrorLocationV2::Node { index: 1 },
    );
    assert_eq!(engine.call_count(), 1);
    assert!(weak.upgrade().is_none());
}

#[test]
fn preparation_runs_layout_then_preserves_late_world_aabb_failures() {
    let source = late_failure_owned();
    let weak = Arc::downgrade(&source);
    let engine = successful_layout_engine();

    let error = prepare_error(prepare_spatial_v2(&engine, source, requested_limits()));

    expect_error(
        error,
        SpatialResolveErrorKindV2::Arithmetic(SpatialArithmeticOperationV2::AabbMaxX),
        SpatialErrorLocationV2::Node { index: 2 },
    );
    assert_eq!(engine.call_count(), 1);
    assert!(weak.upgrade().is_none());
}

fn expect_error(
    error: SpatialResolveErrorV2,
    kind: SpatialResolveErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    assert_eq!(error.kind(), kind);
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
}

fn expect_limit(
    error: SpatialResolveErrorV2,
    kind: SpatialLimitKindV2,
    observed: u128,
    maximum: u128,
) {
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::LimitExceeded(kind));
    assert_eq!(error.location(), SpatialErrorLocationV2::Input);
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
}

fn prepare_error(
    result: Result<PreparedSpatialV2, SpatialResolveErrorV2>,
) -> SpatialResolveErrorV2 {
    match result {
        Ok(_) => panic!("expected preparation failure"),
        Err(error) => error,
    }
}
