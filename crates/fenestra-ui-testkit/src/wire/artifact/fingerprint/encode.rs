use std::fmt::Write as _;

use crate::failure::ReplayFailureV1;
use crate::fingerprint::{
    FailureFingerprintKindV1, FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1,
};
use crate::semantic::NormalizedChildGroupV1;
use crate::wire::path::{write_fragment_path, write_node_path};
use crate::wire::primitive::write_property_value;

use super::FailureScopeV1;

pub(in crate::wire::artifact) fn write_failure_v1(
    output: &mut String,
    scope: FailureScopeV1,
    failure: &ReplayFailureV1,
) {
    let _ = write!(
        output,
        "failure|{}|{}|",
        scope_word(scope),
        failure.transaction().get()
    );
    match failure.operation() {
        Some(operation) => {
            let _ = write!(output, "{}", operation.get());
        }
        None => output.push('-'),
    }
    let fingerprint = failure.fingerprint();
    let _ = write!(output, "|{}|", kind_word(fingerprint.kind()));
    write_location(output, fingerprint.location());
    let _ = write!(output, "|{}|", field_word(fingerprint.field()));
    write_summary(output, fingerprint.expected());
    output.push('|');
    write_summary(output, fingerprint.observed());
}

fn scope_word(scope: FailureScopeV1) -> &'static str {
    match scope {
        FailureScopeV1::Original => "original",
        FailureScopeV1::Minimized => "minimized",
    }
}

fn kind_word(kind: FailureFingerprintKindV1) -> &'static str {
    match kind {
        FailureFingerprintKindV1::CandidateRejected => "candidate-rejected",
        FailureFingerprintKindV1::StateMismatch => "state-mismatch",
        FailureFingerprintKindV1::IdentityMismatch => "identity-mismatch",
    }
}

fn field_word(field: FingerprintFieldV1) -> &'static str {
    match field {
        FingerprintFieldV1::CandidateOutcome => "candidate-outcome",
        FingerprintFieldV1::Template => "template",
        FingerprintFieldV1::Component => "component",
        FingerprintFieldV1::Property => "property",
        FingerprintFieldV1::Parent => "parent",
        FingerprintFieldV1::ChildOrder => "child-order",
        FingerprintFieldV1::FragmentBinding => "fragment-binding",
        FingerprintFieldV1::KeyedOrder => "keyed-order",
        FingerprintFieldV1::NodeCount => "node-count",
        FingerprintFieldV1::FragmentCount => "fragment-count",
        FingerprintFieldV1::PropertyCount => "property-count",
        FingerprintFieldV1::IdentityLifecycle => "identity-lifecycle",
    }
}

fn write_location(output: &mut String, location: &FingerprintLocationV1) {
    match location {
        FingerprintLocationV1::Global => output.push_str("global"),
        FingerprintLocationV1::Node(path) => {
            output.push_str("node:");
            write_node_path(output, path);
        }
        FingerprintLocationV1::Fragment(path) => {
            output.push_str("fragment:");
            write_fragment_path(output, path);
        }
    }
}

fn write_summary(output: &mut String, summary: &FingerprintSummaryV1) {
    match summary {
        FingerprintSummaryV1::None => output.push_str("none"),
        FingerprintSummaryV1::Count(value) => {
            let _ = write!(output, "count:{value}");
        }
        FingerprintSummaryV1::Template(value) => {
            let _ = write!(output, "template:{}", value.get());
        }
        FingerprintSummaryV1::Component(value) => {
            let _ = write!(output, "component:{}", value.get());
        }
        FingerprintSummaryV1::Property(property, value) => {
            let _ = write!(output, "property:{}:", property.get());
            write_property_value(output, value);
        }
        FingerprintSummaryV1::Node(path) => {
            output.push_str("node:");
            write_node_path(output, path);
        }
        FingerprintSummaryV1::Nodes(paths) => {
            output.push_str("nodes:");
            write_list(output, paths, write_node_path);
        }
        FingerprintSummaryV1::Children(children) => {
            output.push_str("children:");
            write_list(output, children, write_child);
        }
        FingerprintSummaryV1::Keys(keys) => {
            output.push_str("keys:");
            write_list(output, keys, |output, key| {
                let _ = write!(output, "{key}");
            });
        }
        FingerprintSummaryV1::BindingPresent => output.push_str("binding:present"),
        FingerprintSummaryV1::BindingAbsent => output.push_str("binding:absent"),
        FingerprintSummaryV1::CandidateAccepted => output.push_str("kind:accept"),
        FingerprintSummaryV1::CandidateRejected(rejection) => {
            let _ = write!(output, "kind:{}", rejection.code());
        }
        FingerprintSummaryV1::LifecycleAbsent => output.push_str("absent"),
        FingerprintSummaryV1::LifecyclePreserved => output.push_str("preserved"),
        FingerprintSummaryV1::LifecycleFresh => output.push_str("fresh"),
        FingerprintSummaryV1::LifecycleRetired => output.push_str("retired"),
        FingerprintSummaryV1::LifecycleDistinct => output.push_str("distinct"),
        FingerprintSummaryV1::LifecycleAliased => output.push_str("aliased"),
    }
}

fn write_child(output: &mut String, child: &NormalizedChildGroupV1) {
    match child {
        NormalizedChildGroupV1::Static(path) => {
            output.push_str("s:");
            write_node_path(output, path);
        }
        NormalizedChildGroupV1::Region(path) => {
            output.push_str("r:");
            write_fragment_path(output, path);
        }
    }
}

fn write_list<T>(output: &mut String, values: &[T], mut write_value: impl FnMut(&mut String, &T)) {
    if values.is_empty() {
        output.push('-');
        return;
    }
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        write_value(output, value);
    }
}
