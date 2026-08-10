use fenestra_ui_layout::prototype::{
    LayoutAxisV1::Column, LayoutEngineErrorKindV1, LayoutErrorLocationV1,
};

use crate::candidate::{
    convert_candidate_output_v1, map_candidate_profile_error_v1, validate_candidate_input_v1,
};

use super::support::{dimension, fixed, fixed_node, node, padding, raw, viewport};

#[test]
fn coordinate_limit_maps_to_neutral_rejected_input() {
    let nodes = [node(
        0,
        None,
        Column,
        dimension(0, 0, 4097),
        fixed(0),
        padding(0, 0, 0, 0),
        0,
    )];
    let profile = validate_candidate_input_v1(viewport(0, 0), &nodes)
        .expect_err("candidate input must exceed its profile");
    let mapped = map_candidate_profile_error_v1(profile);

    assert_eq!(mapped.kind(), LayoutEngineErrorKindV1::RejectedInput);
    assert_eq!(
        mapped.location(),
        LayoutErrorLocationV1::InputNode { index: 0 }
    );
}

#[test]
fn every_output_profile_class_maps_to_neutral_unrepresentable_output() {
    let nodes = [fixed_node(0, None, Column, 0, 0)];
    let cases = [
        raw(0, f32::NAN, 0.0, 0.0, 0.0),
        raw(0, -1.0, 0.0, 0.0, 0.0),
        raw(0, 524_288.25, 0.0, 0.0, 0.0),
    ];

    for raw_record in cases {
        let profile = convert_candidate_output_v1(&nodes, &[raw_record])
            .expect_err("candidate output must violate its profile");
        let mapped = map_candidate_profile_error_v1(profile);
        assert_eq!(
            mapped.kind(),
            LayoutEngineErrorKindV1::UnrepresentableOutput
        );
        assert_eq!(
            mapped.location(),
            LayoutErrorLocationV1::OutputRecord { index: 0 }
        );
    }
}
