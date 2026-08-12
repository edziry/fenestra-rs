use std::collections::BTreeSet;

use super::source::{all_source, read, source_dir};

const V1_EXPORTS: [&str; 29] = [
    "AnchorKindV1",
    "AuthoringDiagnosticKindV1",
    "AuthoringDiagnosticV1",
    "AuthoringFormatVersion",
    "AuthoringFrontendV1",
    "AuthoringLimitKindV1",
    "AuthoringLimitsV1",
    "CompiledAuthoringV1",
    "DiagnosticLocationV1",
    "FenSourceV1",
    "GeneratedRustV1",
    "PhysicalOriginV1",
    "REFERENCE_AUTHORING_LIMITS_V1",
    "REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1",
    "SUPPORTED_AUTHORING_FORMAT",
    "SemanticArtifactErrorKindV1",
    "SemanticArtifactErrorV1",
    "SemanticArtifactLimitKindV1",
    "SemanticArtifactLimitsV1",
    "SemanticArtifactV1",
    "SourceMapEntryV1",
    "SourceMapV1",
    "canonical_rust_v1",
    "canonical_semantics_v1",
    "compile_fen_v1",
    "compile_ui_v1",
    "diagnostic_tokens_v1",
    "emit_tokens_v1",
    "expand_ui_v1",
];

const V2_EXPORTS: [&str; 29] = [
    "AnchorKindV2",
    "AuthoringDiagnosticKindV2",
    "AuthoringDiagnosticV2",
    "AuthoringFrontendV2",
    "AuthoringLimitKindV2",
    "AuthoringLimitsV2",
    "CompiledAuthoringV2",
    "DiagnosticLocationV2",
    "FenSourceV2",
    "GeneratedRustV2",
    "PhysicalOriginV2",
    "REFERENCE_AUTHORING_LIMITS_V2",
    "REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V2",
    "SUPPORTED_AUTHORING_FORMAT_V2",
    "SemanticArtifactErrorKindV2",
    "SemanticArtifactErrorV2",
    "SemanticArtifactLimitKindV2",
    "SemanticArtifactLimitsV2",
    "SemanticArtifactV2",
    "SourceMapEntryV2",
    "SourceMapV2",
    "canonical_rust_v2",
    "canonical_semantics_v2",
    "compile_fen_v2",
    "compile_ui_v2",
    "diagnostic_tokens_v2",
    "emit_tokens_v2",
    "expand_ui",
    "expand_ui_v2",
];

const PUBLIC_STRUCTS: [&str; 23] = [
    "AuthoringDiagnosticV1",
    "AuthoringDiagnosticV2",
    "AuthoringFormatVersion",
    "AuthoringLimitsV1",
    "AuthoringLimitsV2",
    "CompiledAuthoringV1",
    "CompiledAuthoringV2",
    "FenSourceV1",
    "FenSourceV2",
    "GeneratedRustV1",
    "GeneratedRustV2",
    "PhysicalOriginV1",
    "PhysicalOriginV2",
    "SemanticArtifactErrorV1",
    "SemanticArtifactErrorV2",
    "SemanticArtifactLimitsV1",
    "SemanticArtifactLimitsV2",
    "SemanticArtifactV1",
    "SemanticArtifactV2",
    "SourceMapEntryV1",
    "SourceMapEntryV2",
    "SourceMapV1",
    "SourceMapV2",
];

#[test]
fn prototype_registry_is_exactly_29_v1_then_29_additive_names() {
    let source = read(&source_dir().join("lib.rs"));
    let all = all_source();
    for forbidden in ["include!", "#[macro_export]"] {
        assert!(!all.contains(forbidden), "unexpected API form {forbidden}");
    }
    let marker = "pub mod prototype {";
    assert!(source.contains("#[doc(hidden)]\npub mod prototype {"));
    let offset = source.find(marker).expect("prototype module");
    assert!(!source[..offset].lines().any(is_public_line));
    let prototype = &source[offset + marker.len()..source.len() - 2];
    for forbidden in [" as ", "::*", "pub type ", "pub trait ", "pub mod "] {
        assert!(!prototype.contains(forbidden), "unexpected {forbidden}");
    }
    assert!(
        prototype
            .lines()
            .filter(|line| is_public_line(line))
            .all(|line| line.trim_start().starts_with("pub use crate::"))
    );

    let observed = prototype_exports(prototype);
    let v1 = V1_EXPORTS.into_iter().collect::<BTreeSet<_>>();
    let v2 = V2_EXPORTS.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(v1.len(), 29);
    assert_eq!(v2.len(), 29);
    assert!(v1.is_disjoint(&v2));
    assert_eq!(observed.len(), 58);
    assert_eq!(observed, v1.union(&v2).copied().collect());
}

#[test]
fn public_struct_registry_is_exact_and_every_field_is_private() {
    let source = all_source();
    let lines = source.lines().collect::<Vec<_>>();
    let mut observed = BTreeSet::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if !line.starts_with("pub struct ") {
            index += 1;
            continue;
        }
        let name = line["pub struct ".len()..]
            .split(['<', '(', '{', ';'])
            .next()
            .expect("struct name")
            .trim();
        assert!(observed.insert(name.to_owned()), "duplicate {name}");
        if line.ends_with(';') {
            assert!(!line.contains("(pub "), "public tuple field on {name}");
            index += 1;
            continue;
        }
        index += 1;
        while index < lines.len() && lines[index].trim() != "}" {
            assert!(
                !lines[index].trim_start().starts_with("pub "),
                "public field on {name}: {}",
                lines[index]
            );
            index += 1;
        }
        index += 1;
    }
    assert_eq!(
        observed,
        PUBLIC_STRUCTS.into_iter().map(str::to_owned).collect()
    );
}

fn prototype_exports(source: &str) -> BTreeSet<&str> {
    let mut observed = BTreeSet::new();
    for item in source.split("pub use crate::").skip(1) {
        let names = if let Some(start) = item.find("::{") {
            &item[start + 3..item.find("};").expect("grouped reexport")]
        } else {
            let end = item.find(';').expect("singleton reexport");
            item[..end].rsplit("::").next().expect("singleton name")
        };
        for name in names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            assert!(observed.insert(name), "duplicate reexport {name}");
        }
    }
    observed
}

fn is_public_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("pub ") || line.starts_with("pub(")
}
