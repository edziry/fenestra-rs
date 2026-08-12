use std::error::Error;

use fenestra_ui_authoring::prototype::{
    AnchorKindV1, AuthoringDiagnosticKindV1, AuthoringFrontendV1, AuthoringLimitKindV1,
    AuthoringLimitsV1, DiagnosticLocationV1, FenSourceV1, REFERENCE_AUTHORING_LIMITS_V1,
    SUPPORTED_AUTHORING_FORMAT, compile_fen_v1,
};
use fenestra_ui_ir::prototype::{IrValidationErrorKind, SourceId};

const REGISTERED_LIMITS: AuthoringLimitsV1 = REFERENCE_AUTHORING_LIMITS_V1;

#[test]
fn version_limits_and_closed_vocabularies_are_exact() {
    assert_eq!(SUPPORTED_AUTHORING_FORMAT.get(), 1);
    assert_eq!(
        AuthoringFrontendV1::ALL,
        [AuthoringFrontendV1::Fen, AuthoringFrontendV1::UiMacro]
    );
    assert_eq!(
        AnchorKindV1::ALL,
        [
            AnchorKindV1::Document,
            AnchorKindV1::Schema,
            AnchorKindV1::Component,
            AnchorKindV1::Property,
            AnchorKindV1::Construction,
            AnchorKindV1::Template,
            AnchorKindV1::InitialProperty,
            AnchorKindV1::StaticChild,
            AnchorKindV1::RegionChild,
            AnchorKindV1::Region,
            AnchorKindV1::InitialKey,
            AnchorKindV1::Style,
            AnchorKindV1::StyleAssignment,
        ]
    );

    let kinds = [
        AuthoringLimitKindV1::FenSourceBytes,
        AuthoringLimitKindV1::Tokens,
        AuthoringLimitKindV1::IdentifierBytes,
        AuthoringLimitKindV1::NestingDepth,
        AuthoringLimitKindV1::Components,
        AuthoringLimitKindV1::Properties,
        AuthoringLimitKindV1::Templates,
        AuthoringLimitKindV1::Regions,
        AuthoringLimitKindV1::ChildSlots,
        AuthoringLimitKindV1::InitialProperties,
        AuthoringLimitKindV1::InitialKeys,
        AuthoringLimitKindV1::StyleAssignments,
        AuthoringLimitKindV1::SourceAnchors,
        AuthoringLimitKindV1::GeneratedRustBytes,
    ];
    let values = [8_192, 1_024, 32, 8, 1, 5, 4, 1, 3, 12, 2, 2, 34, 32_768];
    assert_eq!(AuthoringLimitKindV1::ALL, kinds);
    for (kind, expected) in kinds.into_iter().zip(values) {
        assert_eq!(REGISTERED_LIMITS.limit(kind), expected);
    }

    let mut diagnostics = vec![
        AuthoringDiagnosticKindV1::InvalidUtf8,
        AuthoringDiagnosticKindV1::UnsupportedToken,
        AuthoringDiagnosticKindV1::UnsupportedAuthoringFormat,
        AuthoringDiagnosticKindV1::UnexpectedToken,
        AuthoringDiagnosticKindV1::UnexpectedEof,
        AuthoringDiagnosticKindV1::InvalidIdentifier,
        AuthoringDiagnosticKindV1::InvalidLiteral,
        AuthoringDiagnosticKindV1::DuplicateComponentName,
        AuthoringDiagnosticKindV1::DuplicatePropertyName,
        AuthoringDiagnosticKindV1::DuplicateTemplateName,
        AuthoringDiagnosticKindV1::DuplicateRegionName,
        AuthoringDiagnosticKindV1::UnknownComponentName,
        AuthoringDiagnosticKindV1::UnknownPropertyName,
        AuthoringDiagnosticKindV1::UnknownTemplateName,
        AuthoringDiagnosticKindV1::UnknownRegionName,
        AuthoringDiagnosticKindV1::ValueTypeMismatch,
    ];
    diagnostics.extend(kinds.map(AuthoringDiagnosticKindV1::LimitExceeded));
    diagnostics.extend(IrValidationErrorKind::ALL.map(AuthoringDiagnosticKindV1::IrValidation));
    assert_eq!(AuthoringDiagnosticKindV1::ALL.len(), 110);
    assert_eq!(AuthoringDiagnosticKindV1::ALL.as_slice(), diagnostics);
}

#[test]
fn fen_preflight_is_bounded_physical_and_privacy_safe() {
    let invalid_utf8 = [0xff];
    let source = FenSourceV1::new(SourceId::new(7), &invalid_utf8);

    let bytes_first = compile_fen_v1(source, limits_with_source_bytes(0))
        .expect_err("source bytes must precede UTF-8 validation");
    assert_eq!(
        bytes_first.kind(),
        AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::FenSourceBytes)
    );
    assert_fen_range(&bytes_first, 7, 0, 1);

    let utf8 = compile_fen_v1(source, limits_with_source_bytes(1))
        .expect_err("invalid UTF-8 must fail before semantic anchors exist");
    assert_eq!(utf8.frontend(), AuthoringFrontendV1::Fen);
    assert_eq!(utf8.kind(), AuthoringDiagnosticKindV1::InvalidUtf8);
    assert_fen_range(&utf8, 7, 0, 1);

    let empty = compile_fen_v1(
        FenSourceV1::new(SourceId::new(7), b""),
        limits_with_source_bytes(0),
    )
    .expect_err("empty format-1 input must be an EOF diagnostic");
    assert_eq!(empty.kind(), AuthoringDiagnosticKindV1::UnexpectedEof);
    assert_fen_range(&empty, 7, 0, 0);

    let _: &dyn Error = &utf8;
    let rendered = format!("{utf8:?} {utf8}");
    assert!(rendered.contains("invalid-utf8"));
    for forbidden in ["0xff", "SourceId(7)", "/home/", "FenSourceV1"] {
        assert!(
            !rendered.contains(forbidden),
            "leaked {forbidden}: {rendered}"
        );
    }
}

fn limits_with_source_bytes(fen_source_bytes: usize) -> AuthoringLimitsV1 {
    AuthoringLimitsV1::new(
        fen_source_bytes,
        1_024,
        32,
        8,
        1,
        5,
        4,
        1,
        3,
        12,
        2,
        2,
        34,
        32_768,
    )
}

fn assert_fen_range(
    diagnostic: &fenestra_ui_authoring::prototype::AuthoringDiagnosticV1,
    source: u32,
    start: u32,
    end: u32,
) {
    match diagnostic.location() {
        DiagnosticLocationV1::Physical(origin) => {
            assert_eq!(origin.source_id(), Some(SourceId::new(source)));
            assert_eq!(origin.fen_byte_range(), Some((start, end)));
        }
        DiagnosticLocationV1::Anchored { .. } => {
            panic!("pre-anchor .fen failure must use physical byte coordinates")
        }
    }
}
