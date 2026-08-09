use fenestra_ui_runtime::prototype::{
    CapacityKind, HeadlessProjectionErrorKind, HeadlessProjectionLimitKind, SchedulerErrorKind,
    SchedulerLane, TransactionErrorKind,
};

pub(super) fn scheduler_error_word_v1(value: &str) -> bool {
    parse_scheduler_error_v1(value).is_some()
}

pub(super) fn parse_scheduler_error_v1(value: &str) -> Option<SchedulerErrorKind> {
    match value {
        "retained-generation-capacity" => Some(SchedulerErrorKind::RetainedGenerationCapacity),
        "tick-regression" => Some(SchedulerErrorKind::TickRegression),
        "input-out-of-order" => Some(SchedulerErrorKind::InputOutOfOrder),
        "frame-id-mismatch" => Some(SchedulerErrorKind::FrameIdMismatch),
        "control-pending" => Some(SchedulerErrorKind::ControlPending),
        "arithmetic-exhausted" => Some(SchedulerErrorKind::ArithmeticExhausted),
        "foreign-renderer-epoch" => Some(SchedulerErrorKind::ForeignRendererEpoch),
        "completion-regression" => Some(SchedulerErrorKind::CompletionRegression),
        "completion-beyond-accepted" => Some(SchedulerErrorKind::CompletionBeyondAccepted),
        _ => parse_lane_error(value).or_else(|| {
            value
                .strip_prefix("transaction-")
                .and_then(parse_transaction)
                .map(SchedulerErrorKind::Transaction)
        }),
    }
}

fn parse_lane_error(value: &str) -> Option<SchedulerErrorKind> {
    for (prefix, make) in [
        (
            "capacity-too-small-",
            SchedulerErrorKind::CapacityTooSmall as fn(SchedulerLane) -> SchedulerErrorKind,
        ),
        ("capacity-exceeded-", SchedulerErrorKind::CapacityExceeded),
        ("residence-exceeded-", SchedulerErrorKind::ResidenceExceeded),
    ] {
        if let Some(lane) = value.strip_prefix(prefix).and_then(parse_lane) {
            return Some(make(lane));
        }
    }
    None
}

fn parse_lane(value: &str) -> Option<SchedulerLane> {
    match value {
        "deferred" => Some(SchedulerLane::Deferred),
        "controls" => Some(SchedulerLane::Controls),
        "visual" => Some(SchedulerLane::Visual),
        "in-flight" => Some(SchedulerLane::InFlight),
        _ => None,
    }
}

fn parse_transaction(value: &str) -> Option<TransactionErrorKind> {
    match value {
        "headless-unavailable" => Some(TransactionErrorKind::HeadlessUnavailable),
        "stale-base" => Some(TransactionErrorKind::StaleBase),
        "missing-node" => Some(TransactionErrorKind::MissingNode),
        "missing-fragment" => Some(TransactionErrorKind::MissingFragment),
        "missing-key" => Some(TransactionErrorKind::MissingKey),
        "duplicate-key" => Some(TransactionErrorKind::DuplicateKey),
        "unknown-property" => Some(TransactionErrorKind::UnknownProperty),
        "property-type-mismatch" => Some(TransactionErrorKind::PropertyTypeMismatch),
        "index-out-of-bounds" => Some(TransactionErrorKind::IndexOutOfBounds),
        "generation-exhausted" => Some(TransactionErrorKind::GenerationExhausted),
        "invariant-violation" => Some(TransactionErrorKind::InvariantViolation),
        _ => value
            .strip_prefix("capacity-exceeded-")
            .and_then(parse_capacity)
            .map(TransactionErrorKind::CapacityExceeded)
            .or_else(|| {
                value
                    .strip_prefix("headless-")
                    .and_then(parse_headless)
                    .map(TransactionErrorKind::Headless)
            }),
    }
}

fn parse_capacity(value: &str) -> Option<CapacityKind> {
    match value {
        "operations" => Some(CapacityKind::Operations),
        "structural" => Some(CapacityKind::StructuralChanges),
        "live-nodes" => Some(CapacityKind::LiveNodes),
        "live-fragments" => Some(CapacityKind::LiveFragments),
        "live-properties" => Some(CapacityKind::LivePropertySlots),
        "retained-generations" => Some(CapacityKind::RetainedGenerations),
        _ => None,
    }
}

fn parse_headless(value: &str) -> Option<HeadlessProjectionErrorKind> {
    match value {
        "missing-specification-target-or-property" => {
            Some(HeadlessProjectionErrorKind::MissingSpecificationTargetOrProperty)
        }
        "property-type-mismatch" => Some(HeadlessProjectionErrorKind::PropertyTypeMismatch),
        "invalid-semantic-template" => Some(HeadlessProjectionErrorKind::InvalidSemanticTemplate),
        "invalid-surface" => Some(HeadlessProjectionErrorKind::InvalidSurface),
        "negative-geometry" => Some(HeadlessProjectionErrorKind::NegativeGeometry),
        "arithmetic-exhausted" => Some(HeadlessProjectionErrorKind::ArithmeticExhausted),
        "invariant-violation" => Some(HeadlessProjectionErrorKind::InvariantViolation),
        _ => value
            .strip_prefix("capacity-exceeded-")
            .and_then(parse_projection_limit)
            .map(HeadlessProjectionErrorKind::CapacityExceeded),
    }
}

fn parse_projection_limit(value: &str) -> Option<HeadlessProjectionLimitKind> {
    match value {
        "computed-styles" => Some(HeadlessProjectionLimitKind::ComputedStyles),
        "geometry" => Some(HeadlessProjectionLimitKind::Geometry),
        "semantics" => Some(HeadlessProjectionLimitKind::Semantics),
        "hit-regions" => Some(HeadlessProjectionLimitKind::HitRegions),
        "scene-rectangles" => Some(HeadlessProjectionLimitKind::SceneRectangles),
        _ => None,
    }
}
