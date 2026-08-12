//! Exact candidate world-AABB derivation and comparison.

use super::super::super::model::PreparedSpatialState;
use super::common::{ordinal, output_error};
use crate::aabb::SpatialAabbV2;
use crate::model::{Affine2V2, SpatialScalarV2};
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::output_aabb::SpatialOutputAabbV2;
use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
use crate::output_view::SpatialOutputV2;
use crate::resolve_error::{SpatialOutputErrorKindV2, SpatialResolveErrorV2};

pub(super) fn validate_aabbs(
    state: &PreparedSpatialState,
    supplied: SpatialOutputV2<'_>,
) -> Result<Vec<SpatialAabbV2>, SpatialResolveErrorV2> {
    for (index, row) in supplied.geometry().iter().enumerate() {
        validate_one(
            SpatialOutputTableV2::Geometry,
            ordinal(index),
            row.world_from_local(),
            geometry_local_bounds(row.base_width(), row.base_height()),
            row.world_aabb(),
        )?;
    }

    let mut primitive_clips = Vec::with_capacity(supplied.clips().len());
    for (index, row) in supplied.clips().iter().enumerate() {
        let source = &state.clips[index];
        let local = trusted_shape(state, source.shape).fill_clip_bounds;
        primitive_clips.push(validate_one(
            SpatialOutputTableV2::Clip,
            ordinal(index),
            row.world_from_local(),
            local,
            row.primitive_world_aabb(),
        )?);
    }

    for (index, row) in supplied.paints().iter().enumerate() {
        validate_one(
            SpatialOutputTableV2::Paint,
            ordinal(index),
            row.world_from_local(),
            state.paints[index].local_bounds,
            row.world_aabb(),
        )?;
    }
    for (index, row) in supplied.hits().iter().enumerate() {
        validate_one(
            SpatialOutputTableV2::Hit,
            ordinal(index),
            row.world_from_local(),
            state.hits[index].local_bounds,
            row.world_aabb(),
        )?;
    }
    for (index, row) in supplied.semantics().iter().enumerate() {
        let source = &state.semantics[index];
        let local = trusted_shape(state, source.shape).fill_clip_bounds;
        validate_one(
            SpatialOutputTableV2::Semantic,
            ordinal(index),
            row.world_from_local(),
            local,
            row.world_aabb(),
        )?;
    }
    Ok(primitive_clips)
}

fn validate_one(
    table: SpatialOutputTableV2,
    index: u32,
    world: Affine2V2,
    local: SpatialAabbV2,
    supplied: SpatialOutputAabbV2,
) -> Result<SpatialAabbV2, SpatialResolveErrorV2> {
    let expected = world.checked_transform_aabb(local).map_err(|operation| {
        output_error(
            SpatialOutputErrorKindV2::InvalidAabb,
            table,
            index,
            aabb_operation_field(operation),
        )
    })?;
    let fields_match = [
        (
            SpatialOutputFieldV2::AabbEmpty,
            supplied.is_empty() == expected.is_empty(),
        ),
        (
            SpatialOutputFieldV2::AabbMinX,
            supplied.min_x() == expected.min_x(),
        ),
        (
            SpatialOutputFieldV2::AabbMinY,
            supplied.min_y() == expected.min_y(),
        ),
        (
            SpatialOutputFieldV2::AabbMaxX,
            supplied.max_x() == expected.max_x(),
        ),
        (
            SpatialOutputFieldV2::AabbMaxY,
            supplied.max_y() == expected.max_y(),
        ),
    ];
    for (field, matches) in fields_match {
        if !matches {
            return Err(output_error(
                SpatialOutputErrorKindV2::InvalidAabb,
                table,
                index,
                field,
            ));
        }
    }
    Ok(expected)
}

fn geometry_local_bounds(width: SpatialScalarV2, height: SpatialScalarV2) -> SpatialAabbV2 {
    let zero = SpatialScalarV2::new(0);
    SpatialAabbV2::from_edges(zero, zero, width, height)
        .expect("candidate extent passes establish canonical local geometry")
}

fn trusted_shape(
    state: &PreparedSpatialState,
    shape: u32,
) -> &super::super::super::model::PreparedShapePlan {
    state
        .shapes
        .get(usize::try_from(shape).expect("validated shape key fits usize"))
        .expect("prepared source references an existing shape")
}

fn aabb_operation_field(operation: SpatialArithmeticOperationV2) -> SpatialOutputFieldV2 {
    match operation {
        SpatialArithmeticOperationV2::AabbMinX => SpatialOutputFieldV2::AabbMinX,
        SpatialArithmeticOperationV2::AabbMinY => SpatialOutputFieldV2::AabbMinY,
        SpatialArithmeticOperationV2::AabbMaxX => SpatialOutputFieldV2::AabbMaxX,
        SpatialArithmeticOperationV2::AabbMaxY => SpatialOutputFieldV2::AabbMaxY,
        _ => unreachable!("AABB projection returns only AABB edge operations"),
    }
}
