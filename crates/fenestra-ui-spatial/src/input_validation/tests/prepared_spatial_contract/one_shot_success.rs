use std::sync::Arc;

use super::ownership::{assert_identities, identities};
use super::snapshot_empty::assert_distinct_root_snapshot;
use super::snapshot_output::assert_rich_snapshot;
use super::snapshot_retention::RetainedFacts;
use super::support::{
    distinct_viewport_root_owned, requested_limits, rich_engine, rich_owned, zero_call_engine,
};
use super::*;

#[test]
fn one_shot_reuses_complete_preparation_and_exact_reference_materialization() {
    let expected_prepared = prepare_spatial_v2(&rich_engine(), rich_owned(), requested_limits())
        .expect("comparison preparation succeeds");
    let expected_facts = RetainedFacts::capture(&expected_prepared);
    drop(expected_prepared);

    let source = rich_owned();
    let weak = Arc::downgrade(&source);
    let source_identities = identities(&source);
    let engine = rich_engine();

    let snapshot = resolve_spatial_v2(&engine, source, requested_limits())
        .expect("one-shot rich resolution succeeds");

    assert_eq!(
        engine.calls(),
        vec![(10, 10, vec![(0, None, 10, 10), (1, Some(0), 3, 4)],)]
    );
    assert_rich_snapshot(&snapshot);
    expected_facts.assert_snapshot(&snapshot);
    let upgraded = weak.upgrade().expect("snapshot retains exact input owner");
    assert!(Arc::ptr_eq(snapshot.source_arc(), &upgraded));
    assert_identities(snapshot.source_arc(), &source_identities);
    let expected_bytes = snapshot.source_arc().as_input().resources().images()[1].bytes();
    assert_eq!(
        identity(snapshot.finalized_image_paint_bytes(1).unwrap()),
        identity(expected_bytes)
    );

    drop(upgraded);
    drop(snapshot);
    assert!(weak.upgrade().is_none());
}

fn identity<T>(slice: &[T]) -> (*const T, usize) {
    (slice.as_ptr(), slice.len())
}

#[test]
fn one_shot_root_only_uses_its_distinct_viewport_without_layout_calls() {
    let engine = zero_call_engine();
    let snapshot = resolve_spatial_v2(&engine, distinct_viewport_root_owned(), requested_limits())
        .expect("one-shot root resolution succeeds");

    assert_eq!(engine.call_count(), 0);
    assert_distinct_root_snapshot(&snapshot);
}
