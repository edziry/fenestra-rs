use super::super::world_transform_support::SCALE;
use super::support::{
    computed_layout_engine, computed_layout_owned, cross_axis_empty_owned, disjoint_clips_owned,
    distinct_viewport_root_owned, requested_limits, root_only_owned, zero_call_engine,
};
use super::*;
use crate::model::SpatialScalarV2;
use crate::output_aabb::SpatialOutputAabbV2;

#[test]
fn root_only_materialization_has_one_geometry_row_and_no_content_rows() {
    let engine = zero_call_engine();
    let prepared = prepare_spatial_v2(&engine, root_only_owned(), requested_limits()).unwrap();
    let snapshot = materialize_reference_spatial_v2(prepared);
    let output = snapshot.output();

    assert_eq!(engine.call_count(), 0);
    assert_eq!(
        output.geometry(),
        &[crate::output_geometry::SpatialGeometryOutputRecordV2::new(
            crate::model::SpatialNodeKeyV2::new(0),
            scalar(0),
            scalar(0),
            scalar(20 * SCALE),
            scalar(20 * SCALE),
            crate::model::Affine2V2::identity(),
            (SCALE as i128) * (SCALE as i128),
            raw(false, 0, 0, 20 * SCALE, 20 * SCALE),
        )]
    );
    assert!(output.clips().is_empty());
    assert!(output.paints().is_empty());
    assert!(output.hits().is_empty());
    assert!(output.semantics().is_empty());
    assert!(snapshot.effective_clip_aabbs().is_empty());
}

#[test]
fn snapshot_retains_a_distinct_input_viewport_and_its_root_extent() {
    let prepared = prepare_spatial_v2(
        &zero_call_engine(),
        distinct_viewport_root_owned(),
        requested_limits(),
    )
    .unwrap();
    let snapshot = materialize_reference_spatial_v2(prepared);
    let geometry = snapshot.output().geometry()[0];

    assert_eq!(
        snapshot.viewport(),
        crate::model::SpatialViewportV2::new(7, 9)
    );
    assert_eq!(geometry.base_width().raw(), 7 * SCALE);
    assert_eq!(geometry.base_height().raw(), 9 * SCALE);
    assert_eq!(
        geometry.world_aabb(),
        raw(false, 0, 0, 7 * SCALE, 9 * SCALE)
    );
}

#[test]
fn materialization_uses_computed_layout_geometry_instead_of_authored_preferences() {
    let prepared = prepare_spatial_v2(
        &computed_layout_engine(),
        computed_layout_owned(),
        requested_limits(),
    )
    .unwrap();
    let snapshot = materialize_reference_spatial_v2(prepared);
    let geometry = snapshot.output().geometry()[1];

    assert_eq!(geometry.base_x().raw(), 7 * SCALE);
    assert_eq!(geometry.base_y().raw(), 8 * SCALE);
    assert_eq!(geometry.base_width().raw(), 5 * SCALE);
    assert_eq!(geometry.base_height().raw(), 6 * SCALE);
    assert_eq!(
        affine(geometry.world_from_local()),
        [SCALE, 0, 0, SCALE, 7 * SCALE, 8 * SCALE]
    );
    assert_eq!(
        geometry.world_aabb(),
        raw(false, 7 * SCALE, 8 * SCALE, 12 * SCALE, 14 * SCALE)
    );
}

#[test]
fn disjoint_nested_clips_retain_nonempty_primitives_and_empty_effective_child() {
    let prepared = prepare_spatial_v2(
        &zero_call_engine(),
        disjoint_clips_owned(),
        requested_limits(),
    )
    .unwrap();
    let snapshot = materialize_reference_spatial_v2(prepared);
    let output = snapshot.output();

    assert_eq!(
        output.clips()[0].primitive_world_aabb(),
        raw(false, 0, 0, 2 * SCALE, 2 * SCALE)
    );
    assert_eq!(
        output.clips()[1].primitive_world_aabb(),
        raw(false, 10 * SCALE, 10 * SCALE, 11 * SCALE, 11 * SCALE,)
    );
    assert_eq!(
        snapshot.effective_clip_aabbs(),
        &[
            crate::aabb::SpatialAabbV2::from_edges(
                scalar(0),
                scalar(0),
                scalar(2 * SCALE),
                scalar(2 * SCALE),
            )
            .unwrap(),
            crate::aabb::SpatialAabbV2::empty(),
        ]
    );
}

#[test]
fn cross_axis_wide_determinant_and_canonical_empty_content_rows_are_materialized() {
    let prepared = prepare_spatial_v2(
        &zero_call_engine(),
        cross_axis_empty_owned(),
        requested_limits(),
    )
    .unwrap();
    let snapshot = materialize_reference_spatial_v2(prepared);
    let output = snapshot.output();
    let q = 1_i64 << 32;
    let world = [0, q, q, 0, 8 * SCALE, 11 * SCALE];
    let determinant = -(1_i128 << 64);

    let geometry = output.geometry()[1];
    assert_eq!(affine(geometry.world_from_local()), world);
    assert_eq!(geometry.world_determinant(), determinant);
    assert_eq!(geometry.base_x().raw(), 3 * SCALE);
    assert_eq!(geometry.base_y().raw(), 4 * SCALE);
    assert_eq!(geometry.base_width().raw(), 0);
    assert_eq!(geometry.base_height().raw(), 0);
    assert_eq!(
        geometry.world_aabb(),
        raw(false, 8 * SCALE, 11 * SCALE, 8 * SCALE, 11 * SCALE)
    );

    let empty = raw(true, 0, 0, 0, 0);
    assert_eq!(output.clips()[0].primitive_world_aabb(), empty);
    assert_eq!(output.paints()[0].world_aabb(), empty);
    assert_eq!(output.hits()[0].world_aabb(), empty);
    assert_eq!(output.semantics()[0].world_aabb(), empty);
    for row in [
        output.clips()[0].world_from_local(),
        output.paints()[0].world_from_local(),
        output.hits()[0].world_from_local(),
        output.semantics()[0].world_from_local(),
    ] {
        assert_eq!(affine(row), world);
    }
    assert_eq!(output.clips()[0].world_determinant(), determinant);
    assert_eq!(output.paints()[0].world_determinant(), determinant);
    assert_eq!(output.hits()[0].world_determinant(), determinant);
    assert_eq!(output.semantics()[0].world_determinant(), determinant);
    assert_eq!(
        snapshot.effective_clip_aabbs(),
        &[crate::aabb::SpatialAabbV2::empty()]
    );
}

fn affine(value: crate::model::Affine2V2) -> [i64; 6] {
    [
        value.a().raw(),
        value.b().raw(),
        value.c().raw(),
        value.d().raw(),
        value.tx().raw(),
        value.ty().raw(),
    ]
}

fn raw(empty: bool, min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> SpatialOutputAabbV2 {
    SpatialOutputAabbV2::new(
        empty,
        scalar(min_x),
        scalar(min_y),
        scalar(max_x),
        scalar(max_y),
    )
}
const fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}
