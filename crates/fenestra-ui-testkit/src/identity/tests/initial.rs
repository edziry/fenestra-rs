use super::*;

#[test]
fn clean_initial_snapshot_records_one_alias_free_snapshot() {
    let (_fixture, _desired, _snapshot, identities) = initial_state();
    let mut ledger = IdentityLedgerV1::new();

    assert_eq!(ledger.summary(), IdentitySummaryV1::default());
    assert!(
        ledger
            .verify_initial_aliases(&identities)
            .expect("clean initial identities should verify")
            .is_none()
    );
    assert_eq!(ledger.summary().alias_free_snapshots(), 1);
    assert_eq!(ledger.summary().preserved(), 0);
    assert_eq!(ledger.summary().retired(), 0);
    assert_eq!(ledger.summary().fresh(), 0);
}

#[test]
fn initial_node_alias_is_atomic_and_a_clean_retry_succeeds() {
    let (_fixture, _desired, _snapshot, clean) = initial_state();
    let mut authored = clean.nodes_in_authored_order();
    let (_, source) = authored.next().expect("source node should exist");
    let (target, _) = authored.next().expect("target node should exist");
    let target = target.clone();
    let aliased = replace_node(&clean, &target, source);
    let mut ledger = IdentityLedgerV1::new();

    expect_mismatch(
        ledger.verify_initial_aliases(&aliased),
        FingerprintLocationV1::Node(target),
        FingerprintSummaryV1::LifecycleDistinct,
        FingerprintSummaryV1::LifecycleAliased,
    );
    assert_eq!(ledger.summary(), IdentitySummaryV1::default());

    assert!(
        ledger
            .verify_initial_aliases(&clean)
            .expect("clean retry should verify")
            .is_none()
    );
    assert_eq!(ledger.summary().alias_free_snapshots(), 1);
}

#[test]
fn initial_fragment_alias_is_atomic_and_a_clean_retry_succeeds() {
    let (_fixture, _desired, _snapshot, clean) = initial_state();
    let mut authored = clean.fragments_in_authored_order();
    let (_, source) = authored.next().expect("source fragment should exist");
    let (target, _) = authored.next().expect("target fragment should exist");
    let target = target.clone();
    let aliased = replace_fragment(&clean, &target, source);
    let mut ledger = IdentityLedgerV1::new();

    expect_mismatch(
        ledger.verify_initial_aliases(&aliased),
        FingerprintLocationV1::Fragment(target),
        FingerprintSummaryV1::LifecycleDistinct,
        FingerprintSummaryV1::LifecycleAliased,
    );
    assert_eq!(ledger.summary(), IdentitySummaryV1::default());

    assert!(
        ledger
            .verify_initial_aliases(&clean)
            .expect("clean retry should verify")
            .is_none()
    );
    assert_eq!(ledger.summary().alias_free_snapshots(), 1);
}
