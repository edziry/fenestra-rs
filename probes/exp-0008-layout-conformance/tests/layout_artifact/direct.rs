#[path = "direct/faults.rs"]
mod faults;

use self::faults::{apply_fault, direct_faults};
use fenestra_ui_exp_0008_layout_conformance::prototype::{
    TaffyStackEngineV1, compare_layout_records_v1, registered_layout_corpus_v1,
};
use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutInputV1, LayoutNodeV1, LayoutRecordV1, LayoutViewportV1,
    REGISTERED_LAYOUT_LIMITS_V1, ReferenceStackEngineV1, compute_layout_v1,
};

const CASE_NAMES: [&str; 23] = [
    "single-fixed-root",
    "column-two",
    "row-two",
    "nested-row-in-column",
    "asymmetric-padding",
    "column-gap-three",
    "row-gap-three",
    "clamp-below",
    "clamp-above",
    "mixed-constraints-padding",
    "main-axis-overflow",
    "cross-axis-overflow",
    "padding-equal-box",
    "zero-width-child",
    "zero-height-gap",
    "zero-width-viewport",
    "zero-height-viewport",
    "zero-by-zero-viewport",
    "large-integer-padding-gap",
    "node-ceiling",
    "child-ceiling",
    "depth-ceiling",
    "registered-runtime-fixture",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DirectArtifactV1 {
    cases: Vec<DirectCaseV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DirectCaseV1 {
    name: String,
    viewport: LayoutViewportV1,
    input: Vec<LayoutNodeV1>,
    outputs: Vec<DirectOutputV1>,
    classification: DirectClassificationV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectOutputV1 {
    oracle: LayoutRecordV1,
    reference: LayoutRecordV1,
    candidate: LayoutRecordV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectClassificationV1 {
    Pass,
    Adapt,
    Stop,
}

impl DirectClassificationV1 {
    const ALL: [Self; 3] = [Self::Pass, Self::Adapt, Self::Stop];
}

pub(super) fn assert_direct_artifact_contract(render: impl Fn(&DirectArtifactV1) -> Vec<u8>) {
    let canonical = registered_direct_artifact_v1();
    assert_canonical_model(&canonical);
    assert_eq!(canonical, registered_direct_artifact_v1());
    let canonical_lines = direct_lines_v1(&canonical);
    assert_direct_lines(&canonical_lines);

    let retained_model = canonical.clone();
    let canonical_slice = render(&canonical);
    let retained_slice = canonical_slice.clone();
    let mut fault_slices = Vec::new();
    for fault in direct_faults() {
        let mut changed = canonical.clone();
        apply_fault(&mut changed, fault);
        assert_ne!(changed, canonical, "{fault:?} must change the typed model");
        assert_ne!(
            direct_lines_v1(&changed),
            canonical_lines,
            "{fault:?} must change the rendered rows"
        );
        fault_slices.push((fault, render(&changed)));
        assert_eq!(
            canonical, retained_model,
            "{fault:?} changed the source model"
        );
        assert_eq!(
            canonical_slice, retained_slice,
            "{fault:?} changed a prior slice"
        );
    }

    assert_eq!(canonical_slice, render(&canonical));
    assert!(canonical_slice.ends_with(b"\n"));
    assert!(!canonical_slice.ends_with(b"\n\n"));
    assert!(
        canonical_slice
            .iter()
            .all(|byte| *byte == b'\n' || (0x20..=0x7e).contains(byte))
    );
    for (fault, slice) in fault_slices {
        assert_ne!(
            slice, canonical_slice,
            "{fault:?} must re-encode differently"
        );
    }
}

pub(super) fn direct_lines_v1(model: &DirectArtifactV1) -> Vec<String> {
    let mut lines = Vec::with_capacity(286);
    for (case_index, case) in model.cases.iter().enumerate() {
        lines.push(format!(
            "case|index={case_index}|name={}|viewport={},{}|nodes={}",
            case.name,
            case.viewport.width(),
            case.viewport.height(),
            case.input.len()
        ));
        for (ordinal, node) in case.input.iter().copied().enumerate() {
            lines.push(input_line(case_index, ordinal, node));
        }
        for (ordinal, output) in case.outputs.iter().copied().enumerate() {
            lines.push(format!(
                "case-output|case={case_index}|ordinal={ordinal}|oracle={}|reference={}|candidate={}",
                record_fields(output.oracle),
                record_fields(output.reference),
                record_fields(output.candidate)
            ));
        }
        lines.push(format!(
            "case-result|case={case_index}|classification={}",
            classification_label(case.classification)
        ));
    }
    lines
}

fn input_line(case_index: usize, ordinal: usize, node: LayoutNodeV1) -> String {
    let style = node.style();
    let width = style.width();
    let height = style.height();
    let padding = style.padding();
    format!(
        "case-input|case={case_index}|ordinal={ordinal}|key={}|parent={}|axis={}|width={},{},{}|height={},{},{}|padding={},{},{},{}|gap={}",
        node.key().get(),
        parent_label(node),
        axis_label(style.axis()),
        width.minimum(),
        width.preferred(),
        width.maximum(),
        height.minimum(),
        height.preferred(),
        height.maximum(),
        padding.left(),
        padding.right(),
        padding.top(),
        padding.bottom(),
        style.gap()
    )
}

fn parent_label(node: LayoutNodeV1) -> String {
    match node.parent() {
        Some(parent) => parent.get().to_string(),
        None => "-".to_owned(),
    }
}

const fn axis_label(axis: LayoutAxisV1) -> &'static str {
    match axis {
        LayoutAxisV1::Row => "row",
        LayoutAxisV1::Column => "column",
    }
}

fn record_fields(record: LayoutRecordV1) -> String {
    let bounds = record.bounds();
    format!(
        "{},{},{},{},{}",
        record.key().get(),
        bounds.x(),
        bounds.y(),
        bounds.width(),
        bounds.height()
    )
}

const fn classification_label(classification: DirectClassificationV1) -> &'static str {
    match classification {
        DirectClassificationV1::Pass => "pass",
        DirectClassificationV1::Adapt => "adapt",
        DirectClassificationV1::Stop => "stop",
    }
}

fn assert_direct_lines(lines: &[String]) {
    assert_eq!(lines.len(), 286);
    assert_eq!(
        lines[0],
        "case|index=0|name=single-fixed-root|viewport=7,5|nodes=1"
    );
    assert_eq!(
        lines[1],
        "case-input|case=0|ordinal=0|key=0|parent=-|axis=column|width=31,31,31|height=19,19,19|padding=0,0,0,0|gap=0"
    );
    assert_eq!(
        lines[2],
        "case-output|case=0|ordinal=0|oracle=0,0,0,31,19|reference=0,0,0,31,19|candidate=0,0,0,31,19"
    );
    assert_eq!(lines[3], "case-result|case=0|classification=pass");
    assert!(lines.contains(&"case-input|case=9|ordinal=0|key=0|parent=-|axis=column|width=50,60,70|height=40,50,60|padding=6,7,4,5|gap=3".to_owned()));
}

pub(super) fn registered_direct_artifact_v1() -> DirectArtifactV1 {
    let reference = ReferenceStackEngineV1::new();
    let candidate = TaffyStackEngineV1::new();
    let cases = registered_layout_corpus_v1()
        .into_iter()
        .map(|registered| {
            let input_nodes = registered.nodes().to_vec();
            let oracle = registered.expected_records().to_vec();
            let input = LayoutInputV1::new(registered.viewport(), &input_nodes);
            let reference = compute_layout_v1(&reference, input, REGISTERED_LAYOUT_LIMITS_V1)
                .unwrap_or_else(|error| {
                    panic!("{}: reference failed: {error:?}", registered.name())
                });
            let candidate = compute_layout_v1(&candidate, input, REGISTERED_LAYOUT_LIMITS_V1)
                .unwrap_or_else(|error| {
                    panic!("{}: candidate failed: {error:?}", registered.name())
                });
            assert_lane_match(registered.name(), "reference", &oracle, reference.records());
            assert_lane_match(registered.name(), "candidate", &oracle, candidate.records());
            assert_lane_match(
                registered.name(),
                "cross-lane",
                reference.records(),
                candidate.records(),
            );
            assert_eq!(
                registered.nodes(),
                input_nodes,
                "{}: input changed",
                registered.name()
            );
            let outputs = oracle
                .into_iter()
                .zip(reference.records().iter().copied())
                .zip(candidate.records().iter().copied())
                .map(|((oracle, reference), candidate)| DirectOutputV1 {
                    oracle,
                    reference,
                    candidate,
                })
                .collect();
            DirectCaseV1 {
                name: registered.name().to_owned(),
                viewport: registered.viewport(),
                input: input_nodes,
                outputs,
                classification: DirectClassificationV1::Pass,
            }
        })
        .collect();
    DirectArtifactV1 { cases }
}

fn assert_lane_match(
    case: &str,
    lane: &str,
    expected: &[LayoutRecordV1],
    actual: &[LayoutRecordV1],
) {
    assert_eq!(actual.len(), expected.len(), "{case}: {lane} count");
    assert!(
        compare_layout_records_v1(expected, actual).is_none(),
        "{case}: {lane} order or field mismatch"
    );
}

fn assert_canonical_model(model: &DirectArtifactV1) {
    assert_eq!(
        DirectClassificationV1::ALL,
        [
            DirectClassificationV1::Pass,
            DirectClassificationV1::Adapt,
            DirectClassificationV1::Stop,
        ]
    );
    assert_eq!(model.cases.len(), CASE_NAMES.len());
    assert_eq!(
        model
            .cases
            .iter()
            .map(|case| case.name.as_str())
            .collect::<Vec<_>>(),
        CASE_NAMES
    );
    assert_eq!(
        model
            .cases
            .iter()
            .map(|case| case.input.len())
            .sum::<usize>(),
        120
    );
    assert_eq!(
        model
            .cases
            .iter()
            .map(|case| case.outputs.len())
            .sum::<usize>(),
        120
    );
    for case in &model.cases {
        assert_eq!(
            case.input.len(),
            case.outputs.len(),
            "{}: row count",
            case.name
        );
        assert_eq!(case.classification, DirectClassificationV1::Pass);
        for (node, output) in case.input.iter().zip(&case.outputs) {
            assert_eq!(
                output.oracle.key(),
                node.key(),
                "{}: output order",
                case.name
            );
            assert_eq!(output.reference, output.oracle, "{}: reference", case.name);
            assert_eq!(output.candidate, output.oracle, "{}: candidate", case.name);
        }
    }
}
