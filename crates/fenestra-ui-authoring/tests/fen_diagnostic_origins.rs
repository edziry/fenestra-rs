#[path = "support/fen_diagnostics/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{AnchorKindV1, AuthoringDiagnosticKindV1};
use support::{BASE, ExpectedDiagnostic, diagnostic_mismatch, replace_once};

#[test]
fn semantic_diagnostics_point_to_the_exact_culpable_token() {
    let cases = [
        case(
            "template component",
            "template leaf = 1: boxy {}",
            "template leaf = 1: missing_component {}",
            expected(
                AuthoringDiagnosticKindV1::UnknownComponentName,
                AnchorKindV1::Template,
                10,
                "missing_component",
            ),
        ),
        case(
            "initial property",
            "set scalar = 1;",
            "set missing_initial = 1;",
            expected(
                AuthoringDiagnosticKindV1::UnknownPropertyName,
                AnchorKindV1::InitialProperty,
                7,
                "missing_initial",
            ),
        ),
        case(
            "initial value",
            "set scalar = 1;",
            "set scalar = true;",
            expected(
                AuthoringDiagnosticKindV1::ValueTypeMismatch,
                AnchorKindV1::InitialProperty,
                7,
                "true",
            ),
        ),
        case(
            "static child",
            "child template leaf;",
            "child template missing_static;",
            expected(
                AuthoringDiagnosticKindV1::UnknownTemplateName,
                AnchorKindV1::StaticChild,
                8,
                "missing_static",
            ),
        ),
        case(
            "region child",
            "child region rows;",
            "child region missing_region;",
            expected(
                AuthoringDiagnosticKindV1::UnknownRegionName,
                AnchorKindV1::RegionChild,
                9,
                "missing_region",
            ),
        ),
        case(
            "region owner",
            "owner root",
            "owner missing_owner",
            expected(
                AuthoringDiagnosticKindV1::UnknownTemplateName,
                AnchorKindV1::Region,
                12,
                "missing_owner",
            ),
        ),
        case(
            "region repeat",
            "repeat cell",
            "repeat missing_repeat",
            expected(
                AuthoringDiagnosticKindV1::UnknownTemplateName,
                AnchorKindV1::Region,
                12,
                "missing_repeat",
            ),
        ),
        case(
            "style target",
            "set leaf.shade",
            "set missing_target.shade",
            expected(
                AuthoringDiagnosticKindV1::UnknownTemplateName,
                AnchorKindV1::StyleAssignment,
                15,
                "missing_target",
            ),
        ),
        case(
            "style property",
            "set leaf.shade",
            "set leaf.missing_style_property",
            expected(
                AuthoringDiagnosticKindV1::UnknownPropertyName,
                AnchorKindV1::StyleAssignment,
                15,
                "missing_style_property",
            ),
        ),
        case(
            "style value",
            "rgba8(10, 20, 30, 255)",
            "true",
            expected(
                AuthoringDiagnosticKindV1::ValueTypeMismatch,
                AnchorKindV1::StyleAssignment,
                15,
                "true",
            ),
        ),
        case(
            "property default",
            "scalar_i32 = 0 invalidates",
            "scalar_i32 = true invalidates",
            expected(
                AuthoringDiagnosticKindV1::ValueTypeMismatch,
                AnchorKindV1::Property,
                3,
                "true",
            ),
        ),
        case(
            "invalid scalar literal",
            "scalar_i32 = 0 invalidates",
            "scalar_i32 = 2147483648 invalidates",
            expected(
                AuthoringDiagnosticKindV1::InvalidLiteral,
                AnchorKindV1::Property,
                3,
                "2147483648",
            ),
        ),
    ];

    let mismatches = cases
        .into_iter()
        .filter_map(|case| {
            let source = replace_once(BASE, case.before, case.after);
            diagnostic_mismatch(&source, case.expected)
                .map(|mismatch| format!("{}: {mismatch}", case.name))
        })
        .collect::<Vec<_>>();
    assert!(
        mismatches.is_empty(),
        "culpable-token contract mismatches:\n{}",
        mismatches.join("\n")
    );
}

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    before: &'static str,
    after: &'static str,
    expected: ExpectedDiagnostic,
}

const fn case(
    name: &'static str,
    before: &'static str,
    after: &'static str,
    expected: ExpectedDiagnostic,
) -> Case {
    Case {
        name,
        before,
        after,
        expected,
    }
}

const fn expected(
    kind: AuthoringDiagnosticKindV1,
    anchor_kind: AnchorKindV1,
    ordinal: u32,
    culprit: &'static str,
) -> ExpectedDiagnostic {
    ExpectedDiagnostic {
        kind,
        anchor_kind,
        ordinal,
        culprit,
        occurrence: 0,
    }
}
