use crate::error::{HarnessError, HarnessErrorKind};
use crate::semantic::{NodePathV1, NormalizedChildGroupV1, NormalizedNodeV1, NormalizedStateV1};

use super::types::{
    FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1,
};

pub(crate) fn compare_normalized_state_v1(
    expected: &NormalizedStateV1,
    observed: &NormalizedStateV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    if let Some(fingerprint) = compare_nodes(expected, observed)? {
        return Ok(Some(fingerprint));
    }
    if let Some(fingerprint) = compare_fragments(expected, observed)? {
        return Ok(Some(fingerprint));
    }
    if let Some(fingerprint) = compare_child_order(expected, observed)? {
        return Ok(Some(fingerprint));
    }
    compare_counts(expected, observed)
}

fn compare_nodes(
    expected: &NormalizedStateV1,
    observed: &NormalizedStateV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    for expected_node in expected.nodes() {
        let location = FingerprintLocationV1::Node(expected_node.path().clone());
        let Some(observed_node) = observed.node(expected_node.path()) else {
            return state_fingerprint(
                location,
                FingerprintFieldV1::Template,
                FingerprintSummaryV1::Template(expected_node.template()),
                FingerprintSummaryV1::None,
            )
            .map(Some);
        };
        if expected_node.template() != observed_node.template() {
            return state_fingerprint(
                location,
                FingerprintFieldV1::Template,
                FingerprintSummaryV1::Template(expected_node.template()),
                FingerprintSummaryV1::Template(observed_node.template()),
            )
            .map(Some);
        }
        if expected_node.component() != observed_node.component() {
            return state_fingerprint(
                location,
                FingerprintFieldV1::Component,
                FingerprintSummaryV1::Component(expected_node.component()),
                FingerprintSummaryV1::Component(observed_node.component()),
            )
            .map(Some);
        }
        if let Some(fingerprint) = compare_properties(expected_node, observed_node)? {
            return Ok(Some(fingerprint));
        }
        if expected_node.parent() != observed_node.parent() {
            return state_fingerprint(
                location,
                FingerprintFieldV1::Parent,
                node_summary(expected_node.parent()),
                node_summary(observed_node.parent()),
            )
            .map(Some);
        }
    }
    Ok(None)
}

fn compare_properties(
    expected: &NormalizedNodeV1,
    observed: &NormalizedNodeV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    for (index, property) in expected.properties().iter().enumerate() {
        let expected_summary =
            FingerprintSummaryV1::Property(property.property(), property.value().clone());
        let observed_property = observed.properties().get(index);
        let observed_summary = observed_property.map_or(FingerprintSummaryV1::None, |candidate| {
            FingerprintSummaryV1::Property(candidate.property(), candidate.value().clone())
        });
        if expected_summary != observed_summary {
            return state_fingerprint(
                FingerprintLocationV1::Node(expected.path().clone()),
                FingerprintFieldV1::Property,
                expected_summary,
                observed_summary,
            )
            .map(Some);
        }
    }
    Ok(None)
}

fn compare_fragments(
    expected: &NormalizedStateV1,
    observed: &NormalizedStateV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    for expected_fragment in expected.fragments() {
        let location = FingerprintLocationV1::Fragment(expected_fragment.path().clone());
        let Some(observed_fragment) = observed.fragment(expected_fragment.path()) else {
            return state_fingerprint(
                location,
                FingerprintFieldV1::FragmentBinding,
                FingerprintSummaryV1::BindingPresent,
                FingerprintSummaryV1::BindingAbsent,
            )
            .map(Some);
        };
        if expected_fragment.descriptor() != observed_fragment.descriptor() {
            return state_fingerprint(
                location,
                FingerprintFieldV1::FragmentBinding,
                FingerprintSummaryV1::BindingPresent,
                FingerprintSummaryV1::BindingAbsent,
            )
            .map(Some);
        }
        let expected_keys = expected_fragment
            .members()
            .iter()
            .map(|member| member.key())
            .collect::<Vec<_>>();
        let observed_keys = observed_fragment
            .members()
            .iter()
            .map(|member| member.key())
            .collect::<Vec<_>>();
        if expected_keys != observed_keys {
            return state_fingerprint(
                location,
                FingerprintFieldV1::KeyedOrder,
                FingerprintSummaryV1::Keys(expected_keys),
                FingerprintSummaryV1::Keys(observed_keys),
            )
            .map(Some);
        }
    }
    Ok(None)
}

fn compare_child_order(
    expected: &NormalizedStateV1,
    observed: &NormalizedStateV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    for expected_node in expected.nodes() {
        let Some(observed_node) = observed.node(expected_node.path()) else {
            continue;
        };
        if expected_node.child_groups() != observed_node.child_groups() {
            return state_fingerprint(
                FingerprintLocationV1::Node(expected_node.path().clone()),
                FingerprintFieldV1::ChildOrder,
                FingerprintSummaryV1::Children(expected_node.child_groups().to_vec()),
                FingerprintSummaryV1::Children(observed_node.child_groups().to_vec()),
            )
            .map(Some);
        }
        let expected_children = flattened_children(expected, expected_node);
        let observed_children = flattened_children(observed, observed_node);
        if expected_children != observed_children {
            return state_fingerprint(
                FingerprintLocationV1::Node(expected_node.path().clone()),
                FingerprintFieldV1::ChildOrder,
                FingerprintSummaryV1::Nodes(expected_children),
                FingerprintSummaryV1::Nodes(observed_children),
            )
            .map(Some);
        }
    }
    Ok(None)
}

fn flattened_children(state: &NormalizedStateV1, node: &NormalizedNodeV1) -> Vec<NodePathV1> {
    let mut children = Vec::new();
    for group in node.child_groups() {
        match group {
            NormalizedChildGroupV1::Static(path) => children.push(path.clone()),
            NormalizedChildGroupV1::Region(path) => {
                if let Some(fragment) = state.fragment(path) {
                    children.extend(
                        fragment
                            .members()
                            .iter()
                            .map(|member| member.node().clone()),
                    );
                }
            }
        }
    }
    children
}

fn compare_counts(
    expected: &NormalizedStateV1,
    observed: &NormalizedStateV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    let counts = [
        (
            FingerprintFieldV1::NodeCount,
            expected.node_count(),
            observed.node_count(),
        ),
        (
            FingerprintFieldV1::FragmentCount,
            expected.fragment_count(),
            observed.fragment_count(),
        ),
        (
            FingerprintFieldV1::PropertyCount,
            expected.property_slot_count(),
            observed.property_slot_count(),
        ),
    ];
    for (field, expected_count, observed_count) in counts {
        let expected_count = u32::try_from(expected_count).map_err(|_| arithmetic_error())?;
        let observed_count = u32::try_from(observed_count).map_err(|_| arithmetic_error())?;
        if expected_count != observed_count {
            return state_fingerprint(
                FingerprintLocationV1::Global,
                field,
                FingerprintSummaryV1::Count(expected_count),
                FingerprintSummaryV1::Count(observed_count),
            )
            .map(Some);
        }
    }
    Ok(None)
}

fn node_summary(path: Option<&NodePathV1>) -> FingerprintSummaryV1 {
    path.map_or(FingerprintSummaryV1::None, |path| {
        FingerprintSummaryV1::Node(path.clone())
    })
}

fn state_fingerprint(
    location: FingerprintLocationV1,
    field: FingerprintFieldV1,
    expected: FingerprintSummaryV1,
    observed: FingerprintSummaryV1,
) -> Result<FailureFingerprintV1, HarnessError> {
    FailureFingerprintV1::state_mismatch(location, field, expected, observed)
        .ok_or_else(state_error)
}

fn state_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::StateMismatch)
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}
