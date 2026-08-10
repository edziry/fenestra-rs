use std::panic::catch_unwind;

use fenestra_ui_ir::prototype::{
    IrValidationError, IrValidationErrorKind, SchemaFormatVersion, SchemaManifest, SchemaNamespace,
    SchemaRevision, SourceId, SourceSpan, ValidationLimits, validate_schema,
};

use crate::diagnostic::AuthoringDiagnosticKindV1;
use crate::fen::parse_fen_document_v1;
use crate::limits::AuthoringLimitsV1;
use crate::source::DiagnosticLocationV1;
use crate::vocabulary::AnchorKindV1;

use super::ir_failure;

const DOCUMENT: &str = "format 1;
schema namespace 1 revision 1 {
  component c = 0 {
    property p = 0: scalar_i32 = 0 invalidates [layout];
  }
}
construction {
  template root = 0: c {
    child region rows;
  }
  template cell = 1: c {}
  region rows = 0 owner root repeat cell keys [] invalidates [structure];
}
style {}
";

#[test]
fn valid_ir_span_maps_to_its_exact_non_document_anchor() {
    let parsed = parsed_document();
    let error = ir_error(SourceSpan::bytes(SourceId::new(0), 1, 2));
    let diagnostic = ir_failure(&parsed, error);
    assert_eq!(
        diagnostic.kind(),
        AuthoringDiagnosticKindV1::IrValidation(IrValidationErrorKind::UnsupportedSchemaFormat)
    );
    assert!(matches!(
        diagnostic.location(),
        DiagnosticLocationV1::Anchored {
            logical: SourceSpan::Bytes {
                source,
                start: 1,
                end: 2,
            },
            anchor_kind: AnchorKindV1::Schema,
            ..
        } if *source == SourceId::new(0)
    ));
}

#[test]
fn invalid_ir_span_mapping_is_an_internal_invariant() {
    let parsed = parsed_document();
    let cases = [
        ("synthetic", SourceSpan::synthetic()),
        ("foreign source", SourceSpan::bytes(SourceId::new(7), 1, 2)),
        ("zero width", SourceSpan::bytes(SourceId::new(0), 1, 1)),
        ("wide", SourceSpan::bytes(SourceId::new(0), 1, 3)),
        (
            "outside catalog",
            SourceSpan::bytes(SourceId::new(0), 99, 100),
        ),
        ("Document anchor", SourceSpan::bytes(SourceId::new(0), 0, 1)),
    ];
    let accepted = cases
        .into_iter()
        .filter_map(|(name, span)| {
            catch_unwind(|| ir_failure(&parsed, ir_error(span)))
                .is_ok()
                .then_some(name)
        })
        .collect::<Vec<_>>();
    assert!(
        accepted.is_empty(),
        "invalid IR spans became ordinary diagnostics: {}",
        accepted.join(", ")
    );
}

fn parsed_document() -> crate::parsed::ParsedDocumentV1 {
    parse_fen_document_v1(SourceId::new(17), DOCUMENT, limits())
        .unwrap_or_else(|error| panic!("unit fixture should parse: {error:?}"))
}

fn ir_error(span: SourceSpan) -> IrValidationError {
    let schema = SchemaManifest::new(
        SchemaFormatVersion::new(99),
        SchemaNamespace::new(1),
        SchemaRevision::new(1),
        Vec::new(),
        span,
    );
    match validate_schema(schema, validation_limits()) {
        Ok(_) => panic!("unsupported schema format should fail"),
        Err(error) => error,
    }
}

const fn limits() -> AuthoringLimitsV1 {
    AuthoringLimitsV1::new(4_096, 1_024, 32, 8, 8, 8, 8, 8, 8, 8, 8, 8, 64, 4_096)
}

const fn validation_limits() -> ValidationLimits {
    ValidationLimits::new(8, 8, 8, 8, 8, 8, 8, 8, 8)
}
