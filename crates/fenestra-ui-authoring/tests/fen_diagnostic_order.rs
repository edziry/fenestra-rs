#[path = "support/fen_diagnostics/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{
    AnchorKindV1, AuthoringDiagnosticKindV1, AuthoringLimitsV1, FenSourceV1, compile_fen_v1,
};
use support::{
    BASE, ExpectedDiagnostic, GENEROUS_LIMITS, SOURCE, diagnostic_mismatch, replace_once,
};

#[test]
fn semantic_order_is_source_order_with_record_local_priority() {
    expect_compiles("forward references", BASE, GENEROUS_LIMITS);
    expect_compiles("emitter-only zero", BASE, limits_with_zero_generated_rust());

    let duplicate_with_invalid = replace_once(
        BASE,
        "    property scalar = 0: scalar_i32 = 0 invalidates [layout];",
        "    property repeated = 0: scalar_i32 = 0 invalidates [layout];\n    property repeated = 2: scalar_i32 = 2147483648 invalidates [layout];",
    );
    let duplicate_with_unknown = replace_once(
        BASE,
        "template cell = 2: boxy {}",
        "template leaf = 2: missing_component {}",
    );
    let unknown_with_invalid =
        replace_once(BASE, "set scalar = 1;", "set missing_initial = 2147483648;");
    let unknown_with_type = replace_once(BASE, "set scalar = 1;", "set missing_initial = true;");
    let literal_with_type = replace_once(
        BASE,
        "property scalar = 0: scalar_i32 = 0 invalidates",
        "property scalar = 0: bool = 2147483648 invalidates",
    );

    let invalid_before_duplicate = replace_once(
        BASE,
        "scalar_i32 = 0 invalidates",
        "scalar_i32 = 2147483648 invalidates",
    );
    let invalid_before_duplicate = replace_once(
        &invalid_before_duplicate,
        "template cell = 2: boxy {}",
        "template leaf = 2: boxy {}",
    );
    let invalid_before_unknown = replace_once(
        BASE,
        "scalar_i32 = 0 invalidates",
        "scalar_i32 = 2147483648 invalidates",
    );
    let invalid_before_unknown = replace_once(
        &invalid_before_unknown,
        "template root = 0: boxy {",
        "template root = 0: missing_component {",
    );

    let duplicate_before_invalid = replace_once(
        BASE,
        "    property scalar = 0: scalar_i32 = 0 invalidates [layout];",
        "    property repeated = 0: scalar_i32 = 0 invalidates [layout];\n    property repeated = 2: scalar_i32 = 1 invalidates [layout];",
    );
    let duplicate_before_invalid = replace_once(
        &duplicate_before_invalid,
        "rgba8(10, 20, 30, 255)",
        "2147483648",
    );
    let unknown_before_invalid = replace_once(
        BASE,
        "template root = 0: boxy {",
        "template root = 0: missing_component {",
    );
    let unknown_before_invalid = replace_once(
        &unknown_before_invalid,
        "rgba8(10, 20, 30, 255)",
        "2147483648",
    );

    let child_before_set = replace_once(
        BASE,
        "  template root = 0: boxy {\n    set scalar = 1;\n    child template leaf;\n    child region rows;\n  }",
        "  template root = 0: boxy {\n    child template missing_child;\n    set missing_set = 1;\n    child region rows;\n  }",
    );
    let set_before_child = replace_once(
        BASE,
        "  template root = 0: boxy {\n    set scalar = 1;\n    child template leaf;\n    child region rows;\n  }",
        "  template root = 0: boxy {\n    set missing_set = 1;\n    child template missing_child;\n    child region rows;\n  }",
    );

    let cases = [
        Case::new(
            "duplicate before its invalid literal",
            duplicate_with_invalid,
            expected(
                AuthoringDiagnosticKindV1::DuplicatePropertyName,
                AnchorKindV1::Property,
                4,
                "repeated",
                1,
            ),
        ),
        Case::new(
            "duplicate before its unknown reference",
            duplicate_with_unknown,
            expected(
                AuthoringDiagnosticKindV1::DuplicateTemplateName,
                AnchorKindV1::Template,
                11,
                "leaf",
                2,
            ),
        ),
        Case::new(
            "unknown reference before its invalid literal",
            unknown_with_invalid,
            expected(
                AuthoringDiagnosticKindV1::UnknownPropertyName,
                AnchorKindV1::InitialProperty,
                7,
                "missing_initial",
                0,
            ),
        ),
        Case::new(
            "unknown reference before its type",
            unknown_with_type,
            expected(
                AuthoringDiagnosticKindV1::UnknownPropertyName,
                AnchorKindV1::InitialProperty,
                7,
                "missing_initial",
                0,
            ),
        ),
        Case::new(
            "invalid literal before its type",
            literal_with_type,
            expected(
                AuthoringDiagnosticKindV1::InvalidLiteral,
                AnchorKindV1::Property,
                3,
                "2147483648",
                0,
            ),
        ),
        Case::new(
            "earlier invalid literal before later duplicate",
            invalid_before_duplicate,
            expected(
                AuthoringDiagnosticKindV1::InvalidLiteral,
                AnchorKindV1::Property,
                3,
                "2147483648",
                0,
            ),
        ),
        Case::new(
            "earlier invalid literal before later unknown",
            invalid_before_unknown,
            expected(
                AuthoringDiagnosticKindV1::InvalidLiteral,
                AnchorKindV1::Property,
                3,
                "2147483648",
                0,
            ),
        ),
        Case::new(
            "earlier duplicate before later invalid literal",
            duplicate_before_invalid,
            expected(
                AuthoringDiagnosticKindV1::DuplicatePropertyName,
                AnchorKindV1::Property,
                4,
                "repeated",
                1,
            ),
        ),
        Case::new(
            "earlier unknown before later invalid literal",
            unknown_before_invalid,
            expected(
                AuthoringDiagnosticKindV1::UnknownComponentName,
                AnchorKindV1::Template,
                6,
                "missing_component",
                0,
            ),
        ),
        Case::new(
            "interleaved child before set",
            child_before_set,
            expected(
                AuthoringDiagnosticKindV1::UnknownTemplateName,
                AnchorKindV1::StaticChild,
                7,
                "missing_child",
                0,
            ),
        ),
        Case::new(
            "interleaved set before child",
            set_before_child,
            expected(
                AuthoringDiagnosticKindV1::UnknownPropertyName,
                AnchorKindV1::InitialProperty,
                7,
                "missing_set",
                0,
            ),
        ),
    ];

    let mismatches = cases
        .into_iter()
        .filter_map(|case| {
            diagnostic_mismatch(&case.source, case.expected)
                .map(|mismatch| format!("{}: {mismatch}", case.name))
        })
        .collect::<Vec<_>>();
    assert!(
        mismatches.is_empty(),
        "semantic-order contract mismatches:\n{}",
        mismatches.join("\n")
    );
}

struct Case {
    name: &'static str,
    source: String,
    expected: ExpectedDiagnostic,
}

impl Case {
    fn new(name: &'static str, source: String, expected: ExpectedDiagnostic) -> Self {
        Self {
            name,
            source,
            expected,
        }
    }
}

const fn expected(
    kind: AuthoringDiagnosticKindV1,
    anchor_kind: AnchorKindV1,
    ordinal: u32,
    culprit: &'static str,
    occurrence: usize,
) -> ExpectedDiagnostic {
    ExpectedDiagnostic {
        kind,
        anchor_kind,
        ordinal,
        culprit,
        occurrence,
    }
}

fn expect_compiles(name: &str, source: &str, limits: AuthoringLimitsV1) {
    compile_fen_v1(FenSourceV1::new(SOURCE, source.as_bytes()), limits)
        .unwrap_or_else(|error| panic!("{name} should compile: {error:?}"));
}

const fn limits_with_zero_generated_rust() -> AuthoringLimitsV1 {
    AuthoringLimitsV1::new(
        16_384, 4_096, 64, 16, 32, 32, 32, 32, 64, 64, 64, 64, 128, 0,
    )
}
