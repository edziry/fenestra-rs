use fenestra_ui_layout::prototype::{
    LayoutErrorKindV1, LayoutErrorLocationV1, LayoutInputErrorKindV1, LayoutOutputErrorKindV1,
};

use super::placement_execution_support::{
    expect_layout, fixture, layout, limits, root, start_free,
};
use crate::error::SpatialErrorLocationV2;
use crate::model::SpatialAnchorTargetV2;
use crate::resolve_error::SpatialLayoutErrorKindV2;

#[test]
fn every_engine_kind_maps_global_valid_record_and_untrusted_record_locations() {
    let fixture = mapping_fixture();
    let graph = prepare_dependency_graph!(
        &fixture,
        super::placement_execution_support::VIEWPORT,
        limits()
    )
    .expect("mapping fixture reaches the dry graph");
    let cases = [
        (
            LayoutErrorLocationV1::Input,
            SpatialErrorLocationV2::Island { index: 1 },
        ),
        (
            LayoutErrorLocationV1::Viewport,
            SpatialErrorLocationV2::Island { index: 1 },
        ),
        (
            LayoutErrorLocationV1::Output,
            SpatialErrorLocationV2::Island { index: 1 },
        ),
        (
            LayoutErrorLocationV1::InputNode { index: 0 },
            SpatialErrorLocationV2::Node { index: 3 },
        ),
        (
            LayoutErrorLocationV1::OutputRecord { index: 0 },
            SpatialErrorLocationV2::Node { index: 3 },
        ),
        (
            LayoutErrorLocationV1::InputNode { index: 1 },
            SpatialErrorLocationV2::Node { index: 4 },
        ),
        (
            LayoutErrorLocationV1::OutputRecord { index: 1 },
            SpatialErrorLocationV2::Node { index: 4 },
        ),
        (
            LayoutErrorLocationV1::InputNode { index: 2 },
            SpatialErrorLocationV2::Node { index: 6 },
        ),
        (
            LayoutErrorLocationV1::OutputRecord { index: 2 },
            SpatialErrorLocationV2::Node { index: 6 },
        ),
        (
            LayoutErrorLocationV1::InputNode { index: u32::MAX },
            SpatialErrorLocationV2::Island { index: 1 },
        ),
        (
            LayoutErrorLocationV1::OutputRecord { index: u32::MAX },
            SpatialErrorLocationV2::Island { index: 1 },
        ),
    ];

    for kind in fenestra_ui_layout::prototype::LayoutEngineErrorKindV1::ALL {
        for (location, expected) in cases {
            let error =
                map_layout_execution_error!(graph, 1, LayoutErrorKindV1::Engine(kind), location);
            expect_layout(
                Err::<(), _>(error),
                SpatialLayoutErrorKindV2::Engine(kind),
                expected,
            );
        }
    }
}

#[test]
fn every_input_kind_and_location_is_an_execution_bridge_invariant() {
    let fixture = mapping_fixture();
    let graph = prepare_dependency_graph!(
        &fixture,
        super::placement_execution_support::VIEWPORT,
        limits()
    )
    .expect("mapping fixture reaches the dry graph");
    let locations = bridge_locations();

    for kind in LayoutInputErrorKindV1::ALL {
        for location in locations {
            let error =
                map_layout_execution_error!(graph, 1, LayoutErrorKindV1::Input(kind), location);
            expect_layout(
                Err::<(), _>(error),
                SpatialLayoutErrorKindV2::BridgeInvariant,
                SpatialErrorLocationV2::Island { index: 1 },
            );
        }
    }
}

#[test]
fn output_kinds_accept_only_locations_the_neutral_validator_can_author() {
    let fixture = mapping_fixture();
    let graph = prepare_dependency_graph!(
        &fixture,
        super::placement_execution_support::VIEWPORT,
        limits()
    )
    .expect("mapping fixture reaches the dry graph");

    for kind in LayoutOutputErrorKindV1::ALL {
        for location in bridge_locations() {
            let valid = match (kind, location) {
                (LayoutOutputErrorKindV1::RecordCountMismatch, LayoutErrorLocationV1::Output) => {
                    Some(SpatialErrorLocationV2::Island { index: 1 })
                }
                (
                    LayoutOutputErrorKindV1::KeyMismatch
                    | LayoutOutputErrorKindV1::Negative(_)
                    | LayoutOutputErrorKindV1::FarEdgeArithmetic(_),
                    LayoutErrorLocationV1::OutputRecord { index: 0 },
                ) => Some(SpatialErrorLocationV2::Node { index: 3 }),
                (
                    LayoutOutputErrorKindV1::KeyMismatch
                    | LayoutOutputErrorKindV1::Negative(_)
                    | LayoutOutputErrorKindV1::FarEdgeArithmetic(_),
                    LayoutErrorLocationV1::OutputRecord { index: 1 },
                ) => Some(SpatialErrorLocationV2::Node { index: 4 }),
                (
                    LayoutOutputErrorKindV1::KeyMismatch
                    | LayoutOutputErrorKindV1::Negative(_)
                    | LayoutOutputErrorKindV1::FarEdgeArithmetic(_),
                    LayoutErrorLocationV1::OutputRecord { index: 2 },
                ) => Some(SpatialErrorLocationV2::Node { index: 6 }),
                _ => None,
            };
            let error =
                map_layout_execution_error!(graph, 1, LayoutErrorKindV1::Output(kind), location);
            match valid {
                Some(expected) => expect_layout(
                    Err::<(), _>(error),
                    SpatialLayoutErrorKindV2::Output(kind),
                    expected,
                ),
                None => expect_layout(
                    Err::<(), _>(error),
                    SpatialLayoutErrorKindV2::BridgeInvariant,
                    SpatialErrorLocationV2::Island { index: 1 },
                ),
            }
        }
    }
}

fn mapping_fixture() -> super::fixture::RawInputFixture {
    fixture(vec![
        root(),
        layout(1, 0, 1, 1),
        start_free(2, 1, SpatialAnchorTargetV2::Viewport),
        start_free(3, 0, SpatialAnchorTargetV2::Viewport),
        layout(4, 3, 1, 1),
        start_free(5, 4, SpatialAnchorTargetV2::Viewport),
        layout(6, 3, 1, 1),
    ])
}

const fn bridge_locations() -> [LayoutErrorLocationV1; 11] {
    [
        LayoutErrorLocationV1::Input,
        LayoutErrorLocationV1::Viewport,
        LayoutErrorLocationV1::InputNode { index: 0 },
        LayoutErrorLocationV1::InputNode { index: 1 },
        LayoutErrorLocationV1::InputNode { index: 2 },
        LayoutErrorLocationV1::InputNode { index: u32::MAX },
        LayoutErrorLocationV1::Output,
        LayoutErrorLocationV1::OutputRecord { index: 0 },
        LayoutErrorLocationV1::OutputRecord { index: 1 },
        LayoutErrorLocationV1::OutputRecord { index: 2 },
        LayoutErrorLocationV1::OutputRecord { index: u32::MAX },
    ]
}
