use fenestra_ui_authoring::prototype::{
    AnchorKindV2, AuthoringDiagnosticKindV1, AuthoringDiagnosticKindV2, AuthoringLimitKindV2,
    AuthoringLimitsV1, AuthoringLimitsV2, FenSourceV1, FenSourceV2, compile_fen_v1, compile_fen_v2,
    compile_ui_v2,
};
use fenestra_ui_ir::prototype::{IrValidationErrorKind, SourceId};
use proc_macro2::TokenStream;

use super::support::{
    ExpectedDiagnostic, FIXTURE, GENEROUS_LIMITS, LIMIT_VALUES, SOURCE, assert_diagnostic,
    assert_diagnostic_with_limits, replace_once,
};

#[test]
fn duplicate_and_unknown_names_report_the_exact_spatial_field() {
    let cases = [
        (
            "duplicate node",
            replace_once(
                FIXTURE,
                "node guide : anchor_ref",
                "node scene : anchor_ref",
            ),
            expected(
                AuthoringDiagnosticKindV2::DuplicateSpatialNodeName,
                AnchorKindV2::SpatialField,
                331,
                "scene",
                8,
            ),
        ),
        (
            "duplicate image",
            duplicate_image(FIXTURE),
            expected(
                AuthoringDiagnosticKindV2::DuplicateSpatialImageName,
                AnchorKindV2::SpatialField,
                65,
                "checker",
                1,
            ),
        ),
        (
            "duplicate shape",
            replace_once(FIXTURE, "shape dot circle", "shape frame circle"),
            expected(
                AuthoringDiagnosticKindV2::DuplicateSpatialShapeName,
                AnchorKindV2::SpatialField,
                96,
                "frame",
                1,
            ),
        ),
        (
            "duplicate brush",
            replace_once(
                FIXTURE,
                "brush fade linear_gradient",
                "brush flat linear_gradient",
            ),
            expected(
                AuthoringDiagnosticKindV2::DuplicateSpatialBrushName,
                AnchorKindV2::SpatialField,
                136,
                "flat",
                1,
            ),
        ),
        (
            "duplicate clip",
            replace_once(FIXTURE, "clip inner {", "clip outer {"),
            expected(
                AuthoringDiagnosticKindV2::DuplicateSpatialClipName,
                AnchorKindV2::SpatialField,
                154,
                "outer",
                1,
            ),
        ),
        (
            "unknown forward node",
            replace_once(FIXTURE, "target node guide", "target node missing_node"),
            expected(
                AuthoringDiagnosticKindV2::UnknownSpatialNodeName,
                AnchorKindV2::SpatialField,
                239,
                "missing_node",
                0,
            ),
        ),
        (
            "unknown image",
            replace_once(FIXTURE, "image checker;", "image missing_image;"),
            expected(
                AuthoringDiagnosticKindV2::UnknownSpatialImageName,
                AnchorKindV2::SpatialField,
                170,
                "missing_image",
                0,
            ),
        ),
        (
            "unknown shape",
            replace_once(FIXTURE, "shape dot;", "shape missing_shape;"),
            expected(
                AuthoringDiagnosticKindV2::UnknownSpatialShapeName,
                AnchorKindV2::SpatialField,
                195,
                "missing_shape",
                0,
            ),
        ),
        (
            "unknown brush",
            replace_once(FIXTURE, "brush fade;", "brush missing_brush;"),
            expected(
                AuthoringDiagnosticKindV2::UnknownSpatialBrushName,
                AnchorKindV2::SpatialField,
                167,
                "missing_brush",
                0,
            ),
        ),
        (
            "unknown clip owner",
            replace_once(
                FIXTURE,
                "opacity 255;\n      clip scene.inner;",
                "opacity 255;\n      clip missing_owner.inner;",
            ),
            expected(
                AuthoringDiagnosticKindV2::UnknownSpatialNodeName,
                AnchorKindV2::SpatialField,
                162,
                "missing_owner",
                0,
            ),
        ),
        (
            "unknown clip",
            replace_once(
                FIXTURE,
                "opacity 255;\n      clip scene.inner;",
                "opacity 255;\n      clip scene.missing_clip;",
            ),
            expected(
                AuthoringDiagnosticKindV2::UnknownSpatialClipName,
                AnchorKindV2::SpatialField,
                163,
                "missing_clip",
                0,
            ),
        ),
    ];
    for (name, source, diagnostic) in cases {
        let assertion = || {
            if name == "duplicate image" {
                let mut values = LIMIT_VALUES;
                values[AuthoringLimitKindV2::Images as usize] = 2;
                values[AuthoringLimitKindV2::ImageBytes as usize] = 32;
                values[AuthoringLimitKindV2::SpatialFields as usize] = 268;
                assert_diagnostic_with_limits(&source, AuthoringLimitsV2::new(values), diagnostic);
            } else {
                assert_diagnostic(&source, diagnostic);
            }
        };
        std::panic::catch_unwind(assertion)
            .unwrap_or_else(|_| panic!("diagnostic case failed: {name}"));
    }
}

#[test]
fn free_placement_reports_fields_before_its_later_anchor_target() {
    let before = concat!(
        "placement free\n",
        "          width property span_x height property span_y\n",
        "          self_anchor anchor(center, end)\n",
        "          target node guide",
    );
    let after = concat!(
        "placement free\n",
        "          width property missing_width height property span_y\n",
        "          self_anchor anchor(center, end)\n",
        "          target node missing_target",
    );
    let source = replace_once(FIXTURE, before, after);
    assert_diagnostic(
        &source,
        expected(
            AuthoringDiagnosticKindV2::UnknownPropertyName,
            AnchorKindV2::SpatialField,
            237,
            "missing_width",
            0,
        ),
    );
}

#[test]
fn primitive_numeric_bounds_and_special_attribution_are_exact() {
    let valid = [
        (
            "i32 minimum",
            "viewport container row padding (4, 4, 3, 3)",
            "viewport container row padding (-2147483648, 4, 3, 3)",
        ),
        (
            "i32 maximum",
            "viewport container row padding (4, 4, 3, 3)",
            "viewport container row padding (2147483647, 4, 3, 3)",
        ),
        (
            "i64 minimum",
            "offset point(fixed(-65536), property factor)",
            "offset point(fixed(-9223372036854775808), property factor)",
        ),
        (
            "i64 maximum",
            "offset point(fixed(-65536), property factor)",
            "offset point(fixed(9223372036854775807), property factor)",
        ),
        ("u8 maximum", "opacity 200;", "opacity 255;"),
        ("u16 maximum", "stop 32768", "stop 65535"),
        ("u32 maximum", "stride 8;", "stride 4294967295;"),
        (
            "u64 maximum",
            "keys [10, 20]",
            "keys [10, 18446744073709551615]",
        ),
    ];
    for (name, before, after) in valid {
        let source = replace_once(FIXTURE, before, after);
        let result = compile_fen_v2(FenSourceV2::new(SOURCE, source.as_bytes()), GENEROUS_LIMITS);
        if name.starts_with("i64") {
            assert_eq!(
                result
                    .expect_err("primitive i64 endpoint is outside canonical Fixed16")
                    .kind(),
                AuthoringDiagnosticKindV2::IrValidation(
                    IrValidationErrorKind::SpatialFixed16OutOfRange,
                )
            );
        } else {
            result.unwrap_or_else(|error| panic!("{name} should compile: {error:?}"));
        }
    }

    let invalid = [
        (
            "i32 low",
            "padding (4, 4, 3, 3)",
            "padding (-2147483649, 4, 3, 3)",
            AnchorKindV2::SpatialField,
            53,
            "-2147483649",
            0,
        ),
        (
            "i32 high",
            "padding (4, 4, 3, 3)",
            "padding (2147483648, 4, 3, 3)",
            AnchorKindV2::SpatialField,
            53,
            "2147483648",
            0,
        ),
        (
            "i64 low",
            "offset point(fixed(-65536), property factor)",
            "offset point(fixed(-9223372036854775809), property factor)",
            AnchorKindV2::SpatialField,
            240,
            "-9223372036854775809",
            0,
        ),
        (
            "i64 high",
            "offset point(fixed(-65536), property factor)",
            "offset point(fixed(9223372036854775808), property factor)",
            AnchorKindV2::SpatialField,
            240,
            "9223372036854775808",
            0,
        ),
        (
            "u8 high",
            "opacity 200;",
            "opacity 256;",
            AnchorKindV2::SpatialField,
            168,
            "256",
            0,
        ),
        (
            "u16 high",
            "stop 32768",
            "stop 65536",
            AnchorKindV2::SpatialField,
            145,
            "65536",
            6,
        ),
        (
            "u32 high",
            "stride 8;",
            "stride 4294967296;",
            AnchorKindV2::SpatialField,
            63,
            "4294967296",
            0,
        ),
        (
            "u64 high",
            "keys [10, 20]",
            "keys [10, 18446744073709551616]",
            AnchorKindV2::InitialKey,
            46,
            "18446744073709551616",
            0,
        ),
    ];
    for (name, before, after, anchor_kind, ordinal, culprit, occurrence) in invalid {
        let source = replace_once(FIXTURE, before, after);
        std::panic::catch_unwind(|| {
            assert_diagnostic(
                &source,
                expected(
                    AuthoringDiagnosticKindV2::InvalidLiteral,
                    anchor_kind,
                    ordinal,
                    culprit,
                    occurrence,
                ),
            );
        })
        .unwrap_or_else(|_| panic!("numeric case failed: {name}"));
    }

    let invalid_turn = replace_once(FIXTURE, "quarter_turn(1)", "quarter_turn(4)");
    assert_diagnostic(
        &invalid_turn,
        expected(
            AuthoringDiagnosticKindV2::InvalidLiteral,
            AnchorKindV2::SpatialTransform,
            266,
            "4",
            24,
        ),
    );

    let invalid_byte = replace_once(FIXTURE, "bytes [255, 0, 0, 255", "bytes [256, 0, 0, 255");
    assert_diagnostic(
        &invalid_byte,
        expected(
            AuthoringDiagnosticKindV2::InvalidLiteral,
            AnchorKindV2::Image,
            59,
            "256",
            0,
        ),
    );
}

#[test]
fn malformed_headers_are_not_trial_dispatched_and_v1_behavior_is_unchanged() {
    for source in ["", "format", "format 2", "format 2 schema", "format 3;"] {
        let tokens = source
            .parse::<TokenStream>()
            .expect("header should tokenize");
        let error = compile_ui_v2(tokens, GENEROUS_LIMITS)
            .expect_err("direct V2 compilation should diagnose the complete stream");
        let expected = match source {
            "format 3;" => AuthoringDiagnosticKindV2::UnsupportedAuthoringFormat,
            "format 2 schema" => AuthoringDiagnosticKindV2::UnexpectedToken,
            _ => AuthoringDiagnosticKindV2::UnexpectedEof,
        };
        assert_eq!(error.kind(), expected, "source `{source}`");
    }

    let v1_limits = AuthoringLimitsV1::new(1_024, 128, 32, 8, 1, 1, 1, 1, 0, 0, 0, 0, 8, 4_096);
    let v1_error = compile_fen_v1(FenSourceV1::new(SourceId::new(1), b"format 2;"), v1_limits)
        .expect_err("V1 should continue rejecting format 2 itself");
    assert_eq!(
        v1_error.kind(),
        AuthoringDiagnosticKindV1::UnsupportedAuthoringFormat
    );
}

fn duplicate_image(source: &str) -> String {
    const IMAGE: &str = "    image checker {
      width 2;
      height 2;
      stride 8;
      bytes [255, 0, 0, 255, 0, 128, 0, 128, 0, 0, 64, 64, 0, 0, 0, 0];
    }
";
    let insertion = format!("{IMAGE}{IMAGE}");
    replace_once(source, IMAGE, &insertion)
}

const fn expected(
    kind: AuthoringDiagnosticKindV2,
    anchor_kind: AnchorKindV2,
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
