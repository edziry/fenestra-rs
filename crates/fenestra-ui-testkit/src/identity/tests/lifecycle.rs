use super::*;

#[test]
fn node_alias_reports_the_second_authored_path() {
    let (_fixture, desired, snapshot, before) = initial_state();
    let mut authored = before.nodes_in_authored_order();
    let (_, first) = authored.next().expect("first node should exist");
    let (second_path, _) = authored.next().expect("second node should exist");
    let second_path = second_path.clone();
    let after = replace_node(&before, &second_path, first);

    expect_mismatch(
        IdentityLedgerV1::new().verify_transition(&before, &desired, &after, &desired, &snapshot),
        FingerprintLocationV1::Node(second_path),
        FingerprintSummaryV1::LifecycleDistinct,
        FingerprintSummaryV1::LifecycleAliased,
    );
}

#[test]
fn fragment_alias_reports_the_second_authored_path() {
    let (_fixture, desired, snapshot, before) = initial_state();
    let mut authored = before.fragments_in_authored_order();
    let (_, first) = authored.next().expect("first fragment should exist");
    let (second_path, _) = authored.next().expect("second fragment should exist");
    let second_path = second_path.clone();
    let after = replace_fragment(&before, &second_path, first);

    expect_mismatch(
        IdentityLedgerV1::new().verify_transition(&before, &desired, &after, &desired, &snapshot),
        FingerprintLocationV1::Fragment(second_path),
        FingerprintSummaryV1::LifecycleDistinct,
        FingerprintSummaryV1::LifecycleAliased,
    );
}

#[test]
fn preserved_node_with_a_new_handle_reports_fresh() {
    let (_fixture, desired, snapshot, before) = initial_state();
    let (_foreign_fixture, _foreign_desired, _foreign_snapshot, foreign) = initial_state();
    let path = NodePathV1::root();
    let replacement = foreign.node(&path).expect("foreign root should exist");
    let after = replace_node(&before, &path, replacement);

    expect_mismatch(
        IdentityLedgerV1::new().verify_transition(&before, &desired, &after, &desired, &snapshot),
        FingerprintLocationV1::Node(path),
        FingerprintSummaryV1::LifecyclePreserved,
        FingerprintSummaryV1::LifecycleFresh,
    );
}

#[test]
fn preserved_fragment_with_a_new_handle_reports_fresh() {
    let (_fixture, desired, snapshot, before) = initial_state();
    let (_foreign_fixture, _foreign_desired, _foreign_snapshot, foreign) = initial_state();
    let path = primary_fragment();
    let replacement = foreign
        .fragment(&path)
        .expect("foreign primary fragment should exist");
    let after = replace_fragment(&before, &path, replacement);

    expect_mismatch(
        IdentityLedgerV1::new().verify_transition(&before, &desired, &after, &desired, &snapshot),
        FingerprintLocationV1::Fragment(path),
        FingerprintSummaryV1::LifecyclePreserved,
        FingerprintSummaryV1::LifecycleFresh,
    );
}

#[test]
fn live_retired_node_reports_preserved() {
    let transition = removal_transition();
    let after = without_node_subtree(&transition.before, &retired_member());

    expect_mismatch(
        IdentityLedgerV1::new().verify_transition(
            &transition.before,
            &transition.before_desired,
            &after,
            &transition.after_desired,
            &transition.before_snapshot,
        ),
        FingerprintLocationV1::Node(retired_member()),
        FingerprintSummaryV1::LifecycleRetired,
        FingerprintSummaryV1::LifecyclePreserved,
    );
}

#[test]
fn live_retired_fragment_reports_preserved() {
    let transition = removal_transition();
    let before = fragments_only(&transition.before);
    let after = without_fragment(&before, &retired_nested_fragment());

    expect_mismatch(
        IdentityLedgerV1::new().verify_transition(
            &before,
            &transition.before_desired,
            &after,
            &transition.after_desired,
            &transition.before_snapshot,
        ),
        FingerprintLocationV1::Fragment(retired_nested_fragment()),
        FingerprintSummaryV1::LifecycleRetired,
        FingerprintSummaryV1::LifecyclePreserved,
    );
}

#[test]
fn reused_retired_node_reports_preserved_instead_of_fresh() {
    let transition = removal_transition();
    let mut ledger = IdentityLedgerV1::new();
    assert!(
        ledger
            .verify_transition(
                &transition.before,
                &transition.before_desired,
                &transition.after,
                &transition.after_desired,
                &transition.after_snapshot,
            )
            .expect("valid retirement should verify")
            .is_none()
    );
    let path = retired_member();
    let old = transition
        .before
        .node(&path)
        .expect("retired member should have an old handle");
    let mut reintroduced = transition.after_desired.clone();
    reintroduced
        .apply_operation(
            &SemanticOperationV1::InsertKeyed {
                fragment: primary_fragment(),
                key: 7,
                final_index: 0,
            },
            transition.fixture.harness_limits(),
        )
        .expect("desired reintroduction should apply");
    let mut after = copy_index(&transition.after);
    assert!(after.record_node(path.clone(), old));

    expect_mismatch(
        ledger.verify_transition(
            &transition.after,
            &transition.after_desired,
            &after,
            &reintroduced,
            &transition.before_snapshot,
        ),
        FingerprintLocationV1::Node(path),
        FingerprintSummaryV1::LifecycleFresh,
        FingerprintSummaryV1::LifecyclePreserved,
    );
}

#[test]
fn reused_retired_fragment_reports_preserved_instead_of_fresh() {
    let transition = removal_transition();
    let mut ledger = IdentityLedgerV1::new();
    assert!(
        ledger
            .verify_transition(
                &transition.before,
                &transition.before_desired,
                &transition.after,
                &transition.after_desired,
                &transition.after_snapshot,
            )
            .expect("valid retirement should verify")
            .is_none()
    );
    let path = retired_nested_fragment();
    let old = transition
        .before
        .fragment(&path)
        .expect("retired fragment should have an old handle");
    let mut reintroduced = transition.after_desired.clone();
    reintroduced
        .apply_operation(
            &SemanticOperationV1::InsertKeyed {
                fragment: primary_fragment(),
                key: 7,
                final_index: 0,
            },
            transition.fixture.harness_limits(),
        )
        .expect("desired reintroduction should apply");
    let mut after = copy_index(&transition.after);
    assert!(after.record_fragment(path.clone(), old));

    expect_mismatch(
        ledger.verify_transition(
            &transition.after,
            &transition.after_desired,
            &after,
            &reintroduced,
            &transition.before_snapshot,
        ),
        FingerprintLocationV1::Fragment(path),
        FingerprintSummaryV1::LifecycleFresh,
        FingerprintSummaryV1::LifecyclePreserved,
    );
}

#[test]
fn node_mismatch_precedes_fragment_mismatch() {
    let (_fixture, desired, snapshot, before) = initial_state();
    let (_foreign_fixture, _foreign_desired, _foreign_snapshot, foreign) = initial_state();
    let node_path = NodePathV1::root();
    let fragment_path = primary_fragment();
    let after = replace_node(
        &replace_fragment(
            &before,
            &fragment_path,
            foreign
                .fragment(&fragment_path)
                .expect("foreign fragment should exist"),
        ),
        &node_path,
        foreign.node(&node_path).expect("foreign root should exist"),
    );

    expect_mismatch(
        IdentityLedgerV1::new().verify_transition(&before, &desired, &after, &desired, &snapshot),
        FingerprintLocationV1::Node(node_path),
        FingerprintSummaryV1::LifecyclePreserved,
        FingerprintSummaryV1::LifecycleFresh,
    );
}
