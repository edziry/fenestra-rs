use fenestra_ui_authoring::prototype::SUPPORTED_AUTHORING_FORMAT;
use fenestra_ui_ir::prototype::IrValidationErrorKind;

use crate::api::{
    AnchorKindV2, AuthoringDiagnosticKindV2, AuthoringFrontendV2, AuthoringLimitKindV2,
    AuthoringLimitsV2, REFERENCE_AUTHORING_LIMITS_V2, REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V2,
    SUPPORTED_AUTHORING_FORMAT_V2, SemanticArtifactErrorKindV2, SemanticArtifactLimitKindV2,
    SemanticArtifactLimitsV2,
};

use super::source::{all_source, significant};
use super::support::enum_body;

const LIMITS: [AuthoringLimitKindV2; 28] = [
    AuthoringLimitKindV2::FenSourceBytes,
    AuthoringLimitKindV2::Tokens,
    AuthoringLimitKindV2::IdentifierBytes,
    AuthoringLimitKindV2::NestingDepth,
    AuthoringLimitKindV2::Components,
    AuthoringLimitKindV2::Properties,
    AuthoringLimitKindV2::Templates,
    AuthoringLimitKindV2::Regions,
    AuthoringLimitKindV2::ChildSlots,
    AuthoringLimitKindV2::InitialProperties,
    AuthoringLimitKindV2::InitialKeys,
    AuthoringLimitKindV2::StyleAssignments,
    AuthoringLimitKindV2::Images,
    AuthoringLimitKindV2::ImageBytes,
    AuthoringLimitKindV2::SpatialNodes,
    AuthoringLimitKindV2::SpatialFields,
    AuthoringLimitKindV2::Shapes,
    AuthoringLimitKindV2::Paths,
    AuthoringLimitKindV2::PathVerbs,
    AuthoringLimitKindV2::PolygonPoints,
    AuthoringLimitKindV2::Brushes,
    AuthoringLimitKindV2::GradientStops,
    AuthoringLimitKindV2::Clips,
    AuthoringLimitKindV2::PaintItems,
    AuthoringLimitKindV2::HitItems,
    AuthoringLimitKindV2::SemanticItems,
    AuthoringLimitKindV2::SourceAnchors,
    AuthoringLimitKindV2::GeneratedRustBytes,
];

const ANCHORS: [AnchorKindV2; 30] = [
    AnchorKindV2::Document,
    AnchorKindV2::Schema,
    AnchorKindV2::Component,
    AnchorKindV2::Property,
    AnchorKindV2::Construction,
    AnchorKindV2::Template,
    AnchorKindV2::InitialProperty,
    AnchorKindV2::StaticChild,
    AnchorKindV2::RegionChild,
    AnchorKindV2::Region,
    AnchorKindV2::InitialKey,
    AnchorKindV2::Style,
    AnchorKindV2::StyleAssignment,
    AnchorKindV2::Spatial,
    AnchorKindV2::Resources,
    AnchorKindV2::Image,
    AnchorKindV2::SpatialNode,
    AnchorKindV2::SpatialContainer,
    AnchorKindV2::SpatialPlacement,
    AnchorKindV2::SpatialTransform,
    AnchorKindV2::SpatialField,
    AnchorKindV2::SpatialShape,
    AnchorKindV2::SpatialPathVerb,
    AnchorKindV2::SpatialPolygonPoint,
    AnchorKindV2::SpatialBrush,
    AnchorKindV2::SpatialGradientStop,
    AnchorKindV2::SpatialClip,
    AnchorKindV2::SpatialPaint,
    AnchorKindV2::SpatialHit,
    AnchorKindV2::SpatialSemantic,
];

#[test]
fn version_frontends_limits_and_anchors_are_closed_and_ordered() {
    assert_eq!(SUPPORTED_AUTHORING_FORMAT.get(), 1);
    assert_eq!(SUPPORTED_AUTHORING_FORMAT_V2.get(), 2);
    assert_eq!(
        AuthoringFrontendV2::ALL,
        [AuthoringFrontendV2::Fen, AuthoringFrontendV2::UiMacro]
    );
    assert_eq!(AuthoringLimitKindV2::ALL, LIMITS);
    assert_eq!(AnchorKindV2::ALL, ANCHORS);

    let _: AuthoringLimitsV2 = REFERENCE_AUTHORING_LIMITS_V2;
    let _: SemanticArtifactLimitsV2 = REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V2;
}

#[test]
fn diagnostic_vocabulary_has_exact_v1_prefix_and_134_outcomes() {
    let mut expected = vec![
        AuthoringDiagnosticKindV2::InvalidUtf8,
        AuthoringDiagnosticKindV2::UnsupportedToken,
        AuthoringDiagnosticKindV2::UnsupportedAuthoringFormat,
        AuthoringDiagnosticKindV2::UnexpectedToken,
        AuthoringDiagnosticKindV2::UnexpectedEof,
        AuthoringDiagnosticKindV2::InvalidIdentifier,
        AuthoringDiagnosticKindV2::InvalidLiteral,
        AuthoringDiagnosticKindV2::DuplicateComponentName,
        AuthoringDiagnosticKindV2::DuplicatePropertyName,
        AuthoringDiagnosticKindV2::DuplicateTemplateName,
        AuthoringDiagnosticKindV2::DuplicateRegionName,
        AuthoringDiagnosticKindV2::UnknownComponentName,
        AuthoringDiagnosticKindV2::UnknownPropertyName,
        AuthoringDiagnosticKindV2::UnknownTemplateName,
        AuthoringDiagnosticKindV2::UnknownRegionName,
        AuthoringDiagnosticKindV2::ValueTypeMismatch,
        AuthoringDiagnosticKindV2::DuplicateSpatialNodeName,
        AuthoringDiagnosticKindV2::DuplicateSpatialImageName,
        AuthoringDiagnosticKindV2::DuplicateSpatialShapeName,
        AuthoringDiagnosticKindV2::DuplicateSpatialBrushName,
        AuthoringDiagnosticKindV2::DuplicateSpatialClipName,
        AuthoringDiagnosticKindV2::UnknownSpatialNodeName,
        AuthoringDiagnosticKindV2::UnknownSpatialImageName,
        AuthoringDiagnosticKindV2::UnknownSpatialShapeName,
        AuthoringDiagnosticKindV2::UnknownSpatialBrushName,
        AuthoringDiagnosticKindV2::UnknownSpatialClipName,
    ];
    expected.extend(LIMITS.map(AuthoringDiagnosticKindV2::LimitExceeded));
    expected.extend(IrValidationErrorKind::ALL.map(AuthoringDiagnosticKindV2::IrValidation));
    assert_eq!(IrValidationErrorKind::ALL.len(), 80);
    assert_eq!(AuthoringDiagnosticKindV2::ALL.len(), 134);
    assert_eq!(AuthoringDiagnosticKindV2::ALL.as_slice(), expected);
}

#[test]
fn semantic_artifact_vocabularies_are_exact() {
    assert_eq!(
        SemanticArtifactLimitKindV2::ALL,
        [
            SemanticArtifactLimitKindV2::Records,
            SemanticArtifactLimitKindV2::LineBytes,
            SemanticArtifactLimitKindV2::ArtifactBytes,
        ]
    );
    assert_eq!(
        SemanticArtifactErrorKindV2::ALL,
        [
            SemanticArtifactErrorKindV2::LimitExceeded(SemanticArtifactLimitKindV2::Records,),
            SemanticArtifactErrorKindV2::LimitExceeded(SemanticArtifactLimitKindV2::LineBytes,),
            SemanticArtifactErrorKindV2::LimitExceeded(SemanticArtifactLimitKindV2::ArtifactBytes,),
            SemanticArtifactErrorKindV2::InvalidCompiledDocument,
        ]
    );
}

#[test]
fn enum_declarations_and_private_labels_are_exact() {
    let source = all_source();
    assert_eq!(enum_body(&source, "AuthoringFrontendV2"), "Fen,UiMacro,");
    assert_eq!(
        enum_body(&source, "AuthoringLimitKindV2"),
        concat!(
            "FenSourceBytes,Tokens,IdentifierBytes,NestingDepth,Components,Properties,",
            "Templates,Regions,ChildSlots,InitialProperties,InitialKeys,StyleAssignments,",
            "Images,ImageBytes,SpatialNodes,SpatialFields,Shapes,Paths,PathVerbs,",
            "PolygonPoints,Brushes,GradientStops,Clips,PaintItems,HitItems,SemanticItems,",
            "SourceAnchors,GeneratedRustBytes,"
        )
    );
    assert_eq!(enum_body(&source, "AnchorKindV2"), ANCHOR_VARIANTS);
    assert_eq!(
        enum_body(&source, "SemanticArtifactLimitKindV2"),
        "ArtifactBytes,LineBytes,Records,"
    );
    assert_eq!(
        enum_body(&source, "SemanticArtifactErrorKindV2"),
        "LimitExceeded(SemanticArtifactLimitKindV2),InvalidCompiledDocument,"
    );

    let compact = significant(&source);
    for (variant, label) in LIMIT_LABELS {
        let fragment = format!("Self::{variant}=>\"{label}\"");
        assert!(compact.contains(&fragment), "missing label {fragment}");
    }
    for (variant, label) in NAME_LABELS {
        let fragment = format!("AuthoringDiagnosticKindV2::{variant}=>");
        let offset = compact.find(&fragment).expect("diagnostic display arm");
        let suffix = &compact[offset + fragment.len()..];
        assert!(
            suffix.starts_with(&format!("formatter.write_str(\"{label}\")"))
                || suffix.starts_with(&format!("{{formatter.write_str(\"{label}\")"))
        );
    }
}

const ANCHOR_VARIANTS: &str = concat!(
    "Document,Schema,Component,Property,Construction,Template,InitialProperty,",
    "StaticChild,RegionChild,Region,InitialKey,Style,StyleAssignment,Spatial,",
    "Resources,Image,SpatialNode,SpatialContainer,SpatialPlacement,SpatialTransform,",
    "SpatialField,SpatialShape,SpatialPathVerb,SpatialPolygonPoint,SpatialBrush,",
    "SpatialGradientStop,SpatialClip,SpatialPaint,SpatialHit,SpatialSemantic,"
);

const LIMIT_LABELS: [(&str, &str); 28] = [
    ("FenSourceBytes", "fen-source-bytes"),
    ("Tokens", "tokens"),
    ("IdentifierBytes", "identifier-bytes"),
    ("NestingDepth", "nesting-depth"),
    ("Components", "components"),
    ("Properties", "properties"),
    ("Templates", "templates"),
    ("Regions", "regions"),
    ("ChildSlots", "child-slots"),
    ("InitialProperties", "initial-properties"),
    ("InitialKeys", "initial-keys"),
    ("StyleAssignments", "style-assignments"),
    ("Images", "images"),
    ("ImageBytes", "image-bytes"),
    ("SpatialNodes", "spatial-nodes"),
    ("SpatialFields", "spatial-fields"),
    ("Shapes", "shapes"),
    ("Paths", "paths"),
    ("PathVerbs", "path-verbs"),
    ("PolygonPoints", "polygon-points"),
    ("Brushes", "brushes"),
    ("GradientStops", "gradient-stops"),
    ("Clips", "clips"),
    ("PaintItems", "paint-items"),
    ("HitItems", "hit-items"),
    ("SemanticItems", "semantic-items"),
    ("SourceAnchors", "source-anchors"),
    ("GeneratedRustBytes", "generated-rust-bytes"),
];

const NAME_LABELS: [(&str, &str); 10] = [
    ("DuplicateSpatialNodeName", "duplicate-spatial-node-name"),
    ("DuplicateSpatialImageName", "duplicate-spatial-image-name"),
    ("DuplicateSpatialShapeName", "duplicate-spatial-shape-name"),
    ("DuplicateSpatialBrushName", "duplicate-spatial-brush-name"),
    ("DuplicateSpatialClipName", "duplicate-spatial-clip-name"),
    ("UnknownSpatialNodeName", "unknown-spatial-node-name"),
    ("UnknownSpatialImageName", "unknown-spatial-image-name"),
    ("UnknownSpatialShapeName", "unknown-spatial-shape-name"),
    ("UnknownSpatialBrushName", "unknown-spatial-brush-name"),
    ("UnknownSpatialClipName", "unknown-spatial-clip-name"),
];
