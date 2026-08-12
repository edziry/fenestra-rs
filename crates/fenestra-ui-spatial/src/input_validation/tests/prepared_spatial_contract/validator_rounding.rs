use std::sync::Arc;

use super::super::effective_clip_aabb_support::clips_only_fixture;
use super::super::validated_clip_support::root_clip;
use super::super::validated_shape_support::rect_values;
use super::super::world_transform_support::VIEWPORT;
use super::support::{requested_limits, zero_call_engine};
use super::validator_success::canonical;
use super::validator_support::*;
use super::*;

#[test]
fn candidate_aabb_derivation_rounds_signed_half_ticks_outward() {
    let source = Arc::new(
        clips_only_fixture(
            vec![rect_values(0, 1, 1, 1, 2, 2)],
            vec![root_clip(0, 1, 0)],
        )
        .into_owned(VIEWPORT),
    );
    let prepared =
        prepare_spatial_v2(&zero_call_engine(), source.clone(), requested_limits()).unwrap();
    let reference = materialize_reference_spatial_v2(
        prepare_spatial_v2(&zero_call_engine(), source, requested_limits()).unwrap(),
    );
    let mut rows = CandidateTables::from_snapshot(&reference);
    let world = [-S / 2, 0, 0, S / 2, 0, 0];
    let determinant = -(D / 4);
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = world;
    geometry.determinant = determinant;
    geometry.aabb = (false, [-10 * S, 0, 0, 10 * S]);
    rows.geometry[1] = geometry.build();
    let mut clip = ClipRow::read(rows.clips[0]);
    clip.world = world;
    clip.determinant = determinant;
    clip.aabb = (false, [-2, 0, 0, 2]);
    rows.clips[0] = clip.build();

    let snapshot = validate(prepared, &rows).expect("half-tick extrema round outward");
    assert_eq!(
        snapshot.output().clips()[0].primitive_world_aabb(),
        raw_aabb(false, [-2, 0, 0, 2])
    );
    assert_eq!(snapshot.effective_clip_aabbs(), &[canonical([-2, 0, 0, 2])]);
}
