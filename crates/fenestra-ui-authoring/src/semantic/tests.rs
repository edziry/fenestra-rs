mod support;

use fenestra_ui_ir::prototype::{InvalidationClass, PropertyValue, ValueType};

use super::{
    REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1, SemanticArtifactErrorKindV1,
    SemanticArtifactLimitKindV1, SemanticArtifactLimitsV1, observe_resolved_v1,
};
use crate::resolved::{ResolvedChildV1, ResolvedDocumentV1};

#[test]
fn semantic_artifact_vocabulary_and_reference_limits_are_closed() {
    assert_eq!(
        SemanticArtifactLimitKindV1::ALL,
        [
            SemanticArtifactLimitKindV1::Records,
            SemanticArtifactLimitKindV1::LineBytes,
            SemanticArtifactLimitKindV1::ArtifactBytes,
        ]
    );
    assert_eq!(
        SemanticArtifactErrorKindV1::ALL,
        [
            SemanticArtifactErrorKindV1::LimitExceeded(SemanticArtifactLimitKindV1::Records),
            SemanticArtifactErrorKindV1::LimitExceeded(SemanticArtifactLimitKindV1::LineBytes),
            SemanticArtifactErrorKindV1::LimitExceeded(SemanticArtifactLimitKindV1::ArtifactBytes,),
            SemanticArtifactErrorKindV1::InvalidCompiledDocument,
        ]
    );
    assert_eq!(
        REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1.limit(SemanticArtifactLimitKindV1::ArtifactBytes),
        8_192
    );
    assert_eq!(
        REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1.limit(SemanticArtifactLimitKindV1::LineBytes),
        512
    );
    assert_eq!(
        REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1.limit(SemanticArtifactLimitKindV1::Records),
        64
    );
}

#[test]
fn every_retained_semantic_field_class_changes_the_observation() {
    assert_changes("format", |document| document.format = 2);
    assert_changes("namespace", |document| document.schema.namespace = 9_001);
    assert_changes("revision", |document| document.schema.revision = 2);
    assert_changes("property-order", |document| {
        document.schema.components[0].properties.swap(0, 1);
    });
    assert_changes("component-name", |document| {
        document.schema.components[0].name = "renamed".into();
    });
    assert_changes("component-id", |document| {
        document.schema.components[0].id = 4;
    });
    assert_changes("property-name", |document| {
        document.schema.components[0].properties[0].name = "renamed".into();
    });
    assert_changes("property-id", |document| {
        document.schema.components[0].properties[0].id = 4;
    });
    assert_changes("property-type", |document| {
        document.schema.components[0].properties[0].value_type = ValueType::Bool;
    });
    assert_changes("property-value", |document| {
        document.schema.components[0].properties[0].default = PropertyValue::ScalarI32(11);
    });
    assert_changes("property-invalidation", |document| {
        document.schema.components[0].properties[0].invalidation =
            support::invalidation(&[InvalidationClass::Paint]);
    });
    assert_changes("template-order", |document| {
        document.construction.templates.swap(0, 1);
    });
    assert_changes("template-name", |document| {
        document.construction.templates[0].name = "renamed".into();
    });
    assert_changes("template-id", |document| {
        document.construction.templates[0].id = 9;
    });
    assert_changes("template-component", |document| {
        document.construction.templates[0].component = 9;
    });
    assert_changes("initial-property-ref", |document| {
        document.construction.templates[0].initial_properties[0].property = 9;
    });
    assert_changes("initial-property-value", |document| {
        document.construction.templates[0].initial_properties[0].value =
            PropertyValue::ScalarI32(11);
    });
    assert_changes("child-kind", |document| {
        document.construction.templates[0].children[0] = ResolvedChildV1::Region {
            region: 0,
            anchor: 8,
        };
    });
    assert_changes("static-child-target", |document| {
        match &mut document.construction.templates[0].children[0] {
            ResolvedChildV1::Static {
                template,
                anchor: _,
            } => *template = 9,
            ResolvedChildV1::Region { region, anchor: _ } => *region = 9,
        }
    });
    assert_changes("region-child-target", |document| {
        match &mut document.construction.templates[0].children[1] {
            ResolvedChildV1::Static {
                template,
                anchor: _,
            } => *template = 9,
            ResolvedChildV1::Region { region, anchor: _ } => *region = 9,
        }
    });
    assert_changes("region-name", |document| {
        document.construction.regions[0].name = "renamed".into();
    });
    assert_changes("region-id", |document| {
        document.construction.regions[0].id = 9;
    });
    assert_changes("region-owner", |document| {
        document.construction.regions[0].owner = 9;
    });
    assert_changes("region-repeat", |document| {
        document.construction.regions[0].repeat_body = 9;
    });
    assert_changes("region-invalidation", |document| {
        document.construction.regions[0].invalidation =
            support::invalidation(&[InvalidationClass::Paint]);
    });
    assert_changes("key", |document| {
        document.construction.regions[0].initial_keys[0].value = 8;
    });
    assert_changes("style-target", |document| {
        document.style.assignments[0].target = 9;
    });
    assert_changes("style-property", |document| {
        document.style.assignments[0].property = 9;
    });
    assert_changes("style-value", |document| {
        document.style.assignments[0].value = support::alternate_policy();
    });
    assert_changes("span", |document| {
        let properties = &mut document.schema.components[0].properties;
        let first = properties[0].anchor;
        properties[0].anchor = properties[1].anchor;
        properties[1].anchor = first;
    });
}

#[test]
fn invalid_anchor_and_name_mutations_are_rejected_without_serializing_input() {
    assert_rejected(|document| {
        document.schema.components[0].properties[0].anchor = 0;
    });
    assert_rejected(|document| {
        document.schema.components[0].name = "\u{e9}".into();
    });
}

#[test]
fn semantic_artifact_bounds_are_inclusive_and_one_under_is_typed() {
    let document = support::document();
    let artifact = observe_resolved_v1(&document, REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1)
        .expect("the private reference document should encode");
    assert!(!artifact.as_bytes().is_empty());
    let artifact_bytes = artifact.as_bytes().len();
    let line_bytes = artifact.as_str().lines().map(str::len).max().unwrap_or(0);
    let records = artifact
        .as_str()
        .lines()
        .filter(|line| line.starts_with("record|"))
        .count();
    let exact = SemanticArtifactLimitsV1::new(artifact_bytes, line_bytes, records);
    assert_eq!(
        observe_resolved_v1(&document, exact)
            .expect("exact semantic limits should be inclusive")
            .as_bytes(),
        artifact.as_bytes()
    );

    let cases = [
        (
            SemanticArtifactLimitsV1::new(artifact_bytes - 1, line_bytes, records),
            SemanticArtifactLimitKindV1::ArtifactBytes,
        ),
        (
            SemanticArtifactLimitsV1::new(artifact_bytes, line_bytes - 1, records),
            SemanticArtifactLimitKindV1::LineBytes,
        ),
        (
            SemanticArtifactLimitsV1::new(artifact_bytes, line_bytes, records - 1),
            SemanticArtifactLimitKindV1::Records,
        ),
    ];
    for (limits, expected) in cases {
        let error = observe_resolved_v1(&document, limits).expect_err("one under should fail");
        assert_eq!(
            error.kind(),
            SemanticArtifactErrorKindV1::LimitExceeded(expected)
        );
    }

    let all_cross = observe_resolved_v1(&document, SemanticArtifactLimitsV1::new(0, 0, 0))
        .expect_err("simultaneous crossings should fail");
    assert_eq!(
        all_cross.kind(),
        SemanticArtifactErrorKindV1::LimitExceeded(SemanticArtifactLimitKindV1::Records)
    );
    let line_and_bytes_cross =
        observe_resolved_v1(&document, SemanticArtifactLimitsV1::new(0, 0, records))
            .expect_err("line crossing should precede artifact bytes");
    assert_eq!(
        line_and_bytes_cross.kind(),
        SemanticArtifactErrorKindV1::LimitExceeded(SemanticArtifactLimitKindV1::LineBytes)
    );
}

fn assert_changes(case: &str, mutate: impl FnOnce(&mut ResolvedDocumentV1)) {
    let baseline = observe_resolved_v1(&support::document(), REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1)
        .expect("the private reference document should encode");
    let mut changed = support::document();
    mutate(&mut changed);
    let observed = observe_resolved_v1(&changed, REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1)
        .unwrap_or_else(|error| panic!("{case} was rejected: {error}"));
    assert_ne!(observed.as_bytes(), baseline.as_bytes(), "missed {case}");
}

fn assert_rejected(mutate: impl FnOnce(&mut ResolvedDocumentV1)) {
    let mut document = support::document();
    mutate(&mut document);
    let error = observe_resolved_v1(&document, REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1)
        .expect_err("invalid private model should be rejected");
    assert_eq!(
        error.kind(),
        SemanticArtifactErrorKindV1::InvalidCompiledDocument
    );
}
