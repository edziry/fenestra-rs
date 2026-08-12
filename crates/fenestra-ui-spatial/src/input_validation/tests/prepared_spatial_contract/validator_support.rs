use std::error::Error;

use super::snapshot_output::{
    expected_clips, expected_geometry, expected_hits, expected_paints, expected_semantics,
};
use super::support::{requested_limits, rich_engine, rich_owned};
use super::*;
use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::model::{Affine2V2, SpatialNodeKeyV2, SpatialScalarV2};
use crate::output_aabb::SpatialOutputAabbV2;
use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
use crate::output_geometry::{SpatialClipOutputRecordV2, SpatialGeometryOutputRecordV2};
use crate::output_item::{
    SpatialHitOutputRecordV2, SpatialPaintOutputRecordV2, SpatialPaintOutputReferenceV2,
    SpatialSemanticOutputRecordV2,
};
use crate::output_view::SpatialOutputV2;
use crate::resolve_error::{
    SpatialOutputErrorKindV2, SpatialResolveErrorKindV2, SpatialResolveErrorV2,
};

pub(super) const S: i64 = SpatialScalarV2::SCALE;
pub(super) const D: i128 = (S as i128) * (S as i128);

pub(super) struct CandidateTables {
    pub(super) geometry: Vec<SpatialGeometryOutputRecordV2>,
    pub(super) clips: Vec<SpatialClipOutputRecordV2>,
    pub(super) paints: Vec<SpatialPaintOutputRecordV2>,
    pub(super) hits: Vec<SpatialHitOutputRecordV2>,
    pub(super) semantics: Vec<SpatialSemanticOutputRecordV2>,
}

impl CandidateTables {
    pub(super) fn from_snapshot(snapshot: &SpatialResolvedSnapshotV2) -> Self {
        let output = snapshot.output();
        Self {
            geometry: output.geometry().to_vec(),
            clips: output.clips().to_vec(),
            paints: output.paints().to_vec(),
            hits: output.hits().to_vec(),
            semantics: output.semantics().to_vec(),
        }
    }

    pub(super) fn view(&self) -> SpatialOutputV2<'_> {
        SpatialOutputV2::new(
            &self.geometry,
            &self.clips,
            &self.paints,
            &self.hits,
            &self.semantics,
        )
    }
}

pub(super) fn rich_case() -> (PreparedSpatialV2, CandidateTables) {
    let prepared = prepare_spatial_v2(&rich_engine(), rich_owned(), requested_limits()).unwrap();
    (prepared, rich_tables())
}

pub(super) fn rich_tables() -> CandidateTables {
    CandidateTables {
        geometry: expected_geometry(),
        clips: expected_clips(),
        paints: expected_paints(),
        hits: expected_hits(),
        semantics: expected_semantics(),
    }
}

pub(super) fn validate(
    prepared: PreparedSpatialV2,
    tables: &CandidateTables,
) -> Result<SpatialResolvedSnapshotV2, SpatialResolveErrorV2> {
    validate_spatial_output_v2(prepared, tables.view())
}

pub(super) fn expect_output_error(
    result: Result<SpatialResolvedSnapshotV2, SpatialResolveErrorV2>,
    kind: SpatialOutputErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    let error = match result {
        Ok(_) => panic!("expected candidate output failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), SpatialResolveErrorKindV2::Output(kind));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(format!("{error}"), "spatial-resolve-error(output)");
    assert_eq!(
        format!("{error:?}"),
        "SpatialResolveErrorV2(spatial-resolve-error(output))"
    );
    assert!(Error::source(&error).is_none());
}

pub(super) const fn output_location(
    table: SpatialOutputTableV2,
    index: u32,
    field: SpatialOutputFieldV2,
) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::OutputRecord {
        table,
        index,
        field,
    }
}

pub(super) const fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

pub(super) const fn affine(values: [i64; 6]) -> Affine2V2 {
    Affine2V2::new(
        scalar(values[0]),
        scalar(values[1]),
        scalar(values[2]),
        scalar(values[3]),
        scalar(values[4]),
        scalar(values[5]),
    )
}

pub(super) const fn raw_aabb(empty: bool, values: [i64; 4]) -> SpatialOutputAabbV2 {
    SpatialOutputAabbV2::new(
        empty,
        scalar(values[0]),
        scalar(values[1]),
        scalar(values[2]),
        scalar(values[3]),
    )
}

#[derive(Clone, Copy)]
pub(super) struct GeometryRow {
    pub key: u32,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub world: [i64; 6],
    pub determinant: i128,
    pub aabb: (bool, [i64; 4]),
}

impl GeometryRow {
    pub(super) fn read(row: SpatialGeometryOutputRecordV2) -> Self {
        Self {
            key: row.key().get(),
            x: row.base_x().raw(),
            y: row.base_y().raw(),
            width: row.base_width().raw(),
            height: row.base_height().raw(),
            world: affine_values(row.world_from_local()),
            determinant: row.world_determinant(),
            aabb: aabb_values(row.world_aabb()),
        }
    }

    pub(super) fn build(self) -> SpatialGeometryOutputRecordV2 {
        SpatialGeometryOutputRecordV2::new(
            SpatialNodeKeyV2::new(self.key),
            scalar(self.x),
            scalar(self.y),
            scalar(self.width),
            scalar(self.height),
            affine(self.world),
            self.determinant,
            raw_aabb(self.aabb.0, self.aabb.1),
        )
    }
}

#[derive(Clone, Copy)]
pub(super) struct ClipRow {
    pub key: u32,
    pub world: [i64; 6],
    pub determinant: i128,
    pub aabb: (bool, [i64; 4]),
    pub owner: u32,
    pub parent: Option<u32>,
    pub shape: u32,
}

impl ClipRow {
    pub(super) fn read(row: SpatialClipOutputRecordV2) -> Self {
        Self {
            key: row.key().get(),
            world: affine_values(row.world_from_local()),
            determinant: row.world_determinant(),
            aabb: aabb_values(row.primitive_world_aabb()),
            owner: row.owner().get(),
            parent: row.parent().map(SpatialClipKeyV2::get),
            shape: row.shape().get(),
        }
    }

    pub(super) fn build(self) -> SpatialClipOutputRecordV2 {
        SpatialClipOutputRecordV2::new(
            SpatialClipKeyV2::new(self.key),
            affine(self.world),
            self.determinant,
            raw_aabb(self.aabb.0, self.aabb.1),
            SpatialNodeKeyV2::new(self.owner),
            self.parent.map(SpatialClipKeyV2::new),
            SpatialShapeKeyV2::new(self.shape),
        )
    }
}

#[derive(Clone, Copy)]
pub(super) struct PaintRow {
    pub key: u32,
    pub world: [i64; 6],
    pub determinant: i128,
    pub aabb: (bool, [i64; 4]),
    pub owner: u32,
    pub reference: SpatialPaintOutputReferenceV2,
    pub clip: Option<u32>,
    pub stack: u32,
    pub item: u32,
}

impl PaintRow {
    pub(super) fn read(row: SpatialPaintOutputRecordV2) -> Self {
        Self {
            key: row.key(),
            world: affine_values(row.world_from_local()),
            determinant: row.world_determinant(),
            aabb: aabb_values(row.world_aabb()),
            owner: row.owner().get(),
            reference: row.reference(),
            clip: row.clip().map(SpatialClipKeyV2::get),
            stack: row.stack_ordinal(),
            item: row.item_ordinal(),
        }
    }

    pub(super) fn build(self) -> SpatialPaintOutputRecordV2 {
        SpatialPaintOutputRecordV2::new(
            self.key,
            affine(self.world),
            self.determinant,
            raw_aabb(self.aabb.0, self.aabb.1),
            SpatialNodeKeyV2::new(self.owner),
            self.reference,
            self.clip.map(SpatialClipKeyV2::new),
            self.stack,
            self.item,
        )
    }
}

#[derive(Clone, Copy)]
pub(super) struct ShapeItemRow {
    pub key: u32,
    pub world: [i64; 6],
    pub determinant: i128,
    pub aabb: (bool, [i64; 4]),
    pub owner: u32,
    pub shape: u32,
    pub clip: Option<u32>,
    pub stack: u32,
    pub item: u32,
}

impl ShapeItemRow {
    pub(super) fn read_hit(row: SpatialHitOutputRecordV2) -> Self {
        Self::new(
            row.key(),
            row.world_from_local(),
            row.world_determinant(),
            row.world_aabb(),
            row.owner(),
            row.shape(),
            row.clip(),
            row.stack_ordinal(),
            row.item_ordinal(),
        )
    }

    pub(super) fn read_semantic(row: SpatialSemanticOutputRecordV2) -> Self {
        Self::new(
            row.key(),
            row.world_from_local(),
            row.world_determinant(),
            row.world_aabb(),
            row.owner(),
            row.shape(),
            row.clip(),
            row.stack_ordinal(),
            row.item_ordinal(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        key: u32,
        world: Affine2V2,
        determinant: i128,
        aabb: SpatialOutputAabbV2,
        owner: SpatialNodeKeyV2,
        shape: SpatialShapeKeyV2,
        clip: Option<SpatialClipKeyV2>,
        stack: u32,
        item: u32,
    ) -> Self {
        Self {
            key,
            world: affine_values(world),
            determinant,
            aabb: aabb_values(aabb),
            owner: owner.get(),
            shape: shape.get(),
            clip: clip.map(SpatialClipKeyV2::get),
            stack,
            item,
        }
    }

    pub(super) fn build_hit(self) -> SpatialHitOutputRecordV2 {
        SpatialHitOutputRecordV2::new(
            self.key,
            affine(self.world),
            self.determinant,
            raw_aabb(self.aabb.0, self.aabb.1),
            SpatialNodeKeyV2::new(self.owner),
            SpatialShapeKeyV2::new(self.shape),
            self.clip.map(SpatialClipKeyV2::new),
            self.stack,
            self.item,
        )
    }

    pub(super) fn build_semantic(self) -> SpatialSemanticOutputRecordV2 {
        SpatialSemanticOutputRecordV2::new(
            self.key,
            affine(self.world),
            self.determinant,
            raw_aabb(self.aabb.0, self.aabb.1),
            SpatialNodeKeyV2::new(self.owner),
            SpatialShapeKeyV2::new(self.shape),
            self.clip.map(SpatialClipKeyV2::new),
            self.stack,
            self.item,
        )
    }
}

pub(super) const fn coverage(shape: u32, brush: u32) -> SpatialPaintOutputReferenceV2 {
    SpatialPaintOutputReferenceV2::Coverage {
        shape: SpatialShapeKeyV2::new(shape),
        brush: SpatialBrushKeyV2::new(brush),
    }
}

pub(super) const fn image(image: u32) -> SpatialPaintOutputReferenceV2 {
    SpatialPaintOutputReferenceV2::Image {
        image: SpatialImageKeyV2::new(image),
    }
}

fn affine_values(value: Affine2V2) -> [i64; 6] {
    [
        value.a().raw(),
        value.b().raw(),
        value.c().raw(),
        value.d().raw(),
        value.tx().raw(),
        value.ty().raw(),
    ]
}

fn aabb_values(value: SpatialOutputAabbV2) -> (bool, [i64; 4]) {
    (
        value.is_empty(),
        [
            value.min_x().raw(),
            value.min_y().raw(),
            value.max_x().raw(),
            value.max_y().raw(),
        ],
    )
}
