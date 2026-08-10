#[path = "support/fen_diagnostics/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{
    AnchorKindV1, AuthoringDiagnosticKindV1, FenSourceV1, compile_fen_v1,
};
use fenestra_ui_ir::prototype::{IrValidationErrorKind, ValidationLimitKind};
use support::{
    BASE, ExpectedDiagnostic, GENEROUS_LIMITS, SOURCE, diagnostic_mismatch, replace_once,
};

const DEPTH_THREE: &str = "format 1;
schema namespace 1 revision 1 {
  component c = 0 {
    property p = 0: scalar_i32 = 0 invalidates [layout];
  }
}
construction {
  template root = 0: c {
    child template mid;
    child region rows;
  }
  template mid = 1: c {
    child template leaf;
  }
  template leaf = 2: c {}
  template cell = 3: c {}
  region rows = 0 owner root repeat cell keys [] invalidates [structure];
}
style {}
";

const DEPTH_FOUR: &str = "format 1;
schema namespace 1 revision 1 {
  component c = 0 {
    property p = 0: scalar_i32 = 0 invalidates [layout];
  }
}
construction {
  template root = 0: c {
    child template mid;
    child region rows;
  }
  template mid = 1: c {
    child template leaf;
  }
  template leaf = 2: c {
    child template deep;
  }
  template deep = 3: c {}
  template cell = 4: c {}
  region rows = 0 owner root repeat cell keys [] invalidates [structure];
}
style {}
";

#[test]
fn registered_depth_and_initial_instance_limits_are_inclusive() {
    expect_compiles("template depth three", DEPTH_THREE);
    let depth_mismatch = diagnostic_mismatch(
        DEPTH_FOUR,
        ir_limit(
            ValidationLimitKind::TemplateDepth,
            AnchorKindV1::StaticChild,
            11,
            "deep",
            0,
        ),
    );

    let five_instances = replace_once(BASE, "keys [10]", "keys [10, 20, 30]");
    expect_compiles("five initial instances", &five_instances);
    let six_instances = replace_once(BASE, "keys [10]", "keys [10, 20, 30, 40]");
    let instance_mismatch = diagnostic_mismatch(
        &six_instances,
        ir_limit(
            ValidationLimitKind::InitialInstances,
            AnchorKindV1::Region,
            12,
            "rows",
            1,
        ),
    );

    let mismatches = [
        ("depth four", depth_mismatch),
        ("six initial instances", instance_mismatch),
    ]
    .into_iter()
    .filter_map(|(name, mismatch)| mismatch.map(|mismatch| format!("{name}: {mismatch}")))
    .collect::<Vec<_>>();
    assert!(
        mismatches.is_empty(),
        "registered IR-limit mismatches:\n{}",
        mismatches.join("\n")
    );
}

fn expect_compiles(name: &str, source: &str) {
    compile_fen_v1(FenSourceV1::new(SOURCE, source.as_bytes()), GENEROUS_LIMITS)
        .unwrap_or_else(|error| panic!("{name} should compile: {error:?}"));
}

const fn ir_limit(
    limit: ValidationLimitKind,
    anchor_kind: AnchorKindV1,
    ordinal: u32,
    culprit: &'static str,
    occurrence: usize,
) -> ExpectedDiagnostic {
    ExpectedDiagnostic {
        kind: AuthoringDiagnosticKindV1::IrValidation(IrValidationErrorKind::LimitExceeded(limit)),
        anchor_kind,
        ordinal,
        culprit,
        occurrence,
    }
}
