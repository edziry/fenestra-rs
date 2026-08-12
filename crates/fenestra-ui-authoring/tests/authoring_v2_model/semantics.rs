use std::collections::{BTreeMap, BTreeSet};

use fenestra_ui_authoring::prototype::{
    SemanticArtifactErrorKindV2, SemanticArtifactLimitKindV2, SemanticArtifactLimitsV2,
    canonical_semantics_v2,
};

use crate::support;

const LABELS: [&str; 30] = [
    "document",
    "schema",
    "component",
    "property",
    "construction",
    "template",
    "initial-property",
    "static-child",
    "region-child",
    "region",
    "initial-key",
    "style",
    "style-assignment",
    "spatial",
    "resources",
    "image",
    "spatial-node",
    "spatial-container",
    "spatial-placement",
    "spatial-transform",
    "spatial-field",
    "spatial-shape",
    "spatial-path-verb",
    "spatial-polygon-point",
    "spatial-brush",
    "spatial-gradient-stop",
    "spatial-clip",
    "spatial-paint",
    "spatial-hit",
    "spatial-semantic",
];

#[test]
fn semantic_artifact_is_frontend_independent_deterministic_and_exactly_framed() {
    let (fen, ui) = support::compile_both();
    let outputs = [artifact(&fen), artifact(&fen), artifact(&ui), artifact(&ui)];
    for output in &outputs[1..] {
        assert_eq!(output.as_bytes(), outputs[0].as_bytes());
    }
    let text = outputs[0].as_str();
    assert!(text.is_ascii());
    assert!(!text.contains('\r'));
    assert!(text.ends_with('\n'));
    assert!(!text[..text.len() - 1].ends_with('\n'));
    assert_eq!(
        text.lines().next(),
        Some(concat!(
            "fenestra-authoring-semantics|2|authoring-format=2|schema-format=1|",
            "construction-format=1|style-format=1|spatial-format=2|records=380"
        ))
    );
    assert_eq!(text.lines().count(), 381);
}

#[test]
fn every_record_has_dense_anchor_span_and_a_closed_label() {
    let compiled = support::compile_fen(support::FIXTURE);
    let output = artifact(&compiled);
    let records = records(output.as_str());
    assert_eq!(records.len(), 380);
    let allowed = LABELS.into_iter().collect::<BTreeSet<_>>();
    let observed = records
        .iter()
        .map(|record| record.label)
        .collect::<BTreeSet<_>>();
    assert_eq!(observed, allowed);
    for (ordinal, record) in records.iter().enumerate() {
        assert_eq!(record.anchor, ordinal);
        assert_eq!(record.span, (ordinal, ordinal + 1));
        assert!(!record.payload.is_empty());
        assert!(!record.payload.contains('\n'));
    }
}

#[test]
fn spatial_field_value_families_owner_scopes_and_image_bytes_are_complete() {
    let compiled = support::compile_fen(support::FIXTURE);
    let output = artifact(&compiled);
    let records = records(output.as_str());
    let labels = records
        .iter()
        .map(|record| (record.anchor, record.label))
        .collect::<BTreeMap<_, _>>();
    let fields = records
        .iter()
        .filter(|record| record.label == "spatial-field")
        .collect::<Vec<_>>();
    assert_eq!(fields.len(), 264);

    let mut families = BTreeSet::new();
    for field in fields {
        let owner = field
            .payload_value("owner")
            .parse::<usize>()
            .expect("field owner anchor");
        assert!(
            matches!(
                labels.get(&owner).copied(),
                Some(
                    "image"
                        | "spatial-node"
                        | "spatial-container"
                        | "spatial-placement"
                        | "spatial-transform"
                        | "spatial-shape"
                        | "spatial-path-verb"
                        | "spatial-polygon-point"
                        | "spatial-brush"
                        | "spatial-gradient-stop"
                        | "spatial-clip"
                        | "spatial-paint"
                        | "spatial-hit"
                        | "spatial-semantic"
                )
            ),
            "invalid field owner: {}",
            field.payload
        );
        let value = field.payload_value("value");
        let family =
            if value == "input-policy-literal:accept" || value == "input-policy-literal:ignore" {
                value
            } else {
                value.split(':').next().expect("field value family")
            };
        families.insert(family);
    }
    assert_eq!(
        families,
        [
            "brush",
            "clip",
            "fixed16-literal",
            "fixed16-property",
            "i32",
            "i32-literal",
            "i32-property",
            "image",
            "input-policy-literal:accept",
            "input-policy-literal:ignore",
            "input-policy-property",
            "node",
            "rgba8-literal",
            "rgba8-property",
            "shape",
            "template",
            "u16",
            "u32",
            "u8",
        ]
        .into_iter()
        .collect()
    );

    let image = records
        .iter()
        .find(|record| record.label == "image")
        .expect("one image record");
    assert_eq!(
        image.payload,
        "order=0|name=checker|bytes=hex:16:ff0000ff008000800000404000000000"
    );
}

#[test]
fn all_spatial_tables_retain_their_authored_order_and_discriminants() {
    let compiled = support::compile_fen(support::FIXTURE);
    let output = artifact(&compiled);
    let records = records(output.as_str());
    assert_payloads(
        &records,
        "spatial-node",
        &[
            "order=0|name=scene|parent=viewport",
            "order=1|name=stack_node|parent=node",
            "order=2|name=floating|parent=node",
            "order=3|name=floating_child|parent=node",
            "order=4|name=tile_node|parent=node",
            "order=5|name=guide|parent=node",
            "order=6|name=viewport_layer|parent=node",
        ],
    );
    assert_payloads(
        &records,
        "spatial-shape",
        &[
            "node=0|order=0|name=frame|kind=rect",
            "node=0|order=1|name=dot|kind=circle",
            "node=0|order=2|name=tri|kind=polygon",
            "node=0|order=3|name=curve|kind=path",
            "node=4|order=0|name=frame|kind=circle",
        ],
    );
    assert_payloads(
        &records,
        "spatial-path-verb",
        &[
            "node=0|shape=3|order=0|kind=move-to",
            "node=0|shape=3|order=1|kind=line-to",
            "node=0|shape=3|order=2|kind=quadratic-to",
            "node=0|shape=3|order=3|kind=cubic-to",
            "node=0|shape=3|order=4|kind=close",
        ],
    );
    assert_payloads(
        &records,
        "spatial-brush",
        &[
            "node=0|order=0|name=flat|kind=solid",
            "node=0|order=1|name=fade|kind=linear-gradient",
            "node=4|order=0|name=flat|kind=solid",
        ],
    );
    assert_payloads(
        &records,
        "spatial-polygon-point",
        &[
            "node=0|shape=2|order=0",
            "node=0|shape=2|order=1",
            "node=0|shape=2|order=2",
        ],
    );
    assert_payloads(
        &records,
        "spatial-gradient-stop",
        &[
            "node=0|brush=1|order=0",
            "node=0|brush=1|order=1",
            "node=0|brush=1|order=2",
        ],
    );
    assert_payloads(
        &records,
        "spatial-clip",
        &[
            "node=0|order=0|name=outer|parent=none|fill-rule=non-zero",
            "node=0|order=1|name=inner|parent=qualified|fill-rule=even-odd",
            "node=4|order=0|name=local|parent=qualified|fill-rule=non-zero",
        ],
    );
    assert_payloads(
        &records,
        "spatial-paint",
        &[
            "node=0|order=0|kind=coverage|coverage=fill|fill-rule=non-zero|clip=qualified",
            "node=0|order=1|kind=coverage|coverage=round-stroke|clip=none",
            "node=0|order=2|kind=image|clip=qualified",
            "node=4|order=0|kind=coverage|coverage=fill|fill-rule=non-zero|clip=qualified",
        ],
    );
    assert_payloads(
        &records,
        "spatial-hit",
        &[
            "node=0|order=0|coverage=fill|fill-rule=even-odd|clip=qualified",
            "node=0|order=1|coverage=round-stroke|clip=none",
            "node=0|order=2|coverage=fill|fill-rule=non-zero|clip=none",
            "node=4|order=0|coverage=round-stroke|clip=qualified",
        ],
    );
    assert_payloads(
        &records,
        "spatial-semantic",
        &[
            "node=0|order=0|fill-rule=non-zero|clip=qualified",
            "node=0|order=1|fill-rule=even-odd|clip=none",
            "node=4|order=0|fill-rule=even-odd|clip=qualified",
        ],
    );
}

#[test]
fn all_three_semantic_bounds_are_inclusive_one_under_and_priority_ordered() {
    let compiled = support::compile_fen(support::FIXTURE);
    let baseline = artifact(&compiled);
    let artifact_bytes = baseline.as_bytes().len();
    let line_bytes = baseline.as_str().lines().map(str::len).max().unwrap();
    let record_count = baseline.as_str().lines().count() - 1;
    let exact = SemanticArtifactLimitsV2::new(artifact_bytes, line_bytes, record_count);
    assert_eq!(
        canonical_semantics_v2(&compiled, exact)
            .expect("exact semantic bounds must pass")
            .as_bytes(),
        baseline.as_bytes()
    );
    let cases = [
        (
            SemanticArtifactLimitsV2::new(artifact_bytes, line_bytes, record_count - 1),
            SemanticArtifactLimitKindV2::Records,
        ),
        (
            SemanticArtifactLimitsV2::new(artifact_bytes, line_bytes - 1, record_count),
            SemanticArtifactLimitKindV2::LineBytes,
        ),
        (
            SemanticArtifactLimitsV2::new(artifact_bytes - 1, line_bytes, record_count),
            SemanticArtifactLimitKindV2::ArtifactBytes,
        ),
    ];
    for (limits, kind) in cases {
        let error = canonical_semantics_v2(&compiled, limits).expect_err("one under must fail");
        assert_eq!(
            error.kind(),
            SemanticArtifactErrorKindV2::LimitExceeded(kind)
        );
    }
    let all = canonical_semantics_v2(&compiled, SemanticArtifactLimitsV2::new(0, 0, 0))
        .expect_err("records must win simultaneous crossings");
    assert_eq!(
        all.kind(),
        SemanticArtifactErrorKindV2::LimitExceeded(SemanticArtifactLimitKindV2::Records)
    );
    let line = canonical_semantics_v2(&compiled, SemanticArtifactLimitsV2::new(0, 0, record_count))
        .expect_err("line bytes must precede artifact bytes");
    assert_eq!(
        line.kind(),
        SemanticArtifactErrorKindV2::LimitExceeded(SemanticArtifactLimitKindV2::LineBytes)
    );
}

fn artifact(
    compiled: &fenestra_ui_authoring::prototype::CompiledAuthoringV2,
) -> fenestra_ui_authoring::prototype::SemanticArtifactV2 {
    canonical_semantics_v2(compiled, support::semantic_limits()).expect("semantic artifact")
}

fn assert_payloads(records: &[Record<'_>], label: &str, expected: &[&str]) {
    assert_eq!(payloads(records, label), expected);
}

fn payloads<'a>(records: &'a [Record<'a>], label: &str) -> Vec<&'a str> {
    records
        .iter()
        .filter(|record| record.label == label)
        .map(|record| record.payload)
        .collect()
}

struct Record<'a> {
    anchor: usize,
    label: &'a str,
    span: (usize, usize),
    payload: &'a str,
}

impl Record<'_> {
    fn payload_value(&self, key: &str) -> &str {
        self.payload
            .split('|')
            .find_map(|part| {
                part.strip_prefix(key)
                    .and_then(|value| value.strip_prefix('='))
            })
            .unwrap_or_else(|| panic!("missing {key} in {}", self.payload))
    }
}

fn records(source: &str) -> Vec<Record<'_>> {
    source
        .lines()
        .skip(1)
        .map(|line| {
            let mut parts = line.splitn(5, '|');
            assert_eq!(parts.next(), Some("record"));
            let anchor = parts.next().unwrap().parse::<usize>().unwrap();
            let label = parts.next().unwrap();
            let span = parts.next().unwrap().strip_prefix("span=").unwrap();
            let (start, end) = span.split_once(':').unwrap();
            Record {
                anchor,
                label,
                span: (start.parse().unwrap(), end.parse().unwrap()),
                payload: parts.next().unwrap(),
            }
        })
        .collect()
}
