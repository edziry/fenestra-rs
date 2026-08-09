use fenestra_ui_ir::prototype::TemplateNodeId;

use super::support::{NodeViewDefectV1, ObservationCaseV1, ViewDefectV1, assert_state_mismatch};
use crate::error::{HarnessError, HarnessErrorKind, HarnessLimitKind};
use crate::fingerprint::{FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1};
use crate::observe::ObservationOutcomeV1;
use crate::semantic::{FragmentPathV1, NodePathV1};

#[test]
fn individual_reported_counts_are_fingerprinted() {
    let cases = [
        (
            reported_counts(10, 4, 14),
            FingerprintFieldV1::NodeCount,
            FingerprintSummaryV1::Count(9),
            FingerprintSummaryV1::Count(10),
        ),
        (
            reported_counts(9, 5, 14),
            FingerprintFieldV1::FragmentCount,
            FingerprintSummaryV1::Count(4),
            FingerprintSummaryV1::Count(5),
        ),
        (
            reported_counts(9, 4, 15),
            FingerprintFieldV1::PropertyCount,
            FingerprintSummaryV1::Count(14),
            FingerprintSummaryV1::Count(15),
        ),
    ];

    for (defect, field, expected, observed) in cases {
        let case = ObservationCaseV1::initial();
        assert_state_mismatch(
            case.observe_defects(vec![defect]),
            FingerprintLocationV1::Global,
            field,
            expected,
            observed,
        );
    }
}

#[test]
fn node_count_precedes_other_reported_count_differences() {
    let case = ObservationCaseV1::initial();

    assert_state_mismatch(
        case.observe_defects(vec![reported_counts(10, 5, 15)]),
        FingerprintLocationV1::Global,
        FingerprintFieldV1::NodeCount,
        FingerprintSummaryV1::Count(9),
        FingerprintSummaryV1::Count(10),
    );
}

#[test]
fn child_order_precedes_all_reported_count_differences() {
    let case = ObservationCaseV1::initial();
    let root = NodePathV1::root();
    let static_child = root.clone().static_child(0);
    let member_seven = root.clone().member(1, 7);
    let member_eight = root.clone().member(1, 8);
    let secondary_member = root.clone().member(2, 7);

    assert_state_mismatch(
        case.observe_defects(vec![
            ViewDefectV1::SwappedChildren {
                node: root.clone(),
                left: 1,
                right: 2,
            },
            reported_counts(10, 5, 15),
        ]),
        FingerprintLocationV1::Node(root),
        FingerprintFieldV1::ChildOrder,
        FingerprintSummaryV1::Nodes(vec![
            static_child.clone(),
            member_seven.clone(),
            member_eight.clone(),
            secondary_member.clone(),
        ]),
        FingerprintSummaryV1::Nodes(vec![
            static_child,
            member_eight,
            member_seven,
            secondary_member,
        ]),
    );
}

#[test]
fn inclusive_reported_count_ceilings_remain_fingerprintable() {
    let cases = [
        (
            reported_counts(256, 4, 14),
            FingerprintFieldV1::NodeCount,
            FingerprintSummaryV1::Count(9),
            FingerprintSummaryV1::Count(256),
        ),
        (
            reported_counts(9, 128, 14),
            FingerprintFieldV1::FragmentCount,
            FingerprintSummaryV1::Count(4),
            FingerprintSummaryV1::Count(128),
        ),
        (
            reported_counts(9, 4, 1_024),
            FingerprintFieldV1::PropertyCount,
            FingerprintSummaryV1::Count(14),
            FingerprintSummaryV1::Count(1_024),
        ),
    ];

    for (defect, field, expected, observed) in cases {
        let case = ObservationCaseV1::initial();
        assert_state_mismatch(
            case.observe_defects(vec![defect]),
            FingerprintLocationV1::Global,
            field,
            expected,
            observed,
        );
    }
}

#[test]
fn reported_counts_above_their_ceilings_are_limits() {
    for (defect, kind) in above_ceiling_counts() {
        let case = ObservationCaseV1::initial();
        assert_limit(case.observe_defects(vec![defect]), kind);
    }
}

#[test]
fn reported_count_limits_precede_an_early_template_mismatch() {
    for (defect, kind) in above_ceiling_counts() {
        let case = ObservationCaseV1::initial();
        assert_limit(
            case.observe_defects(vec![
                defect,
                NodeViewDefectV1::Template {
                    node: NodePathV1::root(),
                    observed: Some(TemplateNodeId::new(1)),
                }
                .into(),
            ]),
            kind,
        );
    }
}

#[test]
fn live_membership_limit_precedes_raw_child_count_limit() {
    let case = ObservationCaseV1::initial();
    let root = NodePathV1::root();
    let primary = FragmentPathV1::new(root.clone(), 1);

    assert_limit(
        case.observe_defects(vec![
            ViewDefectV1::ReportedChildCount {
                node: root,
                count: 257,
            },
            ViewDefectV1::ReportedKeyedCount {
                fragment: primary,
                count: 13,
            },
        ]),
        HarnessLimitKind::LiveMemberships,
    );
}

#[test]
fn raw_child_count_above_the_ceiling_is_a_node_limit() {
    let case = ObservationCaseV1::initial();

    assert_limit(
        case.observe_defects(vec![ViewDefectV1::ReportedChildCount {
            node: NodePathV1::root(),
            count: 257,
        }]),
        HarnessLimitKind::NormalizedNodes,
    );
}

#[test]
fn live_membership_limit_precedes_reported_node_count_limit() {
    let case = ObservationCaseV1::initial();
    let primary = FragmentPathV1::new(NodePathV1::root(), 1);

    assert_limit(
        case.observe_defects(vec![
            ViewDefectV1::ReportedKeyedCount {
                fragment: primary,
                count: 13,
            },
            reported_counts(257, 4, 14),
        ]),
        HarnessLimitKind::LiveMemberships,
    );
}

#[test]
fn extreme_keyed_count_reports_the_limit_without_arithmetic_exhaustion() {
    let case = ObservationCaseV1::initial();
    let primary = FragmentPathV1::new(NodePathV1::root(), 1);

    assert_limit(
        case.observe_defects(vec![ViewDefectV1::ReportedKeyedCount {
            fragment: primary,
            count: usize::MAX,
        }]),
        HarnessLimitKind::LiveMemberships,
    );
}

fn reported_counts(nodes: usize, fragments: usize, properties: usize) -> ViewDefectV1 {
    ViewDefectV1::ReportedCounts {
        nodes,
        fragments,
        properties,
    }
}

fn above_ceiling_counts() -> [(ViewDefectV1, HarnessLimitKind); 3] {
    [
        (
            reported_counts(257, 4, 14),
            HarnessLimitKind::NormalizedNodes,
        ),
        (
            reported_counts(9, 129, 14),
            HarnessLimitKind::NormalizedFragments,
        ),
        (
            reported_counts(9, 4, 1_025),
            HarnessLimitKind::NormalizedProperties,
        ),
    ]
}

fn assert_limit(result: Result<ObservationOutcomeV1, HarnessError>, expected: HarnessLimitKind) {
    let Err(error) = result else {
        panic!("observation should reject an exceeded limit");
    };
    assert_eq!(error.kind(), HarnessErrorKind::LimitExceeded(expected));
    assert_eq!(error.transaction(), None);
    assert_eq!(error.operation(), None);
}
