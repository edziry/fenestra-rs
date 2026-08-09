use fenestra_ui_ir::prototype::{ComponentTypeId, PropertyId, PropertyValue, TemplateNodeId};

use super::support::{NodeViewDefectV1, ObservationCaseV1, assert_state_mismatch};
use crate::fingerprint::{FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1};
use crate::observe::ObservationOutcomeV1;
use crate::semantic::NodePathV1;

#[test]
fn clean_view_completes_with_the_exact_expected_state() {
    let case = ObservationCaseV1::initial();

    let outcome = case
        .observe(None)
        .expect("clean diagnostic observation should succeed");
    let ObservationOutcomeV1::Complete(observed) = outcome else {
        panic!("clean observation should be complete");
    };

    assert_eq!(observed.normalized(), case.expected());
}

#[test]
fn root_template_changes_and_absence_are_fingerprinted() {
    let root = NodePathV1::root();
    let cases = [
        (
            Some(TemplateNodeId::new(1)),
            FingerprintSummaryV1::Template(TemplateNodeId::new(1)),
        ),
        (None, FingerprintSummaryV1::None),
    ];

    for (observed, observed_summary) in cases {
        let case = ObservationCaseV1::initial();
        assert_state_mismatch(
            case.observe(Some(NodeViewDefectV1::Template {
                node: root.clone(),
                observed,
            })),
            FingerprintLocationV1::Node(root.clone()),
            FingerprintFieldV1::Template,
            FingerprintSummaryV1::Template(TemplateNodeId::new(0)),
            observed_summary,
        );
    }
}

#[test]
fn root_component_changes_and_absence_are_fingerprinted() {
    let root = NodePathV1::root();
    let cases = [
        (
            Some(ComponentTypeId::new(1)),
            FingerprintSummaryV1::Component(ComponentTypeId::new(1)),
        ),
        (None, FingerprintSummaryV1::None),
    ];

    for (observed, observed_summary) in cases {
        let case = ObservationCaseV1::initial();
        assert_state_mismatch(
            case.observe(Some(NodeViewDefectV1::Component {
                node: root.clone(),
                observed,
            })),
            FingerprintLocationV1::Node(root.clone()),
            FingerprintFieldV1::Component,
            FingerprintSummaryV1::Component(ComponentTypeId::new(0)),
            observed_summary,
        );
    }
}

#[test]
fn root_property_changes_and_absence_are_fingerprinted() {
    let root = NodePathV1::root();
    let property = PropertyId::new(0);
    let cases = [
        (
            Some(PropertyValue::ScalarI32(121)),
            FingerprintSummaryV1::Property(property, PropertyValue::ScalarI32(121)),
        ),
        (None, FingerprintSummaryV1::None),
    ];

    for (observed, observed_summary) in cases {
        let case = ObservationCaseV1::initial();
        assert_state_mismatch(
            case.observe(Some(NodeViewDefectV1::Property {
                node: root.clone(),
                property,
                observed,
            })),
            FingerprintLocationV1::Node(root.clone()),
            FingerprintFieldV1::Property,
            FingerprintSummaryV1::Property(property, PropertyValue::ScalarI32(120)),
            observed_summary,
        );
    }
}

#[test]
fn static_child_parent_absence_and_known_difference_are_fingerprinted() {
    let root = NodePathV1::root();
    let child = root.clone().static_child(0);
    let other_parent = root.clone().member(1, 7);
    let cases = [
        (None, FingerprintSummaryV1::None),
        (
            Some(other_parent.clone()),
            FingerprintSummaryV1::Node(other_parent),
        ),
    ];

    for (observed, observed_summary) in cases {
        let case = ObservationCaseV1::initial();
        assert_state_mismatch(
            case.observe(Some(NodeViewDefectV1::Parent {
                node: child.clone(),
                observed,
            })),
            FingerprintLocationV1::Node(child.clone()),
            FingerprintFieldV1::Parent,
            FingerprintSummaryV1::Node(root.clone()),
            observed_summary,
        );
    }
}
