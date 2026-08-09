use fenestra_ui_ir::prototype::{
    IrValidationErrorKind, SourceSpan, ValidationLimitKind, ValidationLimits,
};

use super::TEST_LIMITS;
use super::malformed::Fault;
use super::span;

pub fn limit_case(fault: Fault) -> Option<(ValidationLimits, IrValidationErrorKind, SourceSpan)> {
    let (limits, kind, source) = match fault {
        Fault::LimitComponents => (
            TEST_LIMITS.with_components(0),
            ValidationLimitKind::Components,
            span(1),
        ),
        Fault::LimitProperties => (
            TEST_LIMITS.with_properties(0),
            ValidationLimitKind::Properties,
            span(2),
        ),
        Fault::LimitTemplates => (
            TEST_LIMITS.with_templates(1),
            ValidationLimitKind::Templates,
            span(6),
        ),
        Fault::LimitRegions => (
            TEST_LIMITS.with_regions(0),
            ValidationLimitKind::Regions,
            span(8),
        ),
        Fault::LimitChildSlots => (
            TEST_LIMITS.with_child_slots(0),
            ValidationLimitKind::ChildSlots,
            span(7),
        ),
        Fault::LimitInitialProperties => (
            TEST_LIMITS.with_initial_properties(0),
            ValidationLimitKind::InitialProperties,
            span(9),
        ),
        Fault::LimitInitialKeys => (
            TEST_LIMITS.with_initial_keys(0),
            ValidationLimitKind::InitialKeys,
            span(10),
        ),
        Fault::LimitTemplateDepth => (
            TEST_LIMITS.with_template_depth(1),
            ValidationLimitKind::TemplateDepth,
            span(8),
        ),
        Fault::LimitInitialInstances => (
            TEST_LIMITS.with_initial_instances(1),
            ValidationLimitKind::InitialInstances,
            span(8),
        ),
        _ => return None,
    };

    Some((limits, IrValidationErrorKind::LimitExceeded(kind), source))
}
