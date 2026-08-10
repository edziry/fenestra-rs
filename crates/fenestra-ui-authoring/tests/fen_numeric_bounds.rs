#[path = "support/fen_diagnostics/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{
    AnchorKindV1, AuthoringDiagnosticKindV1, FenSourceV1, compile_fen_v1,
};
use support::{
    BASE, ExpectedDiagnostic, GENEROUS_LIMITS, SOURCE, diagnostic_mismatch, replace_once,
};

#[test]
fn numeric_domains_accept_endpoints_and_reject_the_first_value_outside_them() {
    let u32_fields = [
        (
            "revision",
            "revision 1 {",
            "revision 4294967295 {",
            "revision 4294967296 {",
            AnchorKindV1::Schema,
            1,
        ),
        (
            "component id",
            "component boxy = 0 {",
            "component boxy = 4294967295 {",
            "component boxy = 4294967296 {",
            AnchorKindV1::Component,
            2,
        ),
        (
            "property id",
            "property scalar = 0:",
            "property scalar = 4294967295:",
            "property scalar = 4294967296:",
            AnchorKindV1::Property,
            3,
        ),
        (
            "template id",
            "template root = 0:",
            "template root = 4294967295:",
            "template root = 4294967296:",
            AnchorKindV1::Template,
            6,
        ),
        (
            "region id",
            "region rows = 0 owner",
            "region rows = 4294967295 owner",
            "region rows = 4294967296 owner",
            AnchorKindV1::Region,
            12,
        ),
    ];
    let mut mismatches = Vec::new();
    for (name, before, at_maximum, one_over, anchor_kind, ordinal) in u32_fields {
        expect_compiles(name, &replace_once(BASE, before, at_maximum));
        let source = replace_once(BASE, before, one_over);
        record_mismatch(
            &mut mismatches,
            name,
            &source,
            invalid(anchor_kind, ordinal, "4294967296"),
        );
    }

    let namespace_max = replace_once(
        BASE,
        "namespace 77 revision",
        "namespace 18446744073709551615 revision",
    );
    expect_compiles("namespace u64 maximum", &namespace_max);
    let namespace_over = replace_once(
        BASE,
        "namespace 77 revision",
        "namespace 18446744073709551616 revision",
    );
    record_mismatch(
        &mut mismatches,
        "namespace u64 one over",
        &namespace_over,
        invalid(AnchorKindV1::Schema, 1, "18446744073709551616"),
    );

    let key_max = replace_once(BASE, "keys [10]", "keys [18446744073709551615]");
    expect_compiles("key u64 maximum", &key_max);
    let key_over = replace_once(BASE, "keys [10]", "keys [18446744073709551616]");
    record_mismatch(
        &mut mismatches,
        "key u64 one over",
        &key_over,
        invalid(AnchorKindV1::InitialKey, 13, "18446744073709551616"),
    );

    for (name, value) in [
        ("i32 minimum", "-2147483648"),
        ("i32 maximum", "2147483647"),
    ] {
        let source = replace_once(
            BASE,
            "scalar_i32 = 0 invalidates",
            &format!("scalar_i32 = {value} invalidates"),
        );
        expect_compiles(name, &source);
    }
    for (name, value, culprit) in [
        ("i32 below minimum", "-2147483649", "2147483649"),
        ("i32 above maximum", "2147483648", "2147483648"),
    ] {
        let source = replace_once(
            BASE,
            "scalar_i32 = 0 invalidates",
            &format!("scalar_i32 = {value} invalidates"),
        );
        record_mismatch(
            &mut mismatches,
            name,
            &source,
            invalid(AnchorKindV1::Property, 3, culprit),
        );
    }

    let rgba_edges = replace_once(
        BASE,
        "rgba8(1, 2, 3, 255) invalidates",
        "rgba8(0, 255, 0, 255) invalidates",
    );
    expect_compiles("rgba8 endpoints", &rgba_edges);
    let rgba_over = replace_once(
        BASE,
        "rgba8(1, 2, 3, 255) invalidates",
        "rgba8(0, 255, 256, 255) invalidates",
    );
    record_mismatch(
        &mut mismatches,
        "rgba8 channel one over",
        &rgba_over,
        invalid(AnchorKindV1::Property, 4, "256"),
    );

    assert!(
        mismatches.is_empty(),
        "numeric-bound contract mismatches:\n{}",
        mismatches.join("\n")
    );
}

fn expect_compiles(name: &str, source: &str) {
    compile_fen_v1(FenSourceV1::new(SOURCE, source.as_bytes()), GENEROUS_LIMITS)
        .unwrap_or_else(|error| panic!("{name} should compile: {error:?}"));
}

fn record_mismatch(
    mismatches: &mut Vec<String>,
    name: &str,
    source: &str,
    expected: ExpectedDiagnostic,
) {
    if let Some(mismatch) = diagnostic_mismatch(source, expected) {
        mismatches.push(format!("{name}: {mismatch}"));
    }
}

const fn invalid(
    anchor_kind: AnchorKindV1,
    ordinal: u32,
    culprit: &'static str,
) -> ExpectedDiagnostic {
    ExpectedDiagnostic {
        kind: AuthoringDiagnosticKindV1::InvalidLiteral,
        anchor_kind,
        ordinal,
        culprit,
        occurrence: 0,
    }
}
