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
fn one_shot_short_circuits_the_first_direct_limit_without_layout_or_owner_leak() {
    let source = direct_limit_owned();
    let weak = Arc::downgrade(&source);
    let engine = successful_layout_engine();

    let error = resolution_error(resolve_spatial_v2(&engine, source, direct_limit_limits()));

    assert_eq!(
        error.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::Nodes)
    );
    assert_eq!(error.location(), SpatialErrorLocationV2::Input);
    assert_eq!(error.observed(), Some(1));
    assert_eq!(error.maximum(), Some(0));
    assert_eq!(engine.call_count(), 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn one_shot_preserves_the_layout_error_and_stops_before_materialization() {
    let source = layout_failure_owned();
    let weak = Arc::downgrade(&source);
    let engine = rejected_layout_engine();

    let error = resolution_error(resolve_spatial_v2(&engine, source, requested_limits()));

    expect_non_limit(
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
fn one_shot_preserves_late_projection_failure_after_one_successful_layout_call() {
    let source = late_failure_owned();
    let weak = Arc::downgrade(&source);
    let engine = successful_layout_engine();

    let error = resolution_error(resolve_spatial_v2(&engine, source, requested_limits()));

    expect_non_limit(
        error,
        SpatialResolveErrorKindV2::Arithmetic(SpatialArithmeticOperationV2::AabbMaxX),
        SpatialErrorLocationV2::Node { index: 2 },
    );
    assert_eq!(engine.call_count(), 1);
    assert!(weak.upgrade().is_none());
}

fn expect_non_limit(
    error: SpatialResolveErrorV2,
    kind: SpatialResolveErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    assert_eq!(error.kind(), kind);
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
}

fn resolution_error(
    result: Result<SpatialResolvedSnapshotV2, SpatialResolveErrorV2>,
) -> SpatialResolveErrorV2 {
    match result {
        Ok(_) => panic!("expected one-shot resolution failure"),
        Err(error) => error,
    }
}
