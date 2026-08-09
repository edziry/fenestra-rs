#[path = "support/style.rs"]
mod style_support;
mod support;

use fenestra_ui_ir::prototype::{
    InvalidationClass, LinkedStyleValueView, PropertyId, PropertyValue, StyleAssignmentIter,
    StyleAssignmentView, StyleValueOrigin, TemplateNodeId,
};

use style_support::{
    STYLE_REPLACEMENT, basic_style_program, construction, program, validate_program,
};
use support::{PROPERTY, REPEAT, ROOT, span};

fn collect_assignments<'a>(iter: StyleAssignmentIter<'a>) -> Vec<StyleAssignmentView<'a>> {
    iter.collect()
}

fn assert_linked_value(
    linked: LinkedStyleValueView<'_>,
    expected: &PropertyValue,
    origin: StyleValueOrigin,
) {
    assert_eq!(linked.value(), expected);
    assert_eq!(linked.origin(), origin);
    assert!(linked.invalidation().contains(InvalidationClass::Layout));
}

#[test]
fn linked_style_exposes_exact_assignments_in_authored_order() {
    let construction = construction();
    let style = validate_program(&construction, basic_style_program(), 1)
        .expect("style program should validate");

    assert!(style.construction().shares_domain_with(&construction));
    let assignments = collect_assignments(style.assignments());
    assert_eq!(assignments.len(), 1);

    let assignment = assignments[0];
    assert_eq!(assignment.target().id(), ROOT);
    assert_eq!(assignment.property().id(), PROPERTY);
    assert_eq!(assignment.schema_default(), &PropertyValue::ScalarI32(0));
    assert_eq!(
        assignment.replacement(),
        &PropertyValue::ScalarI32(STYLE_REPLACEMENT)
    );
    assert_eq!(assignment.origin(), StyleValueOrigin::ExactAssignment);
    assert!(
        assignment
            .invalidation()
            .contains(InvalidationClass::Layout)
    );
    assert_eq!(assignment.span(), span(21));

    let exact = style
        .assignment(ROOT, PROPERTY)
        .expect("exact assignment should resolve");
    assert_eq!(exact.replacement(), assignment.replacement());
}

#[test]
fn linked_values_match_the_manual_style_only_result() {
    let construction = construction();
    let style = validate_program(&construction, basic_style_program(), 1)
        .expect("style program should validate");
    let expected = [
        (
            ROOT,
            PropertyValue::ScalarI32(STYLE_REPLACEMENT),
            StyleValueOrigin::ExactAssignment,
        ),
        (
            REPEAT,
            PropertyValue::ScalarI32(0),
            StyleValueOrigin::SchemaDefault,
        ),
    ];

    for (target, value, origin) in &expected {
        let linked = style
            .linked_value(*target, PROPERTY)
            .expect("manual target and property should resolve");
        assert_linked_value(linked, value, *origin);
    }

    assert_eq!(
        construction
            .template(REPEAT)
            .expect("repeat template should resolve")
            .effective_value(PROPERTY),
        Some(&PropertyValue::ScalarI32(10))
    );
    assert_eq!(
        style
            .linked_value(REPEAT, PROPERTY)
            .expect("repeat style value should resolve")
            .value(),
        &PropertyValue::ScalarI32(0)
    );
    assert!(
        style
            .linked_value(TemplateNodeId::new(99), PROPERTY)
            .is_none()
    );
    assert!(style.linked_value(ROOT, PropertyId::new(99)).is_none());
}

#[test]
fn empty_and_equal_assignments_keep_typed_value_origin() {
    let construction = construction();
    let empty = validate_program(&construction, program(Vec::new()), 0)
        .expect("empty style should validate");
    assert_eq!(empty.assignments().count(), 0);
    assert_linked_value(
        empty
            .linked_value(ROOT, PROPERTY)
            .expect("schema default should resolve"),
        &PropertyValue::ScalarI32(0),
        StyleValueOrigin::SchemaDefault,
    );

    let equal = style_support::program(vec![style_support::scalar_assignment(ROOT, 0, span(22))]);
    let equal =
        validate_program(&construction, equal, 1).expect("equal assignment should validate");
    assert_linked_value(
        equal
            .linked_value(ROOT, PROPERTY)
            .expect("exact value should resolve"),
        &PropertyValue::ScalarI32(0),
        StyleValueOrigin::ExactAssignment,
    );
}
