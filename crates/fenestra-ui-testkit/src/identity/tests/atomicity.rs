use super::*;

#[test]
fn failed_transition_is_atomic_and_the_next_valid_transition_works() {
    let transition = removal_transition();
    let (_foreign_fixture, _foreign_desired, _foreign_snapshot, foreign) = initial_state();
    let target = transition
        .after
        .fragments_in_authored_order()
        .last()
        .map(|(path, _)| path.clone())
        .expect("a surviving fragment should exist");
    let replacement = foreign
        .fragment(&target)
        .expect("matching foreign fragment should exist");
    let faulty_after = replace_fragment(&transition.after, &target, replacement);
    let mut ledger = IdentityLedgerV1::new();
    let summary_before = ledger.summary();
    let retired_nodes_before = ledger.retired_nodes.clone();
    let retired_fragments_before = ledger.retired_fragments.clone();

    expect_mismatch(
        ledger.verify_transition(
            &transition.before,
            &transition.before_desired,
            &faulty_after,
            &transition.after_desired,
            &transition.after_snapshot,
        ),
        FingerprintLocationV1::Fragment(target),
        FingerprintSummaryV1::LifecyclePreserved,
        FingerprintSummaryV1::LifecycleFresh,
    );

    assert_eq!(ledger.summary(), summary_before);
    assert_eq!(ledger.retired_nodes, retired_nodes_before);
    assert_eq!(ledger.retired_fragments, retired_fragments_before);

    let valid = ledger
        .verify_transition(
            &transition.before,
            &transition.before_desired,
            &transition.after,
            &transition.after_desired,
            &transition.after_snapshot,
        )
        .expect("valid transition should remain operational");
    assert!(valid.is_none());
    assert_ne!(ledger.summary(), IdentitySummaryV1::default());
    assert!(ledger.retired_nodes.contains_key(&retired_member()));
    assert!(
        ledger
            .retired_fragments
            .contains_key(&retired_nested_fragment())
    );
}
