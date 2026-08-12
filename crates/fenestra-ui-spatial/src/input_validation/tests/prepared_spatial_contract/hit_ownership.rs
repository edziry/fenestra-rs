use std::sync::Arc;

use super::hit_support::*;
use super::ownership::{assert_identities, identities};
use super::support::{requested_limits, rich_engine, rich_owned};
use super::*;

#[test]
fn repeated_queries_preserve_all_snapshot_and_source_allocations() {
    let source = rich_owned();
    let weak = Arc::downgrade(&source);
    let source_identities = identities(&source);
    let prepared = prepare_spatial_v2(&rich_engine(), source, requested_limits())
        .expect("rich input prepares");
    let snapshot = materialize_reference_spatial_v2(prepared);
    let output = snapshot.output();
    let table_identities = [
        erased(output.geometry()),
        erased(output.clips()),
        erased(output.paints()),
        erased(output.hits()),
        erased(output.semantics()),
    ];
    let effective = erased(snapshot.effective_clip_aabbs());

    for _ in 0..3 {
        assert_hit(
            snapshot.hit_test(point(2 * S, 15 * S)),
            0,
            2,
            0,
            point(S, 3 * S),
        );
        let output = snapshot.output();
        assert_eq!(
            [
                erased(output.geometry()),
                erased(output.clips()),
                erased(output.paints()),
                erased(output.hits()),
                erased(output.semantics()),
            ],
            table_identities
        );
        assert_eq!(erased(snapshot.effective_clip_aabbs()), effective);
        assert_identities(snapshot.source_arc(), &source_identities);
    }

    let upgraded = weak
        .upgrade()
        .expect("queries retain the exact source owner");
    assert!(Arc::ptr_eq(snapshot.source_arc(), &upgraded));
    drop(upgraded);
    drop(snapshot);
    assert!(weak.upgrade().is_none());
}

fn erased<T>(slice: &[T]) -> (*const (), usize) {
    (slice.as_ptr().cast(), slice.len())
}
