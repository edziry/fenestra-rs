use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};

use super::support::{NodeViewDefectV1, ObservationCaseV1, ViewDefectV1, assert_state_mismatch};
use crate::fingerprint::{FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1};
use crate::observe::ObservationOutcomeV1;
use crate::semantic::{FragmentPathV1, NodePathV1};

#[test]
fn hidden_empty_fragment_binding_is_fingerprinted() {
    let case = ObservationCaseV1::with_empty_root_fragments();
    let primary = FragmentPathV1::new(NodePathV1::root(), 1);
    let secondary = FragmentPathV1::new(NodePathV1::root(), 2);

    assert_eq!(case.expected().node_count(), 2);
    assert_eq!(case.expected().fragment_count(), 2);
    assert_eq!(case.expected().property_slot_count(), 5);
    for fragment in [&primary, &secondary] {
        assert!(
            case.expected()
                .fragment(fragment)
                .expect("empty root fragment should remain bound")
                .members()
                .is_empty()
        );
    }
    let clean = case
        .observe_defects(Vec::new())
        .expect("clean empty-fragment observation should succeed");
    let ObservationOutcomeV1::Complete(observed) = clean else {
        panic!("clean empty-fragment observation should be complete");
    };
    assert_eq!(observed.normalized(), case.expected());

    assert_state_mismatch(
        case.observe_defects(vec![ViewDefectV1::HiddenFragmentBinding {
            fragment: secondary.clone(),
        }]),
        FingerprintLocationV1::Fragment(secondary),
        FingerprintFieldV1::FragmentBinding,
        FingerprintSummaryV1::BindingPresent,
        FingerprintSummaryV1::BindingAbsent,
    );
}

#[test]
fn swapped_complete_keyed_pairs_are_fingerprinted() {
    let case = ObservationCaseV1::initial();
    let primary = FragmentPathV1::new(NodePathV1::root(), 1);

    assert_state_mismatch(
        case.observe_defects(vec![ViewDefectV1::SwappedKeyedMembers {
            fragment: primary.clone(),
            left: 0,
            right: 1,
        }]),
        FingerprintLocationV1::Fragment(primary),
        FingerprintFieldV1::KeyedOrder,
        FingerprintSummaryV1::Keys(vec![7, 8]),
        FingerprintSummaryV1::Keys(vec![8, 7]),
    );
}

#[test]
fn swapped_known_flat_children_are_fingerprinted() {
    let case = ObservationCaseV1::initial();
    let root = NodePathV1::root();
    let static_child = root.clone().static_child(0);
    let member_seven = root.clone().member(1, 7);
    let member_eight = root.clone().member(1, 8);
    let secondary_member = root.clone().member(2, 7);

    assert_state_mismatch(
        case.observe_defects(vec![ViewDefectV1::SwappedChildren {
            node: root.clone(),
            left: 1,
            right: 2,
        }]),
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
fn keyed_order_precedes_a_later_flat_child_difference() {
    let case = ObservationCaseV1::initial();
    let root = NodePathV1::root();
    let primary = FragmentPathV1::new(root.clone(), 1);

    assert_state_mismatch(
        case.observe_defects(vec![
            ViewDefectV1::SwappedKeyedMembers {
                fragment: primary.clone(),
                left: 0,
                right: 1,
            },
            ViewDefectV1::SwappedChildren {
                node: root,
                left: 2,
                right: 3,
            },
        ]),
        FingerprintLocationV1::Fragment(primary),
        FingerprintFieldV1::KeyedOrder,
        FingerprintSummaryV1::Keys(vec![7, 8]),
        FingerprintSummaryV1::Keys(vec![8, 7]),
    );
}

#[test]
fn last_node_property_precedes_fragment_and_child_differences() {
    let case = ObservationCaseV1::initial();
    let root = NodePathV1::root();
    let primary = FragmentPathV1::new(root.clone(), 1);
    let last_node = root.clone().member(2, 7);
    let property = PropertyId::new(0);

    assert_state_mismatch(
        case.observe_defects(vec![
            ViewDefectV1::SwappedKeyedMembers {
                fragment: primary,
                left: 0,
                right: 1,
            },
            ViewDefectV1::SwappedChildren {
                node: root,
                left: 2,
                right: 3,
            },
            NodeViewDefectV1::Property {
                node: last_node.clone(),
                property,
                observed: Some(PropertyValue::ScalarI32(21)),
            }
            .into(),
        ]),
        FingerprintLocationV1::Node(last_node),
        FingerprintFieldV1::Property,
        FingerprintSummaryV1::Property(property, PropertyValue::ScalarI32(20)),
        FingerprintSummaryV1::Property(property, PropertyValue::ScalarI32(21)),
    );
}
