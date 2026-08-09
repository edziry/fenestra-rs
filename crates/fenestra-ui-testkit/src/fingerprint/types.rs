use fenestra_ui_ir::prototype::{ComponentTypeId, PropertyId, PropertyValue, TemplateNodeId};

use crate::semantic::{FragmentPathV1, NodePathV1, NormalizedChildGroupV1};
use crate::trace::CandidateRejectionV1;

/// Closed failure classes retained by a V1 failure fingerprint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureFingerprintKindV1 {
    /// The candidate rejected a transaction that the clean model accepted.
    CandidateRejected,
    /// Clean reconstruction and candidate observation had unequal state.
    StateMismatch,
    /// A physical runtime identity broke its semantic lifecycle contract.
    IdentityMismatch,
}

/// Stable semantic location of one V1 failure fingerprint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FingerprintLocationV1 {
    /// A failure that applies to the complete normalized state.
    Global,
    /// A failure at one semantic node.
    Node(NodePathV1),
    /// A failure at one semantic structural fragment.
    Fragment(FragmentPathV1),
}

/// Closed normalized fields addressable by a V1 failure fingerprint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FingerprintFieldV1 {
    /// Candidate acceptance versus one closed rejection reason.
    CandidateOutcome,
    /// Authored node template.
    Template,
    /// Authored node component.
    Component,
    /// One schema-ordered effective property.
    Property,
    /// Semantic parent node.
    Parent,
    /// Authored groups or flattened direct-child order.
    ChildOrder,
    /// Presence of one authored structural fragment.
    FragmentBinding,
    /// Committed keyed-member order in one fragment.
    KeyedOrder,
    /// Total normalized node count.
    NodeCount,
    /// Total normalized fragment count.
    FragmentCount,
    /// Total normalized property-slot count.
    PropertyCount,
    /// Physical identity state across one semantic lifecycle.
    IdentityLifecycle,
}

/// Closed values used as expected and observed V1 fingerprint summaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FingerprintSummaryV1 {
    /// The addressed semantic value was absent.
    None,
    /// One bounded normalized-state count.
    Count(u32),
    /// One authored template symbol.
    Template(TemplateNodeId),
    /// One authored component symbol.
    Component(ComponentTypeId),
    /// One property symbol and its effective value.
    Property(PropertyId, PropertyValue),
    /// One semantic node path.
    Node(NodePathV1),
    /// Flat direct-child paths in committed order.
    Nodes(Vec<NodePathV1>),
    /// Authored child groups in slot order.
    Children(Vec<NormalizedChildGroupV1>),
    /// Fragment-local keys in committed order.
    Keys(Vec<u64>),
    /// An authored fragment binding was present.
    BindingPresent,
    /// An authored fragment binding was absent.
    BindingAbsent,
    /// The clean model accepted the candidate transaction.
    CandidateAccepted,
    /// The candidate returned one closed rejection reason.
    CandidateRejected(CandidateRejectionV1),
    /// No physical identity existed at the semantic path.
    LifecycleAbsent,
    /// A surviving semantic path preserved its physical identity.
    LifecyclePreserved,
    /// A reintroduced semantic path received a fresh physical identity.
    LifecycleFresh,
    /// A removed semantic path retired its physical identity.
    LifecycleRetired,
    /// Two semantic paths had distinct physical identities.
    LifecycleDistinct,
    /// Two semantic paths incorrectly shared one physical identity.
    LifecycleAliased,
}

/// First unequal normalized field, independent of transaction position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureFingerprintV1 {
    kind: FailureFingerprintKindV1,
    location: FingerprintLocationV1,
    field: FingerprintFieldV1,
    expected: FingerprintSummaryV1,
    observed: FingerprintSummaryV1,
}

impl FailureFingerprintV1 {
    pub(crate) fn candidate_rejected(rejection: CandidateRejectionV1) -> Self {
        Self {
            kind: FailureFingerprintKindV1::CandidateRejected,
            location: FingerprintLocationV1::Global,
            field: FingerprintFieldV1::CandidateOutcome,
            expected: FingerprintSummaryV1::CandidateAccepted,
            observed: FingerprintSummaryV1::CandidateRejected(rejection),
        }
    }

    pub(crate) fn state_mismatch(
        location: FingerprintLocationV1,
        field: FingerprintFieldV1,
        expected: FingerprintSummaryV1,
        observed: FingerprintSummaryV1,
    ) -> Option<Self> {
        Self::from_parts(
            FailureFingerprintKindV1::StateMismatch,
            location,
            field,
            expected,
            observed,
        )
    }

    pub(crate) fn identity_mismatch(
        location: FingerprintLocationV1,
        expected: FingerprintSummaryV1,
        observed: FingerprintSummaryV1,
    ) -> Option<Self> {
        Self::from_parts(
            FailureFingerprintKindV1::IdentityMismatch,
            location,
            FingerprintFieldV1::IdentityLifecycle,
            expected,
            observed,
        )
    }

    pub(crate) fn from_parts(
        kind: FailureFingerprintKindV1,
        location: FingerprintLocationV1,
        field: FingerprintFieldV1,
        expected: FingerprintSummaryV1,
        observed: FingerprintSummaryV1,
    ) -> Option<Self> {
        if !valid_combination(kind, &location, field, &expected, &observed) {
            return None;
        }
        Some(Self {
            kind,
            location,
            field,
            expected,
            observed,
        })
    }

    /// Returns the closed failure class.
    #[must_use]
    pub const fn kind(&self) -> FailureFingerprintKindV1 {
        self.kind
    }

    /// Returns the semantic location independent of transaction position.
    #[must_use]
    pub const fn location(&self) -> &FingerprintLocationV1 {
        &self.location
    }

    /// Returns the first unequal normalized field.
    #[must_use]
    pub const fn field(&self) -> FingerprintFieldV1 {
        self.field
    }

    /// Returns the clean-model summary.
    #[must_use]
    pub const fn expected(&self) -> &FingerprintSummaryV1 {
        &self.expected
    }

    /// Returns the candidate-observation summary.
    #[must_use]
    pub const fn observed(&self) -> &FingerprintSummaryV1 {
        &self.observed
    }
}

fn valid_combination(
    kind: FailureFingerprintKindV1,
    location: &FingerprintLocationV1,
    field: FingerprintFieldV1,
    expected: &FingerprintSummaryV1,
    observed: &FingerprintSummaryV1,
) -> bool {
    if expected == observed {
        return false;
    }
    match (kind, location, field) {
        (
            FailureFingerprintKindV1::CandidateRejected,
            FingerprintLocationV1::Global,
            FingerprintFieldV1::CandidateOutcome,
        ) => {
            matches!(expected, FingerprintSummaryV1::CandidateAccepted)
                && matches!(observed, FingerprintSummaryV1::CandidateRejected(_))
        }
        (FailureFingerprintKindV1::StateMismatch, FingerprintLocationV1::Node(_), field) => {
            valid_node_state(field, expected, observed)
        }
        (
            FailureFingerprintKindV1::StateMismatch,
            FingerprintLocationV1::Fragment(_),
            FingerprintFieldV1::FragmentBinding,
        ) => {
            matches!(expected, FingerprintSummaryV1::BindingPresent)
                && matches!(observed, FingerprintSummaryV1::BindingAbsent)
        }
        (
            FailureFingerprintKindV1::StateMismatch,
            FingerprintLocationV1::Fragment(_),
            FingerprintFieldV1::KeyedOrder,
        ) => {
            matches!(expected, FingerprintSummaryV1::Keys(_))
                && matches!(observed, FingerprintSummaryV1::Keys(_))
        }
        (FailureFingerprintKindV1::StateMismatch, FingerprintLocationV1::Global, field) => {
            matches!(
                field,
                FingerprintFieldV1::NodeCount
                    | FingerprintFieldV1::FragmentCount
                    | FingerprintFieldV1::PropertyCount
            ) && matches!(expected, FingerprintSummaryV1::Count(_))
                && matches!(observed, FingerprintSummaryV1::Count(_))
        }
        (
            FailureFingerprintKindV1::IdentityMismatch,
            FingerprintLocationV1::Node(_) | FingerprintLocationV1::Fragment(_),
            FingerprintFieldV1::IdentityLifecycle,
        ) => is_lifecycle(expected) && is_lifecycle(observed),
        _ => false,
    }
}

fn valid_node_state(
    field: FingerprintFieldV1,
    expected: &FingerprintSummaryV1,
    observed: &FingerprintSummaryV1,
) -> bool {
    match field {
        FingerprintFieldV1::Template => {
            matches!(expected, FingerprintSummaryV1::Template(_))
                && matches!(
                    observed,
                    FingerprintSummaryV1::Template(_) | FingerprintSummaryV1::None
                )
        }
        FingerprintFieldV1::Component => {
            matches!(expected, FingerprintSummaryV1::Component(_))
                && matches!(
                    observed,
                    FingerprintSummaryV1::Component(_) | FingerprintSummaryV1::None
                )
        }
        FingerprintFieldV1::Property => {
            matches!(expected, FingerprintSummaryV1::Property(_, _))
                && matches!(
                    observed,
                    FingerprintSummaryV1::Property(_, _) | FingerprintSummaryV1::None
                )
        }
        FingerprintFieldV1::Parent => is_node_or_none(expected) && is_node_or_none(observed),
        FingerprintFieldV1::ChildOrder => matches!(
            (expected, observed),
            (
                FingerprintSummaryV1::Nodes(_),
                FingerprintSummaryV1::Nodes(_)
            ) | (
                FingerprintSummaryV1::Children(_),
                FingerprintSummaryV1::Children(_)
            )
        ),
        _ => false,
    }
}

fn is_node_or_none(summary: &FingerprintSummaryV1) -> bool {
    matches!(
        summary,
        FingerprintSummaryV1::Node(_) | FingerprintSummaryV1::None
    )
}

fn is_lifecycle(summary: &FingerprintSummaryV1) -> bool {
    matches!(
        summary,
        FingerprintSummaryV1::LifecycleAbsent
            | FingerprintSummaryV1::LifecyclePreserved
            | FingerprintSummaryV1::LifecycleFresh
            | FingerprintSummaryV1::LifecycleRetired
            | FingerprintSummaryV1::LifecycleDistinct
            | FingerprintSummaryV1::LifecycleAliased
    )
}
