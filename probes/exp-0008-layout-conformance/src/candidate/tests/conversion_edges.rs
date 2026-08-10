use fenestra_ui_layout::prototype::{LayoutAxisV1::Column, LayoutErrorLocationV1, LayoutExtentV1};

use crate::candidate::{
    CandidateEdgeV1, CandidateProfileErrorFieldV1 as Field, CandidateProfileErrorKindV1 as Kind,
    convert_candidate_output_v1,
};

use super::support::{assert_profile_error, bounds, fixed_node, raw};

#[test]
fn candidate_output_edge_ceiling_is_inclusive_for_near_and_far_edges() {
    let nodes = [fixed_node(0, None, Column, 0, 0)];

    let near = convert_candidate_output_v1(&nodes, &[raw(0, 524_288.0, 524_288.0, 0.0, 0.0)])
        .expect("near edges exactly at 524288 must pass");
    assert_eq!(bounds(near.records()[0]), [524_288, 524_288, 0, 0]);

    let far = convert_candidate_output_v1(&nodes, &[raw(0, 0.0, 0.0, 524_288.0, 524_288.0)])
        .expect("far edges exactly at 524288 must pass");
    assert_eq!(bounds(far.records()[0]), [0, 0, 524_288, 524_288]);
}

#[test]
fn every_edge_one_over_reports_extent_edge_and_record_ordinal() {
    let nodes = [fixed_node(0, None, Column, 0, 0)];
    let cases = [
        (
            raw(0, 524_288.25, 0.0, 0.0, 0.0),
            Field::OutputEdge {
                extent: LayoutExtentV1::Width,
                edge: CandidateEdgeV1::Near,
            },
        ),
        (
            raw(0, 0.0, 0.0, 524_288.25, 0.0),
            Field::OutputEdge {
                extent: LayoutExtentV1::Width,
                edge: CandidateEdgeV1::Far,
            },
        ),
        (
            raw(0, 0.0, 524_288.25, 0.0, 0.0),
            Field::OutputEdge {
                extent: LayoutExtentV1::Height,
                edge: CandidateEdgeV1::Near,
            },
        ),
        (
            raw(0, 0.0, 0.0, 0.0, 524_288.25),
            Field::OutputEdge {
                extent: LayoutExtentV1::Height,
                edge: CandidateEdgeV1::Far,
            },
        ),
    ];

    for (changed, field) in cases {
        assert_profile_error(
            convert_candidate_output_v1(&nodes, &[changed]),
            Kind::OutputEdgeLimit,
            field,
            LayoutErrorLocationV1::OutputRecord { index: 0 },
        );
    }
}

#[test]
fn absolute_edges_are_screened_x_near_far_then_y_near_far() {
    let nodes = [fixed_node(0, None, Column, 0, 0)];

    assert_profile_error(
        convert_candidate_output_v1(&nodes, &[raw(0, 524_288.25, 524_288.25, 1.0, 1.0)]),
        Kind::OutputEdgeLimit,
        Field::OutputEdge {
            extent: LayoutExtentV1::Width,
            edge: CandidateEdgeV1::Near,
        },
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );
    assert_profile_error(
        convert_candidate_output_v1(&nodes, &[raw(0, 524_288.0, 524_288.25, 0.25, 1.0)]),
        Kind::OutputEdgeLimit,
        Field::OutputEdge {
            extent: LayoutExtentV1::Width,
            edge: CandidateEdgeV1::Far,
        },
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );
    assert_profile_error(
        convert_candidate_output_v1(&nodes, &[raw(0, 0.0, 524_288.25, 0.0, 1.0)]),
        Kind::OutputEdgeLimit,
        Field::OutputEdge {
            extent: LayoutExtentV1::Height,
            edge: CandidateEdgeV1::Near,
        },
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );
    assert_profile_error(
        convert_candidate_output_v1(&nodes, &[raw(0, 0.0, 524_288.0, 0.0, 0.25)]),
        Kind::OutputEdgeLimit,
        Field::OutputEdge {
            extent: LayoutExtentV1::Height,
            edge: CandidateEdgeV1::Far,
        },
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );
}

#[test]
fn edge_screening_is_record_major_before_axis_priority() {
    let nodes = [
        fixed_node(0, None, Column, 0, 0),
        fixed_node(1, Some(0), Column, 0, 0),
    ];
    let records = [
        raw(0, 0.0, 0.0, 0.0, 524_288.25),
        raw(1, 524_288.25, 0.0, 0.0, 0.0),
    ];

    assert_profile_error(
        convert_candidate_output_v1(&nodes, &records),
        Kind::OutputEdgeLimit,
        Field::OutputEdge {
            extent: LayoutExtentV1::Height,
            edge: CandidateEdgeV1::Far,
        },
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );
}

#[test]
fn parent_relative_origins_are_accumulated_before_edge_screening() {
    let nodes = [
        fixed_node(0, None, Column, 0, 0),
        fixed_node(1, Some(0), Column, 0, 0),
    ];
    let records = [
        raw(0, 524_000.0, 0.0, 100.0, 0.0),
        raw(1, 300.0, 0.0, 0.0, 0.0),
    ];

    assert_profile_error(
        convert_candidate_output_v1(&nodes, &records),
        Kind::OutputEdgeLimit,
        Field::OutputEdge {
            extent: LayoutExtentV1::Width,
            edge: CandidateEdgeV1::Near,
        },
        LayoutErrorLocationV1::OutputRecord { index: 1 },
    );
}

#[test]
fn cumulative_near_and_far_edges_round_once_with_halves_away_from_zero() {
    let nodes = [
        fixed_node(0, None, Column, 0, 0),
        fixed_node(1, Some(0), Column, 0, 0),
    ];
    let records = [raw(0, 0.4, 0.5, 10.0, 10.0), raw(1, 0.2, 0.0, 1.2, 1.0)];

    let output = convert_candidate_output_v1(&nodes, &records)
        .expect("bounded fractional edges must convert");
    assert_eq!(bounds(output.records()[0]), [0, 1, 10, 10]);
    assert_eq!(bounds(output.records()[1]), [1, 1, 1, 1]);
}

#[test]
fn raw_edge_one_over_is_rejected_before_rounding_back_to_the_ceiling() {
    let nodes = [fixed_node(0, None, Column, 0, 0)];

    assert_profile_error(
        convert_candidate_output_v1(&nodes, &[raw(0, 524_288.0, 0.0, 0.25, 0.0)]),
        Kind::OutputEdgeLimit,
        Field::OutputEdge {
            extent: LayoutExtentV1::Width,
            edge: CandidateEdgeV1::Far,
        },
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );
}
