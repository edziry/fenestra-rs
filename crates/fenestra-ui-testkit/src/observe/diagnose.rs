use fenestra_ui_runtime::prototype::NodeId;

use super::bind::{BoundFragmentV1, BoundSnapshotV1};
use super::limits::{arithmetic_error, state_mismatch};
use super::view::SnapshotViewV1;
use crate::error::HarnessError;
use crate::fingerprint::{
    FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1,
};
use crate::identity::IdentityIndexV1;
use crate::semantic::{NodePathV1, NormalizedChildGroupV1, NormalizedNodeV1, NormalizedStateV1};

pub(super) fn diagnose_node_fields_v1<V: SnapshotViewV1 + ?Sized>(
    expected: &NormalizedStateV1,
    view: &V,
    identities: &IdentityIndexV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    for expected_node in expected.nodes() {
        let location = FingerprintLocationV1::Node(expected_node.path().clone());
        let Some(node) = identities.node(expected_node.path()) else {
            return fingerprint(
                location,
                FingerprintFieldV1::Template,
                FingerprintSummaryV1::Template(expected_node.template()),
                FingerprintSummaryV1::None,
            )
            .map(Some);
        };

        let observed_template = view.template(node);
        if observed_template != Some(expected_node.template()) {
            return fingerprint(
                location,
                FingerprintFieldV1::Template,
                FingerprintSummaryV1::Template(expected_node.template()),
                observed_template
                    .map_or(FingerprintSummaryV1::None, FingerprintSummaryV1::Template),
            )
            .map(Some);
        }

        let observed_component = view.component(node);
        if observed_component != Some(expected_node.component()) {
            return fingerprint(
                location,
                FingerprintFieldV1::Component,
                FingerprintSummaryV1::Component(expected_node.component()),
                observed_component
                    .map_or(FingerprintSummaryV1::None, FingerprintSummaryV1::Component),
            )
            .map(Some);
        }

        if let Some(fingerprint) = diagnose_properties(expected_node, node, view)? {
            return Ok(Some(fingerprint));
        }
        if let Some(fingerprint) = diagnose_parent(expected_node, node, view, identities)? {
            return Ok(Some(fingerprint));
        }
    }
    Ok(None)
}

pub(super) fn diagnose_fragments_v1(
    expected: &NormalizedStateV1,
    observed: &BoundSnapshotV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    for expected_fragment in expected.fragments() {
        let location = FingerprintLocationV1::Fragment(expected_fragment.path().clone());
        match observed
            .fragment(expected_fragment.path())
            .ok_or_else(state_mismatch)?
        {
            BoundFragmentV1::Absent => {
                return fingerprint(
                    location,
                    FingerprintFieldV1::FragmentBinding,
                    FingerprintSummaryV1::BindingPresent,
                    FingerprintSummaryV1::BindingAbsent,
                )
                .map(Some);
            }
            BoundFragmentV1::Present(members) => {
                let expected_keys = expected_fragment
                    .members()
                    .iter()
                    .map(|member| member.key())
                    .collect::<Vec<_>>();
                let observed_keys = members.iter().map(|(key, _)| *key).collect::<Vec<_>>();
                if expected_keys != observed_keys {
                    return fingerprint(
                        location,
                        FingerprintFieldV1::KeyedOrder,
                        FingerprintSummaryV1::Keys(expected_keys),
                        FingerprintSummaryV1::Keys(observed_keys),
                    )
                    .map(Some);
                }
            }
        }
    }
    Ok(None)
}

pub(super) fn diagnose_child_order_v1(
    expected: &NormalizedStateV1,
    observed: &BoundSnapshotV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    for expected_node in expected.nodes() {
        let expected_children = flattened_children(expected, expected_node)?;
        let observed_children = observed
            .children(expected_node.path())
            .ok_or_else(state_mismatch)?;
        let observed_paths =
            observed_child_paths(&expected_children, observed_children, observed.identities())?;
        if expected_children != observed_paths {
            return fingerprint(
                FingerprintLocationV1::Node(expected_node.path().clone()),
                FingerprintFieldV1::ChildOrder,
                FingerprintSummaryV1::Nodes(expected_children),
                FingerprintSummaryV1::Nodes(observed_paths),
            )
            .map(Some);
        }
    }
    Ok(None)
}

pub(super) fn diagnose_counts_v1(
    expected: &NormalizedStateV1,
    observed: &BoundSnapshotV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    let reported = observed.reported_counts();
    let counts = [
        (
            FingerprintFieldV1::NodeCount,
            expected.node_count(),
            reported.nodes(),
        ),
        (
            FingerprintFieldV1::FragmentCount,
            expected.fragment_count(),
            reported.fragments(),
        ),
        (
            FingerprintFieldV1::PropertyCount,
            expected.property_slot_count(),
            reported.properties(),
        ),
    ];
    for (field, expected_count, observed_count) in counts {
        let expected_count = u32::try_from(expected_count).map_err(|_| arithmetic_error())?;
        let observed_count = u32::try_from(observed_count).map_err(|_| arithmetic_error())?;
        if expected_count != observed_count {
            return fingerprint(
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

fn diagnose_properties<V: SnapshotViewV1 + ?Sized>(
    expected: &NormalizedNodeV1,
    node: NodeId,
    view: &V,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    for property in expected.properties() {
        let expected_summary =
            FingerprintSummaryV1::Property(property.property(), property.value().clone());
        let observed = view.property(node, property.property());
        let observed_summary = observed.map_or(FingerprintSummaryV1::None, |value| {
            FingerprintSummaryV1::Property(property.property(), value)
        });
        if expected_summary != observed_summary {
            return fingerprint(
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

fn diagnose_parent<V: SnapshotViewV1 + ?Sized>(
    expected: &NormalizedNodeV1,
    node: NodeId,
    view: &V,
    identities: &IdentityIndexV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    let expected_parent = match expected.parent() {
        Some(path) => Some(identities.node(path).ok_or_else(state_mismatch)?),
        None => None,
    };
    let observed_parent = view.parent(node);
    if observed_parent == expected_parent {
        return Ok(None);
    }

    let expected_summary = node_summary(expected.parent());
    let observed_summary = match observed_parent {
        Some(parent) => FingerprintSummaryV1::Node(unique_node_path(identities, parent)?),
        None => FingerprintSummaryV1::None,
    };
    fingerprint(
        FingerprintLocationV1::Node(expected.path().clone()),
        FingerprintFieldV1::Parent,
        expected_summary,
        observed_summary,
    )
    .map(Some)
}

fn unique_node_path(
    identities: &IdentityIndexV1,
    node: NodeId,
) -> Result<NodePathV1, HarnessError> {
    let mut matches = identities
        .nodes_in_authored_order()
        .filter(|(_, candidate)| *candidate == node)
        .map(|(path, _)| path);
    let path = matches.next().ok_or_else(state_mismatch)?;
    if matches.next().is_some() {
        return Err(state_mismatch());
    }
    Ok(path.clone())
}

fn flattened_children(
    state: &NormalizedStateV1,
    node: &NormalizedNodeV1,
) -> Result<Vec<NodePathV1>, HarnessError> {
    let mut children = Vec::new();
    for group in node.child_groups() {
        match group {
            NormalizedChildGroupV1::Static(path) => children.push(path.clone()),
            NormalizedChildGroupV1::Region(path) => {
                let fragment = state.fragment(path).ok_or_else(state_mismatch)?;
                children.extend(
                    fragment
                        .members()
                        .iter()
                        .map(|member| member.node().clone()),
                );
            }
        }
    }
    Ok(children)
}

fn observed_child_paths(
    expected: &[NodePathV1],
    observed: &[NodeId],
    identities: &IdentityIndexV1,
) -> Result<Vec<NodePathV1>, HarnessError> {
    observed
        .iter()
        .enumerate()
        .map(|(index, node)| {
            if let Some(path) = expected.get(index)
                && identities.node(path) == Some(*node)
            {
                return Ok(path.clone());
            }
            unique_node_path(identities, *node)
        })
        .collect()
}

fn node_summary(path: Option<&NodePathV1>) -> FingerprintSummaryV1 {
    path.map_or(FingerprintSummaryV1::None, |path| {
        FingerprintSummaryV1::Node(path.clone())
    })
}

fn fingerprint(
    location: FingerprintLocationV1,
    field: FingerprintFieldV1,
    expected: FingerprintSummaryV1,
    observed: FingerprintSummaryV1,
) -> Result<FailureFingerprintV1, HarnessError> {
    FailureFingerprintV1::state_mismatch(location, field, expected, observed)
        .ok_or_else(state_mismatch)
}
