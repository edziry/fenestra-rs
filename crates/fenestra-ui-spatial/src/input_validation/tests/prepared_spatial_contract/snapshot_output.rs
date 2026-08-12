use super::super::world_transform_support::SCALE;
use super::support::{requested_limits, rich_engine, rich_owned};
use super::*;
use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::model::{Affine2V2, SpatialNodeKeyV2, SpatialScalarV2};
use crate::output_aabb::SpatialOutputAabbV2;
use crate::output_geometry::{SpatialClipOutputRecordV2, SpatialGeometryOutputRecordV2};
use crate::output_item::{
    SpatialHitOutputRecordV2, SpatialPaintOutputRecordV2, SpatialPaintOutputReferenceV2,
    SpatialSemanticOutputRecordV2,
};

const DETERMINANT: i128 = (SCALE as i128) * (SCALE as i128);

#[test]
fn reference_materialization_emits_every_exact_rich_output_row_in_table_order() {
    let prepared = prepare_spatial_v2(&rich_engine(), rich_owned(), requested_limits())
        .expect("rich owned input prepares successfully");
    let snapshot = materialize_reference_spatial_v2(prepared);
    assert_rich_snapshot(&snapshot);
}

pub(super) fn assert_rich_snapshot(snapshot: &SpatialResolvedSnapshotV2) {
    let output = snapshot.output();

    assert_eq!(
        snapshot.viewport(),
        super::super::world_transform_support::VIEWPORT
    );
    assert_eq!(output.geometry(), expected_geometry().as_slice());
    assert_eq!(output.clips(), expected_clips().as_slice());
    assert_eq!(output.paints(), expected_paints().as_slice());
    assert_eq!(output.hits(), expected_hits().as_slice());
    assert_eq!(output.semantics(), expected_semantics().as_slice());
    assert_eq!(
        snapshot.effective_clip_aabbs(),
        vec![canonical(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE); 3]
    );

    let again = snapshot.output();
    assert_eq!(identity(output.geometry()), identity(again.geometry()));
    assert_eq!(identity(output.clips()), identity(again.clips()));
    assert_eq!(identity(output.paints()), identity(again.paints()));
    assert_eq!(identity(output.hits()), identity(again.hits()));
    assert_eq!(identity(output.semantics()), identity(again.semantics()));
    assert_eq!(
        identity(snapshot.effective_clip_aabbs()),
        identity(snapshot.effective_clip_aabbs())
    );
}

fn identity<T>(slice: &[T]) -> (*const T, usize) {
    (slice.as_ptr(), slice.len())
}

fn expected_geometry() -> Vec<SpatialGeometryOutputRecordV2> {
    let worlds = worlds();
    [
        (0, 0, 0, 20, 20, bounds(0, 0, 20 * SCALE, 20 * SCALE)),
        (
            1,
            0,
            10 * SCALE,
            10,
            10,
            bounds(0, 10 * SCALE, 10 * SCALE, 20 * SCALE),
        ),
        (
            2,
            SCALE,
            12 * SCALE,
            3,
            4,
            bounds(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE),
        ),
        (
            3,
            SCALE,
            6 * SCALE,
            10,
            10,
            bounds(SCALE, 6 * SCALE, 11 * SCALE, 16 * SCALE),
        ),
    ]
    .into_iter()
    .map(|(key, x, y, width, height, aabb)| {
        SpatialGeometryOutputRecordV2::new(
            SpatialNodeKeyV2::new(key),
            scalar(x),
            scalar(y),
            scalar(width * SCALE),
            scalar(height * SCALE),
            worlds[key as usize],
            DETERMINANT,
            aabb,
        )
    })
    .collect()
}

fn expected_clips() -> Vec<SpatialClipOutputRecordV2> {
    let world = worlds()[1];
    [
        (0, None, 0, bounds(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE)),
        (1, Some(0), 3, bounds(0, 10 * SCALE, 10 * SCALE, 20 * SCALE)),
        (2, Some(1), 3, bounds(0, 10 * SCALE, 10 * SCALE, 20 * SCALE)),
    ]
    .into_iter()
    .map(|(key, parent, shape, aabb)| {
        SpatialClipOutputRecordV2::new(
            SpatialClipKeyV2::new(key),
            world,
            DETERMINANT,
            aabb,
            SpatialNodeKeyV2::new(1),
            parent.map(SpatialClipKeyV2::new),
            SpatialShapeKeyV2::new(shape),
        )
    })
    .collect()
}

fn expected_paints() -> Vec<SpatialPaintOutputRecordV2> {
    let worlds = worlds();
    vec![
        paint(
            0,
            worlds[1],
            bounds(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE),
            1,
            SpatialPaintOutputReferenceV2::Coverage {
                shape: SpatialShapeKeyV2::new(0),
                brush: SpatialBrushKeyV2::new(0),
            },
            Some(2),
            0,
        ),
        paint(
            1,
            worlds[1],
            bounds(10 * SCALE, 30 * SCALE, 13 * SCALE, 34 * SCALE),
            1,
            SpatialPaintOutputReferenceV2::Image {
                image: SpatialImageKeyV2::new(1),
            },
            None,
            1,
        ),
        paint(
            2,
            worlds[2],
            bounds(-2 * SCALE, 10 * SCALE, 6 * SCALE, 18 * SCALE),
            2,
            SpatialPaintOutputReferenceV2::Coverage {
                shape: SpatialShapeKeyV2::new(1),
                brush: SpatialBrushKeyV2::new(1),
            },
            None,
            0,
        ),
    ]
}

fn expected_hits() -> Vec<SpatialHitOutputRecordV2> {
    let worlds = worlds();
    vec![
        hit(
            0,
            worlds[2],
            bounds(-SCALE, 11 * SCALE, 5 * SCALE, 17 * SCALE),
            2,
            1,
            Some(0),
            0,
        ),
        hit(
            1,
            worlds[3],
            bounds(0, 5 * SCALE, 4 * SCALE, 7 * SCALE),
            3,
            2,
            Some(1),
            0,
        ),
        hit(
            2,
            worlds[3],
            bounds(-SCALE, 4 * SCALE, 5 * SCALE, 8 * SCALE),
            3,
            2,
            Some(2),
            1,
        ),
    ]
}

fn expected_semantics() -> Vec<SpatialSemanticOutputRecordV2> {
    let world = worlds()[3];
    [Some(2), Some(1)]
        .into_iter()
        .enumerate()
        .map(|(key, clip)| {
            SpatialSemanticOutputRecordV2::new(
                key as u32,
                world,
                DETERMINANT,
                bounds(SCALE, 6 * SCALE, 3 * SCALE, 6 * SCALE),
                SpatialNodeKeyV2::new(3),
                SpatialShapeKeyV2::new(2),
                clip.map(SpatialClipKeyV2::new),
                3,
                key as u32,
            )
        })
        .collect()
}

fn paint(
    key: u32,
    world: Affine2V2,
    aabb: SpatialOutputAabbV2,
    owner: u32,
    reference: SpatialPaintOutputReferenceV2,
    clip: Option<u32>,
    item: u32,
) -> SpatialPaintOutputRecordV2 {
    SpatialPaintOutputRecordV2::new(
        key,
        world,
        DETERMINANT,
        aabb,
        SpatialNodeKeyV2::new(owner),
        reference,
        clip.map(SpatialClipKeyV2::new),
        owner,
        item,
    )
}

fn hit(
    key: u32,
    world: Affine2V2,
    aabb: SpatialOutputAabbV2,
    owner: u32,
    shape: u32,
    clip: Option<u32>,
    item: u32,
) -> SpatialHitOutputRecordV2 {
    SpatialHitOutputRecordV2::new(
        key,
        world,
        DETERMINANT,
        aabb,
        SpatialNodeKeyV2::new(owner),
        SpatialShapeKeyV2::new(shape),
        clip.map(SpatialClipKeyV2::new),
        owner,
        item,
    )
}

fn worlds() -> [Affine2V2; 4] {
    [
        affine([SCALE, 0, 0, SCALE, 0, 0]),
        affine([SCALE, 0, 0, SCALE, 0, 10 * SCALE]),
        affine([SCALE, 0, 0, SCALE, SCALE, 12 * SCALE]),
        affine([SCALE, 0, 0, SCALE, SCALE, 6 * SCALE]),
    ]
}

fn affine(v: [i64; 6]) -> Affine2V2 {
    Affine2V2::new(
        scalar(v[0]),
        scalar(v[1]),
        scalar(v[2]),
        scalar(v[3]),
        scalar(v[4]),
        scalar(v[5]),
    )
}
fn bounds(a: i64, b: i64, c: i64, d: i64) -> SpatialOutputAabbV2 {
    SpatialOutputAabbV2::new(false, scalar(a), scalar(b), scalar(c), scalar(d))
}
fn canonical(a: i64, b: i64, c: i64, d: i64) -> crate::aabb::SpatialAabbV2 {
    crate::aabb::SpatialAabbV2::from_edges(scalar(a), scalar(b), scalar(c), scalar(d)).unwrap()
}
const fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}
