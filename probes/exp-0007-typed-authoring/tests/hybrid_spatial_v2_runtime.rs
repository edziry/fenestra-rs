#[path = "support/hybrid_spatial_v2/mod.rs"]
mod support;

use fenestra_ui_exp_0007_typed_authoring::{generated_hybrid_spatial_v2, macro_hybrid_spatial_v2};
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};
use fenestra_ui_spatial::prototype::{
    SpatialErrorLocationV2, SpatialResolveErrorKindV2, SpatialTransformErrorKindV2,
    SpatialViewportV2,
};

use support::{manual_hybrid_spatial_v2, run_authored_spatial_lane};

#[test]
fn manual_fen_and_ui_lanes_publish_the_same_complete_spatial_log() {
    let manual = run_authored_spatial_lane(manual_hybrid_spatial_v2());
    let fen = run_authored_spatial_lane(generated_hybrid_spatial_v2());
    let ui = run_authored_spatial_lane(macro_hybrid_spatial_v2());

    assert_eq!(manual, fen);
    assert_eq!(manual, ui);
    assert_eq!(manual.generations(), (0_u64..=8).collect::<Vec<_>>());
    assert_eq!(
        manual.viewports(),
        std::iter::once(SpatialViewportV2::new(192, 128))
            .chain(std::iter::repeat_n(SpatialViewportV2::new(224, 160), 8))
            .collect::<Vec<_>>()
    );
    assert_eq!(manual.mapping_counts(), [9, 9, 9, 9, 9, 10, 10, 10, 9]);
    assert_eq!(manual.geometry_counts(), manual.mapping_counts());
    assert_eq!(manual.clip_counts(), [4, 4, 4, 4, 4, 5, 5, 5, 4]);
    assert_eq!(manual.paint_counts(), [5, 5, 5, 5, 5, 6, 6, 6, 5]);
    assert_eq!(manual.hit_counts(), [5, 5, 5, 5, 5, 6, 6, 6, 5]);
    assert_eq!(manual.semantic_counts(), [4, 4, 4, 4, 4, 5, 5, 5, 4]);
    assert_eq!(
        manual.hit_query_counts(),
        [
            24_580, 35_844, 35_844, 35_844, 35_844, 35_844, 35_844, 35_844, 35_844
        ]
    );
    assert_eq!(
        manual.raster_byte_counts(),
        [
            98_304, 143_360, 143_360, 143_360, 143_360, 143_360, 143_360, 143_360, 143_360
        ]
    );
    assert_eq!(manual.final_keys(), &[10, 30]);
    assert!(manual.noop_checks().all_preserved());
}

#[test]
fn singular_authored_property_failure_is_exact_and_rolls_back_every_observation() {
    for programs in [
        manual_hybrid_spatial_v2(),
        generated_hybrid_spatial_v2(),
        macro_hybrid_spatial_v2(),
    ] {
        let log = run_authored_spatial_lane(programs);
        let failure = log.failure();
        assert_eq!(
            failure.resolve_kind(),
            SpatialResolveErrorKindV2::Transform(SpatialTransformErrorKindV2::SingularTransform)
        );
        assert_eq!(
            failure.resolve_location(),
            SpatialErrorLocationV2::Node { index: 3 }
        );
        assert_eq!(
            failure.ir_span(),
            SourceSpan::bytes(SourceId::new(0), 226, 227)
        );
        assert_eq!(failure.operation_index(), None);
        assert!(failure.outer_state_preserved());
        assert!(failure.spatial_snapshot_preserved());
        assert!(failure.complete_observation_preserved());
    }
}
