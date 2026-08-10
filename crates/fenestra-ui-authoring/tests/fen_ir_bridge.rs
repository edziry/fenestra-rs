#[path = "support/fen_diagnostics/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{AnchorKindV1, AuthoringDiagnosticKindV1};
use fenestra_ui_ir::prototype::IrValidationErrorKind;
use support::{BASE, ExpectedDiagnostic, diagnostic_mismatch, replace_once};

#[test]
fn numeric_collisions_reach_exact_ir_kinds_and_validation_phase_order() {
    let component_collision = replace_once(
        BASE,
        "  }\n}\nconstruction {",
        "  }\n  component other = 0 {\n    property alternate = 0: scalar_i32 = 0 invalidates [layout];\n  }\n}\nconstruction {",
    );
    let property_collision = replace_once(BASE, "property shade = 1:", "property shade = 0:");
    let template_collision = replace_once(BASE, "template leaf = 1:", "template leaf = 0:");
    let region_collision = replace_once(
        BASE,
        "  region rows = 0 owner root repeat cell keys [10] invalidates [structure];\n}",
        "  region rows = 0 owner root repeat cell keys [10] invalidates [structure];\n  region other = 0 owner root repeat cell keys [] invalidates [structure];\n}",
    );
    let key_collision = replace_once(BASE, "keys [10]", "keys [999, 999]");
    let initial_collision = replace_once(
        BASE,
        "    set scalar = 1;",
        "    set scalar = 1;\n    set scalar = 2;",
    );
    let style_collision = replace_once(
        BASE,
        "  set leaf.shade = rgba8(10, 20, 30, 255);",
        "  set leaf.shade = rgba8(10, 20, 30, 255);\n  set leaf.shade = rgba8(40, 50, 60, 255);",
    );
    let extra_root = replace_once(
        BASE,
        "  template cell = 2: boxy {}\n  region rows",
        "  template cell = 2: boxy {}\n  template orphan = 3: boxy {}\n  region rows",
    );

    let construction_then_style =
        replace_once(&style_collision, "template leaf = 1:", "template leaf = 0:");
    let schema_then_construction_then_style = replace_once(
        &construction_then_style,
        "property shade = 1:",
        "property shade = 0:",
    );

    let cases = [
        Case::new(
            "component id collision",
            component_collision,
            ir_expected(
                IrValidationErrorKind::DuplicateComponent,
                AnchorKindV1::Component,
                5,
                "other",
                0,
            ),
        ),
        Case::new(
            "property id collision",
            property_collision,
            ir_expected(
                IrValidationErrorKind::DuplicateProperty,
                AnchorKindV1::Property,
                4,
                "shade",
                0,
            ),
        ),
        Case::new(
            "template id collision",
            template_collision,
            ir_expected(
                IrValidationErrorKind::DuplicateNode,
                AnchorKindV1::Template,
                10,
                "leaf",
                1,
            ),
        ),
        Case::new(
            "region id collision",
            region_collision,
            ir_expected(
                IrValidationErrorKind::DuplicateRegion,
                AnchorKindV1::Region,
                14,
                "other",
                0,
            ),
        ),
        Case::new(
            "region key collision",
            key_collision,
            ir_expected(
                IrValidationErrorKind::DuplicateRegionKey,
                AnchorKindV1::InitialKey,
                14,
                "999",
                1,
            ),
        ),
        Case::new(
            "resolved initial property collision",
            initial_collision,
            ir_expected(
                IrValidationErrorKind::DuplicateInitialProperty,
                AnchorKindV1::InitialProperty,
                8,
                "scalar",
                3,
            ),
        ),
        Case::new(
            "resolved style assignment collision",
            style_collision.clone(),
            ir_expected(
                IrValidationErrorKind::DuplicateStyleAssignment,
                AnchorKindV1::StyleAssignment,
                16,
                "shade",
                2,
            ),
        ),
        Case::new(
            "construction header maps without Document fallback",
            extra_root,
            ir_expected(
                IrValidationErrorKind::InvalidRootCount,
                AnchorKindV1::Construction,
                5,
                "construction",
                0,
            ),
        ),
        Case::new(
            "schema precedes construction and style",
            schema_then_construction_then_style,
            ir_expected(
                IrValidationErrorKind::DuplicateProperty,
                AnchorKindV1::Property,
                4,
                "shade",
                0,
            ),
        ),
        Case::new(
            "construction precedes style",
            construction_then_style,
            ir_expected(
                IrValidationErrorKind::DuplicateNode,
                AnchorKindV1::Template,
                10,
                "leaf",
                1,
            ),
        ),
        Case::new(
            "style is the final IR phase",
            style_collision,
            ir_expected(
                IrValidationErrorKind::DuplicateStyleAssignment,
                AnchorKindV1::StyleAssignment,
                16,
                "shade",
                2,
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
        "IR-bridge contract mismatches:\n{}",
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

const fn ir_expected(
    kind: IrValidationErrorKind,
    anchor_kind: AnchorKindV1,
    ordinal: u32,
    culprit: &'static str,
    occurrence: usize,
) -> ExpectedDiagnostic {
    ExpectedDiagnostic {
        kind: AuthoringDiagnosticKindV1::IrValidation(kind),
        anchor_kind,
        ordinal,
        culprit,
        occurrence,
    }
}
