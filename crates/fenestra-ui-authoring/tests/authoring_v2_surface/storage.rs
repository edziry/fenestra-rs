use super::source::all_source;
use super::support::{
    assert_method_surface, enum_body, names, public_constants, public_methods, struct_fields,
};

#[test]
fn opaque_output_compiled_and_source_map_storage_is_exact() {
    let source = all_source();
    assert_eq!(
        struct_fields(&source, "GeneratedRustV2"),
        [("source".to_owned(), "Box<str>".to_owned())]
    );
    assert_eq!(
        struct_fields(&source, "SourceMapEntryV2"),
        [
            ("logical_span".to_owned(), "SourceSpan".to_owned()),
            ("anchor_kind".to_owned(), "AnchorKindV2".to_owned()),
            ("canonical_label".to_owned(), "Box<str>".to_owned()),
            ("physical_origin".to_owned(), "PhysicalOriginV2".to_owned()),
        ]
    );
    assert_eq!(
        struct_fields(&source, "SourceMapV2"),
        [("entries".to_owned(), "Vec<SourceMapEntryV2>".to_owned())]
    );
    assert_eq!(
        struct_fields(&source, "CompiledAuthoringV2"),
        [
            ("frontend".to_owned(), "AuthoringFrontendV2".to_owned()),
            ("document_origin".to_owned(), "PhysicalOriginV2".to_owned()),
            ("schema".to_owned(), "SchemaManifest".to_owned()),
            ("construction".to_owned(), "ConstructionProgram".to_owned()),
            ("style".to_owned(), "StyleProgram".to_owned()),
            ("spatial".to_owned(), "SpatialProgramV2".to_owned()),
            ("logical_source_catalog".to_owned(), "Vec<u8>".to_owned()),
            ("source_map".to_owned(), "SourceMapV2".to_owned()),
            ("resolved".to_owned(), "ResolvedDocumentV2".to_owned()),
        ]
    );

    assert_method_surface(&source, "GeneratedRustV2", &["as_str"], &[]);
    assert_method_surface(
        &source,
        "SourceMapEntryV2",
        &[
            "anchor_kind",
            "canonical_label",
            "logical_span",
            "physical_origin",
        ],
        &["anchor_kind", "logical_span", "physical_origin"],
    );
    assert_method_surface(&source, "SourceMapV2", &["entries"], &[]);
    assert_method_surface(
        &source,
        "CompiledAuthoringV2",
        &[
            "construction",
            "logical_source_catalog",
            "schema",
            "source_map",
            "spatial",
            "style",
        ],
        &["construction", "schema", "source_map", "spatial", "style"],
    );
}

#[test]
fn limits_sources_and_diagnostics_have_exact_private_storage_and_methods() {
    let source = all_source();
    assert_eq!(
        struct_fields(&source, "AuthoringLimitsV2"),
        [("values".to_owned(), "[usize;28]".to_owned())]
    );
    assert_eq!(
        struct_fields(&source, "FenSourceV2"),
        [
            ("source".to_owned(), "SourceId".to_owned()),
            ("bytes".to_owned(), "&'a[u8]".to_owned()),
        ]
    );
    assert_eq!(
        struct_fields(&source, "PhysicalOriginV2"),
        [("kind".to_owned(), "PhysicalOriginKindV2".to_owned())]
    );
    assert_eq!(
        struct_fields(&source, "AuthoringDiagnosticV2"),
        [
            ("frontend".to_owned(), "AuthoringFrontendV2".to_owned()),
            ("kind".to_owned(), "AuthoringDiagnosticKindV2".to_owned()),
            ("location".to_owned(), "DiagnosticLocationV2".to_owned()),
        ]
    );
    assert_eq!(
        enum_body(&source, "DiagnosticLocationV2"),
        concat!(
            "Physical(PhysicalOriginV2),Anchored{logical:SourceSpan,",
            "anchor_kind:AnchorKindV2,physical:PhysicalOriginV2},"
        )
    );

    assert_method_surface(
        &source,
        "AuthoringLimitsV2",
        &["limit", "new"],
        &["limit", "new"],
    );
    assert_method_surface(&source, "FenSourceV2", &["new"], &["new"]);
    assert_method_surface(
        &source,
        "PhysicalOriginV2",
        &["fen_byte_range", "source_id"],
        &["fen_byte_range", "source_id"],
    );
    assert_method_surface(
        &source,
        "AuthoringDiagnosticV2",
        &["frontend", "kind", "location"],
        &["frontend", "kind", "location"],
    );
    assert!(public_methods(&source, "DiagnosticLocationV2").is_empty());
    assert!(public_constants(&source, "DiagnosticLocationV2").is_empty());
}

#[test]
fn semantic_artifact_storage_and_observation_are_exact() {
    let source = all_source();
    assert_eq!(
        struct_fields(&source, "SemanticArtifactLimitsV2"),
        [("values".to_owned(), "[usize;3]".to_owned())]
    );
    assert_eq!(
        struct_fields(&source, "SemanticArtifactErrorV2"),
        [("kind".to_owned(), "SemanticArtifactErrorKindV2".to_owned())]
    );
    assert_eq!(
        struct_fields(&source, "SemanticArtifactV2"),
        [("source".to_owned(), "Box<str>".to_owned())]
    );
    assert_method_surface(
        &source,
        "SemanticArtifactLimitsV2",
        &["limit", "new"],
        &["limit", "new"],
    );
    assert_method_surface(&source, "SemanticArtifactErrorV2", &["kind"], &["kind"]);
    assert_method_surface(&source, "SemanticArtifactV2", &["as_bytes", "as_str"], &[]);
}

#[test]
fn enums_expose_only_their_frozen_all_constants() {
    let source = all_source();
    for type_name in [
        "AnchorKindV2",
        "AuthoringDiagnosticKindV2",
        "AuthoringFrontendV2",
        "AuthoringLimitKindV2",
        "SemanticArtifactErrorKindV2",
        "SemanticArtifactLimitKindV2",
    ] {
        assert!(public_methods(&source, type_name).is_empty());
        assert_eq!(public_constants(&source, type_name), names(&["ALL"]));
    }
}
