mod support;

use fenestra_ui_ir::prototype::{IrValidationErrorKind, validate_construction, validate_schema};

use support::construction_faults::CONSTRUCTION_FAULTS;
use support::malformed::{Fault, malformed_fixture};

#[test]
fn malformed_fixture_corpus_covers_every_error_kind_and_span() {
    for fault in CONSTRUCTION_FAULTS {
        let (manifest, program, limits, expected_kind, expected_span) = malformed_fixture(fault);
        let error = match validate_schema(manifest, limits) {
            Ok(schema) => validate_construction(&schema, program, limits)
                .expect_err("malformed construction should fail validation"),
            Err(error) => error,
        };

        assert_eq!(error.kind(), expected_kind, "unexpected kind for {fault:?}");
        assert_eq!(error.span(), expected_span, "unexpected span for {fault:?}");
    }
}

#[test]
fn declaration_order_selects_the_first_error_in_one_stage() {
    let (manifest, _program, limits, _, first_span) =
        malformed_fixture(Fault::TwoDuplicateComponents);
    let error = validate_schema(manifest, limits).expect_err("fixture should fail");

    assert_eq!(error.kind(), IrValidationErrorKind::DuplicateComponent);
    assert_eq!(error.span(), first_span);
}

#[test]
fn invalidation_ownership_rejects_each_forbidden_class() {
    for fault in [Fault::InvalidPropertySurface, Fault::InvalidRegionSurface] {
        let (manifest, program, limits, expected_kind, expected_span) = malformed_fixture(fault);
        let error = match validate_schema(manifest, limits) {
            Ok(schema) => validate_construction(&schema, program, limits)
                .expect_err("invalid region invalidation should fail"),
            Err(error) => error,
        };

        assert_eq!(error.kind(), expected_kind);
        assert_eq!(error.span(), expected_span);
    }
}

#[test]
fn error_formatting_does_not_expose_values_or_internal_domains() {
    let (manifest, _program, limits, _, _) = malformed_fixture(Fault::PropertyDefaultTypeMismatch);
    let error = validate_schema(manifest, limits).expect_err("schema should fail");
    let rendered = format!("{error:?} {error}");

    assert!(rendered.contains("property-default-type-mismatch"));
    for forbidden in ["ScalarI32", "validation_domain", "dense", "index"] {
        assert!(
            !rendered.contains(forbidden),
            "leaked {forbidden}: {rendered}"
        );
    }
}
