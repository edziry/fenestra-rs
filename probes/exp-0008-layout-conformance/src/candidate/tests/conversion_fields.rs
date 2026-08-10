use fenestra_ui_layout::prototype::{
    LayoutAxisV1::Column, LayoutErrorLocationV1, LayoutOutputFieldV1,
};

use crate::candidate::{
    CandidateProfileErrorFieldV1 as Field, CandidateProfileErrorKindV1 as Kind,
    convert_candidate_output_v1,
};

use super::support::{assert_profile_error, fixed_node, raw};

#[test]
fn every_raw_field_rejects_non_finite_values_at_its_record_ordinal() {
    let nodes = [
        fixed_node(0, None, Column, 10, 10),
        fixed_node(1, Some(0), Column, 2, 2),
    ];
    let cases = [
        (
            raw(1, f32::NAN, f32::INFINITY, f32::NAN, f32::INFINITY),
            LayoutOutputFieldV1::X,
        ),
        (
            raw(1, 0.0, f32::INFINITY, f32::NAN, f32::INFINITY),
            LayoutOutputFieldV1::Y,
        ),
        (
            raw(1, 0.0, 0.0, f32::NAN, f32::INFINITY),
            LayoutOutputFieldV1::Width,
        ),
        (
            raw(1, 0.0, 0.0, 2.0, f32::INFINITY),
            LayoutOutputFieldV1::Height,
        ),
    ];

    for (changed, field) in cases {
        let records = [raw(0, 0.0, 0.0, 10.0, 10.0), changed];
        assert_profile_error(
            convert_candidate_output_v1(&nodes, &records),
            Kind::NonFiniteOutput,
            Field::Output(field),
            LayoutErrorLocationV1::OutputRecord { index: 1 },
        );
    }
}

#[test]
fn non_finite_phase_is_global_and_uses_record_field_order() {
    let nodes = [
        fixed_node(0, None, Column, 10, 10),
        fixed_node(1, Some(0), Column, 2, 2),
    ];
    let records = [
        raw(0, -1.0, 0.0, 10.0, 10.0),
        raw(1, 0.0, 0.0, 2.0, f32::NAN),
    ];
    assert_profile_error(
        convert_candidate_output_v1(&nodes, &records),
        Kind::NonFiniteOutput,
        Field::Output(LayoutOutputFieldV1::Height),
        LayoutErrorLocationV1::OutputRecord { index: 1 },
    );

    let same_record = [raw(0, f32::NAN, f32::INFINITY, f32::NAN, f32::INFINITY)];
    assert_profile_error(
        convert_candidate_output_v1(&nodes[..1], &same_record),
        Kind::NonFiniteOutput,
        Field::Output(LayoutOutputFieldV1::X),
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );

    let record_major = [
        raw(0, 0.0, 0.0, 10.0, f32::NAN),
        raw(1, f32::NAN, 0.0, 2.0, 2.0),
    ];
    assert_profile_error(
        convert_candidate_output_v1(&nodes, &record_major),
        Kind::NonFiniteOutput,
        Field::Output(LayoutOutputFieldV1::Height),
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );
}

#[test]
fn every_raw_field_rejects_negative_values_at_its_record_ordinal() {
    let nodes = [
        fixed_node(0, None, Column, 10, 10),
        fixed_node(1, Some(0), Column, 2, 2),
    ];
    let cases = [
        (raw(1, -0.25, -0.25, -0.25, -0.25), LayoutOutputFieldV1::X),
        (raw(1, 0.0, -0.25, -0.25, -0.25), LayoutOutputFieldV1::Y),
        (raw(1, 0.0, 0.0, -0.25, -0.25), LayoutOutputFieldV1::Width),
        (raw(1, 0.0, 0.0, 2.0, -0.25), LayoutOutputFieldV1::Height),
    ];

    for (changed, field) in cases {
        let records = [raw(0, 0.0, 0.0, 10.0, 10.0), changed];
        assert_profile_error(
            convert_candidate_output_v1(&nodes, &records),
            Kind::NegativeOutput,
            Field::Output(field),
            LayoutErrorLocationV1::OutputRecord { index: 1 },
        );
    }
}

#[test]
fn negative_phase_uses_record_then_field_order() {
    let nodes = [
        fixed_node(0, None, Column, 10, 10),
        fixed_node(1, Some(0), Column, 2, 2),
    ];
    let records = [raw(0, 0.0, 0.0, 10.0, -1.0), raw(1, -1.0, 0.0, 2.0, 2.0)];
    assert_profile_error(
        convert_candidate_output_v1(&nodes, &records),
        Kind::NegativeOutput,
        Field::Output(LayoutOutputFieldV1::Height),
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );

    let same_record = [raw(0, -1.0, -1.0, -1.0, -1.0)];
    assert_profile_error(
        convert_candidate_output_v1(&nodes[..1], &same_record),
        Kind::NegativeOutput,
        Field::Output(LayoutOutputFieldV1::X),
        LayoutErrorLocationV1::OutputRecord { index: 0 },
    );

    let negative_after_edge = [
        raw(0, 524_288.25, 0.0, 10.0, 10.0),
        raw(1, 0.0, 0.0, 2.0, -1.0),
    ];
    assert_profile_error(
        convert_candidate_output_v1(&nodes, &negative_after_edge),
        Kind::NegativeOutput,
        Field::Output(LayoutOutputFieldV1::Height),
        LayoutErrorLocationV1::OutputRecord { index: 1 },
    );
}
