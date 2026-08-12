use fenestra_ui_ir::prototype::SourceSpan;
use fenestra_ui_layout::prototype::ReferenceStackEngineV1;
use fenestra_ui_spatial::prototype::{
    SpatialLimitsV2, SpatialOwnedInputV2, SpatialResolveErrorV2, SpatialViewportV2,
    resolve_spatial_v2,
};

use crate::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};

use super::source::all_source;
use super::support::{
    assert_method_surface, enum_body, public_constants, public_methods, significant, struct_fields,
    trait_impl,
};

#[test]
fn runtime_ir_error_kind_has_exact_variants_and_no_surface() {
    let source = all_source();
    assert_eq!(
        enum_body(&source, "RuntimeSpatialIrErrorKindV2"),
        "ArithmeticExhausted,InvariantViolation,Resolve(SpatialResolveErrorV2),"
    );
    assert!(public_methods(&source, "RuntimeSpatialIrErrorKindV2").is_empty());
    assert!(public_constants(&source, "RuntimeSpatialIrErrorKindV2").is_empty());
    assert!(!source.contains("RuntimeSpatialIrErrorKindV2::ALL"));

    let resolve = resolve_error();
    let values = [
        RuntimeSpatialIrErrorKindV2::ArithmeticExhausted,
        RuntimeSpatialIrErrorKindV2::InvariantViolation,
        RuntimeSpatialIrErrorKindV2::Resolve(resolve),
    ];
    assert_eq!(values[0], RuntimeSpatialIrErrorKindV2::ArithmeticExhausted);
    assert_eq!(values[1], RuntimeSpatialIrErrorKindV2::InvariantViolation);
    match values[2] {
        RuntimeSpatialIrErrorKindV2::Resolve(actual) => assert_eq!(actual, resolve),
        _ => panic!("resolve payload must remain typed"),
    }
}

#[test]
fn runtime_ir_error_has_exact_private_storage_and_getters() {
    let source = all_source();
    assert_eq!(
        struct_fields(&source, "RuntimeSpatialIrErrorV2"),
        [
            ("kind".to_owned(), "RuntimeSpatialIrErrorKindV2".to_owned()),
            ("span".to_owned(), "SourceSpan".to_owned()),
        ]
    );
    assert_method_surface(
        &source,
        "RuntimeSpatialIrErrorV2",
        &["kind", "span"],
        &["kind", "span"],
        &["kind", "span"],
    );
    let _: fn(RuntimeSpatialIrErrorV2) -> RuntimeSpatialIrErrorKindV2 =
        RuntimeSpatialIrErrorV2::kind;
    let _: fn(RuntimeSpatialIrErrorV2) -> SourceSpan = RuntimeSpatialIrErrorV2::span;
}

#[test]
fn runtime_ir_error_formatting_and_source_are_exact_and_redacted() {
    let source = all_source();
    let display = significant(trait_impl(
        &source,
        "fmt::Display",
        "RuntimeSpatialIrErrorV2",
    ));
    for fragment in [
        "ArithmeticExhausted=>\"arithmetic-exhausted\"",
        "InvariantViolation=>\"invariant-violation\"",
        "Resolve(_)=>\"resolve\"",
        "runtime-spatial-ir-error({label})",
    ] {
        assert!(
            display.contains(fragment),
            "missing display contract {fragment}"
        );
    }
    let debug = significant(trait_impl(&source, "fmt::Debug", "RuntimeSpatialIrErrorV2"));
    assert!(debug.contains("RuntimeSpatialIrErrorV2({self})"));
    let error = significant(trait_impl(&source, "Error", "RuntimeSpatialIrErrorV2"));
    assert!(error.contains("fnsource(&self)->Option<&(dynError+'static)>{None}"));
    for forbidden in [
        "symbols",
        "logical",
        "property",
        "counts",
        "limits",
        "bytes",
        "allocation",
        "evidence",
    ] {
        assert!(!display.contains(forbidden));
        assert!(!debug.contains(forbidden));
    }
}

#[test]
fn runtime_spatial_error_adds_ir_immediately_before_resolve() {
    let source = all_source();
    assert_eq!(
        enum_body(&source, "RuntimeSpatialErrorV2"),
        concat!(
            "ViewportMismatch,",
            "MappingLengthMismatch,",
            "MissingLogicalNode{key:SpatialNodeKeyV2},",
            "DuplicateLogicalNode{key:SpatialNodeKeyV2},",
            "Ir(RuntimeSpatialIrErrorV2),",
            "Resolve(SpatialResolveErrorV2),"
        )
    );
    assert!(public_methods(&source, "RuntimeSpatialErrorV2").is_empty());
    assert!(public_constants(&source, "RuntimeSpatialErrorV2").is_empty());

    let display = significant(trait_impl(&source, "fmt::Display", "RuntimeSpatialErrorV2"));
    for fragment in [
        "Self::ViewportMismatch=>\"viewport-mismatch\"",
        "Self::MappingLengthMismatch=>\"mapping-length-mismatch\"",
        "Self::MissingLogicalNode{..}=>\"missing-logical-node\"",
        "Self::DuplicateLogicalNode{..}=>\"duplicate-logical-node\"",
        "Self::Ir(_)=>\"ir\"",
        "Self::Resolve(_)=>\"resolve\"",
        "runtime-spatial-error({label})",
    ] {
        assert!(
            display.contains(fragment),
            "missing outer error contract {fragment}"
        );
    }
    let debug = significant(trait_impl(&source, "fmt::Debug", "RuntimeSpatialErrorV2"));
    assert!(debug.contains("RuntimeSpatialErrorV2({self})"));
    let error = significant(trait_impl(&source, "Error", "RuntimeSpatialErrorV2"));
    assert!(error.contains("fnsource(&self)->Option<&(dynError+'static)>{None}"));
}

fn resolve_error() -> SpatialResolveErrorV2 {
    let input = SpatialOwnedInputV2::new(
        SpatialViewportV2::new(1, 1),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
    );
    match resolve_spatial_v2(
        &ReferenceStackEngineV1::new(),
        std::sync::Arc::new(input),
        SpatialLimitsV2::new([usize::MAX; 30]),
    ) {
        Err(error) => error,
        Ok(_) => panic!("empty topology must fail"),
    }
}
