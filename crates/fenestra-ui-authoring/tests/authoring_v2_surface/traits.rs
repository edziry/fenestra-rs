use std::error::Error;
use std::fmt::{Debug, Display};
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::api::{
    SemanticArtifactErrorKindV2, SemanticArtifactErrorV2, SemanticArtifactLimitKindV2,
    SemanticArtifactLimitsV2, SemanticArtifactV2,
};

use super::source::{all_source, significant};
use super::support::{item_attributes, names, trait_impl};

#[test]
fn copy_value_types_have_exact_declared_derives() {
    let source = all_source();
    for name in [
        "AnchorKindV2",
        "AuthoringDiagnosticKindV2",
        "AuthoringFrontendV2",
        "AuthoringLimitKindV2",
        "AuthoringLimitsV2",
        "SemanticArtifactErrorKindV2",
        "SemanticArtifactLimitKindV2",
        "SemanticArtifactLimitsV2",
    ] {
        let marker = if source.contains(&format!("pub enum {name}")) {
            format!("pub enum {name}")
        } else {
            format!("pub struct {name}")
        };
        assert_eq!(
            item_attributes(&source, &marker),
            names(&["#[derive(Clone, Copy, Debug, Eq, PartialEq)]"]),
            "derives on {name}"
        );
    }
    for name in ["AuthoringDiagnosticV2", "FenSourceV2", "PhysicalOriginV2"] {
        assert!(
            item_attributes(&source, &format!("pub struct {name}"))
                .contains("#[derive(Clone, Copy)]")
        );
    }
    assert!(
        item_attributes(&source, "pub enum DiagnosticLocationV2")
            .contains("#[derive(Clone, Copy)]")
    );
    assert_eq!(
        item_attributes(&source, "pub struct PhysicalOriginV2"),
        names(&["#[derive(Clone, Copy)]", "#[non_exhaustive]"])
    );
}

#[test]
fn semantic_types_have_the_frozen_thread_safe_auto_traits() {
    assert_value::<SemanticArtifactLimitKindV2>();
    assert_value::<SemanticArtifactLimitsV2>();
    assert_value::<SemanticArtifactErrorKindV2>();
    assert_error::<SemanticArtifactErrorV2>();
    assert_artifact::<SemanticArtifactV2>();
}

#[test]
fn errors_and_opaque_outputs_have_exact_redacted_formatting() {
    let source = all_source();
    let diagnostic_display =
        significant(trait_impl(&source, "fmt::Display", "AuthoringDiagnosticV2"));
    assert!(diagnostic_display.contains("IrValidation(_)=>formatter.write_str(\"ir-validation\")"));
    let diagnostic_debug = significant(trait_impl(&source, "fmt::Debug", "AuthoringDiagnosticV2"));
    assert!(diagnostic_debug.contains("AuthoringDiagnosticV2({self})"));
    assert!(
        significant(trait_impl(&source, "Error", "AuthoringDiagnosticV2"))
            .contains("implErrorforAuthoringDiagnosticV2{}")
    );

    let semantic_debug = significant(trait_impl(&source, "fmt::Debug", "SemanticArtifactErrorV2"));
    assert!(semantic_debug.contains("SemanticArtifactErrorV2({self})"));
    let artifact_debug = significant(trait_impl(&source, "fmt::Debug", "SemanticArtifactV2"));
    assert!(artifact_debug.contains("debug_struct(\"SemanticArtifactV2\")"));
    assert!(artifact_debug.contains("field(\"bytes\",&self.source.len())"));
    let generated_debug = significant(trait_impl(&source, "fmt::Debug", "GeneratedRustV2"));
    assert!(generated_debug.contains("debug_struct(\"GeneratedRustV2\")"));
    assert!(generated_debug.contains("field(\"bytes\",&self.source.len())"));
    let map_debug = significant(trait_impl(&source, "fmt::Debug", "SourceMapV2"));
    assert!(map_debug.contains("debug_struct(\"SourceMapV2\")"));
    assert!(map_debug.contains("field(\"entries\",&self.entries.len())"));

    for type_name in [
        "CompiledAuthoringV2",
        "GeneratedRustV2",
        "SemanticArtifactErrorV2",
        "SemanticArtifactV2",
        "SourceMapV2",
    ] {
        let body = significant(trait_impl(&source, "fmt::Debug", type_name));
        for forbidden in ["canonical_label", "physical_origin", "FenBytes", "UiToken"] {
            assert!(
                !body.contains(forbidden),
                "{type_name} debug leaked {forbidden}"
            );
        }
    }
}

fn assert_value<T>()
where
    T: Clone
        + Copy
        + Debug
        + Eq
        + PartialEq
        + Send
        + Sync
        + Unpin
        + RefUnwindSafe
        + UnwindSafe
        + 'static,
{
}

fn assert_error<T>()
where
    T: Debug + Display + Error + Send + Sync + Unpin + RefUnwindSafe + UnwindSafe + 'static,
{
}

fn assert_artifact<T>()
where
    T: Debug + Send + Sync + Unpin + RefUnwindSafe + UnwindSafe + 'static,
{
}
