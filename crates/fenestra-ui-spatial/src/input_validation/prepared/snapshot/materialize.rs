use super::super::model::{PreparedCoverage, PreparedPaintContent, PreparedSpatialState};
use crate::aabb::SpatialAabbV2;
use crate::content_key::{SpatialBrushKeyV2, SpatialImageKeyV2};
use crate::geometry_key::{SpatialClipKeyV2, SpatialShapeKeyV2};
use crate::model::{SpatialNodeKeyV2, SpatialScalarV2};
use crate::output_aabb::SpatialOutputAabbV2;
use crate::output_geometry::{SpatialClipOutputRecordV2, SpatialGeometryOutputRecordV2};
use crate::output_item::{
    SpatialHitOutputRecordV2, SpatialPaintOutputRecordV2, SpatialPaintOutputReferenceV2,
    SpatialSemanticOutputRecordV2,
};

pub(super) struct MaterializedTables {
    pub(super) geometry: Box<[SpatialGeometryOutputRecordV2]>,
    pub(super) clips: Box<[SpatialClipOutputRecordV2]>,
    pub(super) paints: Box<[SpatialPaintOutputRecordV2]>,
    pub(super) hits: Box<[SpatialHitOutputRecordV2]>,
    pub(super) semantics: Box<[SpatialSemanticOutputRecordV2]>,
}

pub(super) fn materialize_tables(state: &PreparedSpatialState) -> MaterializedTables {
    MaterializedTables {
        geometry: materialize_geometry(state),
        clips: materialize_clips(state),
        paints: materialize_paints(state),
        hits: materialize_hits(state),
        semantics: materialize_semantics(state),
    }
}

fn materialize_geometry(state: &PreparedSpatialState) -> Box<[SpatialGeometryOutputRecordV2]> {
    state
        .base_geometry
        .iter()
        .enumerate()
        .map(|(index, base)| {
            let world = state.world_transforms[index];
            SpatialGeometryOutputRecordV2::new(
                SpatialNodeKeyV2::new(ordinal(index)),
                base.x,
                base.y,
                fixed_extent(base.width),
                fixed_extent(base.height),
                world,
                world.determinant_raw(),
                output_aabb(state.world_aabbs.geometry[index]),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn materialize_clips(state: &PreparedSpatialState) -> Box<[SpatialClipOutputRecordV2]> {
    state
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            let world = state.world_transforms[clip.owner as usize];
            SpatialClipOutputRecordV2::new(
                SpatialClipKeyV2::new(ordinal(index)),
                world,
                world.determinant_raw(),
                output_aabb(state.world_aabbs.clips[index]),
                SpatialNodeKeyV2::new(clip.owner),
                clip.parent.map(SpatialClipKeyV2::new),
                SpatialShapeKeyV2::new(clip.shape),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn materialize_paints(state: &PreparedSpatialState) -> Box<[SpatialPaintOutputRecordV2]> {
    state
        .paints
        .iter()
        .enumerate()
        .map(|(index, paint)| {
            let world = state.world_transforms[paint.owner as usize];
            let (reference, clip) = match &paint.content {
                PreparedPaintContent::Coverage {
                    coverage,
                    brush,
                    clip,
                    ..
                } => (
                    SpatialPaintOutputReferenceV2::Coverage {
                        shape: SpatialShapeKeyV2::new(coverage_shape(coverage)),
                        brush: SpatialBrushKeyV2::new(*brush),
                    },
                    *clip,
                ),
                PreparedPaintContent::Image { image, clip, .. } => (
                    SpatialPaintOutputReferenceV2::Image {
                        image: SpatialImageKeyV2::new(*image),
                    },
                    *clip,
                ),
            };
            SpatialPaintOutputRecordV2::new(
                ordinal(index),
                world,
                world.determinant_raw(),
                output_aabb(state.world_aabbs.paints[index]),
                SpatialNodeKeyV2::new(paint.owner),
                reference,
                clip.map(SpatialClipKeyV2::new),
                paint.owner,
                paint.item_ordinal,
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn materialize_hits(state: &PreparedSpatialState) -> Box<[SpatialHitOutputRecordV2]> {
    state
        .hits
        .iter()
        .enumerate()
        .map(|(index, hit)| {
            let world = state.world_transforms[hit.owner as usize];
            SpatialHitOutputRecordV2::new(
                ordinal(index),
                world,
                world.determinant_raw(),
                output_aabb(state.world_aabbs.hits[index]),
                SpatialNodeKeyV2::new(hit.owner),
                SpatialShapeKeyV2::new(coverage_shape(&hit.coverage)),
                hit.clip.map(SpatialClipKeyV2::new),
                hit.owner,
                hit.item_ordinal,
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn materialize_semantics(state: &PreparedSpatialState) -> Box<[SpatialSemanticOutputRecordV2]> {
    state
        .semantics
        .iter()
        .enumerate()
        .map(|(index, semantic)| {
            let world = state.world_transforms[semantic.owner as usize];
            SpatialSemanticOutputRecordV2::new(
                ordinal(index),
                world,
                world.determinant_raw(),
                output_aabb(state.world_aabbs.semantics[index]),
                SpatialNodeKeyV2::new(semantic.owner),
                SpatialShapeKeyV2::new(semantic.shape),
                semantic.clip.map(SpatialClipKeyV2::new),
                semantic.owner,
                semantic.item_ordinal,
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn coverage_shape(coverage: &PreparedCoverage) -> u32 {
    match coverage {
        PreparedCoverage::Fill { shape, .. } | PreparedCoverage::RoundStroke { shape, .. } => {
            *shape
        }
    }
}

fn fixed_extent(value: i32) -> SpatialScalarV2 {
    SpatialScalarV2::checked_from_i32(value).expect("prepared base extent belongs to scalar domain")
}

const fn output_aabb(bounds: SpatialAabbV2) -> SpatialOutputAabbV2 {
    SpatialOutputAabbV2::new(
        bounds.is_empty(),
        bounds.min_x(),
        bounds.min_y(),
        bounds.max_x(),
        bounds.max_y(),
    )
}

fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("prepared output ordinal fits u32")
}
