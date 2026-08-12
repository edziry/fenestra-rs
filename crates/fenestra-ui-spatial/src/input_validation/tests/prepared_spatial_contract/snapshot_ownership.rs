use std::sync::Arc;

use super::ownership::{assert_identities, identities};
use super::support::{requested_limits, rich_engine, rich_owned};
use super::*;

#[test]
fn materialization_transfers_the_exact_source_and_effective_clip_allocation() {
    let source = rich_owned();
    let weak = Arc::downgrade(&source);
    let source_identities = identities(&source);
    let prepared = prepare_spatial_v2(&rich_engine(), source, requested_limits())
        .expect("rich owned input prepares successfully");
    let effective = prepared.effective_clip_identity();

    let snapshot = materialize_reference_spatial_v2(prepared);
    let upgraded = weak.upgrade().expect("snapshot retains the source owner");
    assert!(Arc::ptr_eq(snapshot.source_arc(), &upgraded));
    assert_identities(snapshot.source_arc(), &source_identities);
    assert_eq!(snapshot.effective_clip_identity(), effective);
    assert_eq!(
        snapshot.effective_clip_identity(),
        (
            snapshot.effective_clip_aabbs().as_ptr(),
            snapshot.effective_clip_aabbs().len(),
        )
    );

    drop(upgraded);
    drop(snapshot);
    assert!(weak.upgrade().is_none());
}
