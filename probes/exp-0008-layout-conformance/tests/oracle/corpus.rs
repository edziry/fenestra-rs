use fenestra_ui_exp_0008_layout_conformance::prototype::{
    RegisteredLayoutCaseV1, registered_layout_corpus_v1,
};

use super::support::ExpectedCaseV1;

const EXPECTED_NAMES: [&str; 23] = [
    "single-fixed-root",
    "column-two",
    "row-two",
    "nested-row-in-column",
    "asymmetric-padding",
    "column-gap-three",
    "row-gap-three",
    "clamp-below",
    "clamp-above",
    "mixed-constraints-padding",
    "main-axis-overflow",
    "cross-axis-overflow",
    "padding-equal-box",
    "zero-width-child",
    "zero-height-gap",
    "zero-width-viewport",
    "zero-height-viewport",
    "zero-by-zero-viewport",
    "large-integer-padding-gap",
    "node-ceiling",
    "child-ceiling",
    "depth-ceiling",
    "registered-runtime-fixture",
];

pub fn expected_corpus() -> Vec<ExpectedCaseV1> {
    let mut cases = super::cases_geometry::cases();
    cases.extend(super::cases_boundaries::cases());
    cases.extend(super::cases_limits::cases());
    cases
}

#[test]
fn registered_corpus_has_exact_names_and_literal_tables() {
    let registered = registered_layout_corpus_v1();
    let expected = expected_corpus();

    assert_eq!(registered.len(), 23, "registered case count changed");
    assert_eq!(expected.len(), 23, "independent oracle table is incomplete");
    assert_eq!(
        registered
            .iter()
            .map(|case| case.name())
            .collect::<Vec<_>>(),
        EXPECTED_NAMES,
        "registered case names or order changed"
    );

    for (ordinal, (actual, literal)) in registered.iter().zip(&expected).enumerate() {
        assert_literal_case(ordinal, actual, literal);
    }
}

fn assert_literal_case(ordinal: usize, actual: &RegisteredLayoutCaseV1, literal: &ExpectedCaseV1) {
    assert_eq!(actual.name(), literal.name, "case {ordinal}: wrong name");
    assert_eq!(
        actual.viewport(),
        literal.viewport,
        "{}: wrong viewport",
        literal.name
    );
    assert_eq!(
        actual.nodes(),
        literal.nodes,
        "{}: wrong nodes",
        literal.name
    );
    assert_eq!(
        actual.expected_records(),
        literal.expected_records,
        "{}: wrong expected records",
        literal.name
    );
}
