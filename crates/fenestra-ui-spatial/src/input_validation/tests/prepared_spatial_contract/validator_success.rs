use std::sync::Arc;

use super::ownership::{assert_identities, identities};
use super::snapshot_retention::RetainedFacts;
use super::support::{
    cross_axis_empty_owned, disjoint_clips_owned, distinct_viewport_root_owned, requested_limits,
    rich_engine, rich_owned, zero_call_engine,
};
use super::validator_support::*;
use super::*;
use crate::aabb::SpatialAabbV2;

#[test]
fn rich_reference_different_candidate_is_copied_and_retains_owned_state() {
    let source = rich_owned();
    let weak = Arc::downgrade(&source);
    let source_identities = identities(&source);
    let engine = rich_engine();
    let prepared = prepare_spatial_v2(&engine, source, requested_limits()).unwrap();
    let retained = RetainedFacts::capture(&prepared);
    let mut rows = rich_tables();
    shift_owner_one(&mut rows);
    let expected = clone_tables(&rows);
    let identities = table_identities(&rows);

    let snapshot = validate(prepared, &rows).expect("rich structural candidate is valid");

    assert_eq!(engine.call_count(), 1);
    assert_eq!(
        snapshot.viewport(),
        super::super::world_transform_support::VIEWPORT
    );
    assert_snapshot_tables(&snapshot, &expected);
    assert_different_allocations(&snapshot, identities);
    assert_eq!(
        snapshot.effective_clip_aabbs(),
        vec![canonical([3 * S, 22 * S, 6 * S, 26 * S]); 3]
    );
    retained.assert_snapshot(&snapshot);
    let upgraded = weak
        .upgrade()
        .expect("candidate snapshot retains input owner");
    assert!(Arc::ptr_eq(snapshot.source_arc(), &upgraded));
    assert_identities(snapshot.source_arc(), &source_identities);
    let bytes = snapshot.source_arc().as_input().resources().images()[1].bytes();
    assert_eq!(
        identity(snapshot.finalized_image_paint_bytes(1).unwrap()),
        identity(bytes)
    );
    drop(rows);
    assert_snapshot_tables(&snapshot, &expected);
    drop(upgraded);
    drop(snapshot);
    assert!(weak.upgrade().is_none());
}

#[test]
fn valid_geometry_may_differ_from_the_reference_and_base_origin_does_not_shift_aabb() {
    let prepared = prepare_spatial_v2(
        &zero_call_engine(),
        distinct_viewport_root_owned(),
        requested_limits(),
    )
    .unwrap();
    let mut rows = CandidateTables {
        geometry: vec![
            GeometryRow {
                key: 0,
                x: 123,
                y: -456,
                width: 2 * S,
                height: 3 * S,
                world: [-S, 0, 0, S, 7 * S, 11 * S],
                determinant: -D,
                aabb: (false, [5 * S, 11 * S, 7 * S, 14 * S]),
            }
            .build(),
        ],
        clips: Vec::new(),
        paints: Vec::new(),
        hits: Vec::new(),
        semantics: Vec::new(),
    };
    let expected = rows.geometry[0];
    let snapshot = validate(prepared, &rows).expect("reference-different geometry is structural");
    assert_eq!(
        snapshot.viewport(),
        crate::model::SpatialViewportV2::new(7, 9)
    );
    assert_eq!(snapshot.output().geometry(), &[expected]);
    rows.geometry.clear();
    assert_eq!(snapshot.output().geometry(), &[expected]);
}

#[test]
fn exact_cross_axis_candidate_accepts_wide_negative_determinant_and_empty_rows() {
    let source = cross_axis_empty_owned();
    let prepared =
        prepare_spatial_v2(&zero_call_engine(), source.clone(), requested_limits()).unwrap();
    let reference = materialize_reference_spatial_v2(
        prepare_spatial_v2(&zero_call_engine(), source, requested_limits()).unwrap(),
    );
    let rows = CandidateTables::from_snapshot(&reference);
    let snapshot =
        validate(prepared, &rows).expect("wide reflection and canonical empties are valid");
    assert_eq!(
        snapshot.output().geometry()[1].world_determinant(),
        -(1_i128 << 64)
    );
    assert!(!snapshot.output().geometry()[1].world_aabb().is_empty());
    assert!(
        snapshot.output().clips()[0]
            .primitive_world_aabb()
            .is_empty()
    );
    assert!(snapshot.output().paints()[0].world_aabb().is_empty());
    assert!(snapshot.output().hits()[0].world_aabb().is_empty());
    assert!(snapshot.output().semantics()[0].world_aabb().is_empty());
}

#[test]
fn candidate_clip_primitives_replace_reference_effective_bounds() {
    let source = disjoint_clips_owned();
    let prepared =
        prepare_spatial_v2(&zero_call_engine(), source.clone(), requested_limits()).unwrap();
    let reference = materialize_reference_spatial_v2(
        prepare_spatial_v2(&zero_call_engine(), source, requested_limits()).unwrap(),
    );
    let mut rows = CandidateTables::from_snapshot(&reference);
    let world = [S, 0, 0, S, 5 * S, 7 * S];
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = world;
    geometry.determinant = D;
    geometry.aabb = (false, [5 * S, 7 * S, 25 * S, 27 * S]);
    rows.geometry[1] = geometry.build();
    for (index, aabb) in [
        [5 * S, 7 * S, 7 * S, 9 * S],
        [15 * S, 17 * S, 16 * S, 18 * S],
    ]
    .into_iter()
    .enumerate()
    {
        let mut clip = ClipRow::read(rows.clips[index]);
        clip.world = world;
        clip.determinant = D;
        clip.aabb = (false, aabb);
        rows.clips[index] = clip.build();
    }
    let snapshot =
        validate(prepared, &rows).expect("candidate-derived disjoint clip chain is valid");
    assert_eq!(snapshot.output().clips(), rows.clips);
    assert_eq!(
        snapshot.effective_clip_aabbs(),
        &[
            canonical([5 * S, 7 * S, 7 * S, 9 * S]),
            SpatialAabbV2::empty()
        ]
    );
}

fn shift_owner_one(rows: &mut CandidateTables) {
    let world = [S, 0, 0, S, 2 * S, 20 * S];
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.x = 123;
    geometry.y = -456;
    geometry.width = 8 * S;
    geometry.height = 9 * S;
    geometry.world = world;
    geometry.determinant = D;
    geometry.aabb = (false, [2 * S, 20 * S, 10 * S, 29 * S]);
    rows.geometry[1] = geometry.build();
    for (index, aabb) in [
        [3 * S, 22 * S, 6 * S, 26 * S],
        [2 * S, 20 * S, 12 * S, 30 * S],
        [2 * S, 20 * S, 12 * S, 30 * S],
    ]
    .into_iter()
    .enumerate()
    {
        let mut clip = ClipRow::read(rows.clips[index]);
        clip.world = world;
        clip.determinant = D;
        clip.aabb = (false, aabb);
        rows.clips[index] = clip.build();
    }
    for (index, aabb) in [
        [3 * S, 22 * S, 6 * S, 26 * S],
        [12 * S, 40 * S, 15 * S, 44 * S],
    ]
    .into_iter()
    .enumerate()
    {
        let mut paint = PaintRow::read(rows.paints[index]);
        paint.world = world;
        paint.determinant = D;
        paint.aabb = (false, aabb);
        rows.paints[index] = paint.build();
    }
}

fn clone_tables(rows: &CandidateTables) -> CandidateTables {
    CandidateTables {
        geometry: rows.geometry.clone(),
        clips: rows.clips.clone(),
        paints: rows.paints.clone(),
        hits: rows.hits.clone(),
        semantics: rows.semantics.clone(),
    }
}

type TableIdentities = [(*const (), usize); 5];
fn table_identities(rows: &CandidateTables) -> TableIdentities {
    [
        erased(&rows.geometry),
        erased(&rows.clips),
        erased(&rows.paints),
        erased(&rows.hits),
        erased(&rows.semantics),
    ]
}
fn assert_different_allocations(snapshot: &SpatialResolvedSnapshotV2, old: TableIdentities) {
    let output = snapshot.output();
    for (new, old) in [
        erased(output.geometry()),
        erased(output.clips()),
        erased(output.paints()),
        erased(output.hits()),
        erased(output.semantics()),
    ]
    .into_iter()
    .zip(old)
    {
        assert_eq!(new.1, old.1);
        assert_ne!(new.0, old.0);
    }
}
fn assert_snapshot_tables(snapshot: &SpatialResolvedSnapshotV2, expected: &CandidateTables) {
    let output = snapshot.output();
    assert_eq!(output.geometry(), expected.geometry);
    assert_eq!(output.clips(), expected.clips);
    assert_eq!(output.paints(), expected.paints);
    assert_eq!(output.hits(), expected.hits);
    assert_eq!(output.semantics(), expected.semantics);
}
pub(super) fn canonical(v: [i64; 4]) -> SpatialAabbV2 {
    SpatialAabbV2::from_edges(scalar(v[0]), scalar(v[1]), scalar(v[2]), scalar(v[3])).unwrap()
}
fn erased<T>(slice: &[T]) -> (*const (), usize) {
    (slice.as_ptr().cast(), slice.len())
}
fn identity<T>(slice: &[T]) -> (*const T, usize) {
    (slice.as_ptr(), slice.len())
}
