//! Candidate scalar, extent, and determinant validation passes.

use super::common::{ordinal, output_error};
use crate::model::{Affine2V2, SpatialScalarV2};
use crate::output_aabb::SpatialOutputAabbV2;
use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
use crate::output_view::SpatialOutputV2;
use crate::resolve_error::{SpatialOutputErrorKindV2, SpatialResolveErrorV2};
use crate::vocabulary::SpatialExtentV2;

pub(super) fn validate_scalars(supplied: SpatialOutputV2<'_>) -> Result<(), SpatialResolveErrorV2> {
    for (index, row) in supplied.geometry().iter().enumerate() {
        let index = ordinal(index);
        validate_geometry_scalar(index, SpatialOutputFieldV2::BaseX, row.base_x(), false)?;
        validate_geometry_scalar(index, SpatialOutputFieldV2::BaseY, row.base_y(), false)?;
        validate_geometry_scalar(
            index,
            SpatialOutputFieldV2::BaseWidth,
            row.base_width(),
            true,
        )?;
        validate_geometry_scalar(
            index,
            SpatialOutputFieldV2::BaseHeight,
            row.base_height(),
            true,
        )?;
        validate_composite_scalars(
            SpatialOutputTableV2::Geometry,
            index,
            row.world_from_local(),
            row.world_aabb(),
        )?;
    }
    for (index, row) in supplied.clips().iter().enumerate() {
        validate_composite_scalars(
            SpatialOutputTableV2::Clip,
            ordinal(index),
            row.world_from_local(),
            row.primitive_world_aabb(),
        )?;
    }
    for (index, row) in supplied.paints().iter().enumerate() {
        validate_composite_scalars(
            SpatialOutputTableV2::Paint,
            ordinal(index),
            row.world_from_local(),
            row.world_aabb(),
        )?;
    }
    for (index, row) in supplied.hits().iter().enumerate() {
        validate_composite_scalars(
            SpatialOutputTableV2::Hit,
            ordinal(index),
            row.world_from_local(),
            row.world_aabb(),
        )?;
    }
    for (index, row) in supplied.semantics().iter().enumerate() {
        validate_composite_scalars(
            SpatialOutputTableV2::Semantic,
            ordinal(index),
            row.world_from_local(),
            row.world_aabb(),
        )?;
    }
    Ok(())
}

pub(super) fn validate_extents(supplied: SpatialOutputV2<'_>) -> Result<(), SpatialResolveErrorV2> {
    for (index, row) in supplied.geometry().iter().enumerate() {
        let index = ordinal(index);
        if row.base_width().raw() < 0 {
            return Err(output_error(
                SpatialOutputErrorKindV2::NegativeBaseExtent(SpatialExtentV2::Width),
                SpatialOutputTableV2::Geometry,
                index,
                SpatialOutputFieldV2::BaseWidth,
            ));
        }
        if row.base_height().raw() < 0 {
            return Err(output_error(
                SpatialOutputErrorKindV2::NegativeBaseExtent(SpatialExtentV2::Height),
                SpatialOutputTableV2::Geometry,
                index,
                SpatialOutputFieldV2::BaseHeight,
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_determinants(
    supplied: SpatialOutputV2<'_>,
) -> Result<(), SpatialResolveErrorV2> {
    for (index, row) in supplied.geometry().iter().enumerate() {
        validate_determinant(
            SpatialOutputTableV2::Geometry,
            ordinal(index),
            row.world_from_local(),
            row.world_determinant(),
        )?;
    }
    for (index, row) in supplied.clips().iter().enumerate() {
        validate_determinant(
            SpatialOutputTableV2::Clip,
            ordinal(index),
            row.world_from_local(),
            row.world_determinant(),
        )?;
    }
    for (index, row) in supplied.paints().iter().enumerate() {
        validate_determinant(
            SpatialOutputTableV2::Paint,
            ordinal(index),
            row.world_from_local(),
            row.world_determinant(),
        )?;
    }
    for (index, row) in supplied.hits().iter().enumerate() {
        validate_determinant(
            SpatialOutputTableV2::Hit,
            ordinal(index),
            row.world_from_local(),
            row.world_determinant(),
        )?;
    }
    for (index, row) in supplied.semantics().iter().enumerate() {
        validate_determinant(
            SpatialOutputTableV2::Semantic,
            ordinal(index),
            row.world_from_local(),
            row.world_determinant(),
        )?;
    }
    Ok(())
}

fn validate_geometry_scalar(
    index: u32,
    field: SpatialOutputFieldV2,
    value: SpatialScalarV2,
    require_integer: bool,
) -> Result<(), SpatialResolveErrorV2> {
    if !value.is_in_domain() || (require_integer && value.raw() % SpatialScalarV2::SCALE != 0) {
        return Err(output_error(
            SpatialOutputErrorKindV2::ScalarOutOfDomain,
            SpatialOutputTableV2::Geometry,
            index,
            field,
        ));
    }
    Ok(())
}

fn validate_composite_scalars(
    table: SpatialOutputTableV2,
    index: u32,
    world: Affine2V2,
    aabb: SpatialOutputAabbV2,
) -> Result<(), SpatialResolveErrorV2> {
    let values = [
        (SpatialOutputFieldV2::AffineA, world.a()),
        (SpatialOutputFieldV2::AffineB, world.b()),
        (SpatialOutputFieldV2::AffineC, world.c()),
        (SpatialOutputFieldV2::AffineD, world.d()),
        (SpatialOutputFieldV2::AffineTx, world.tx()),
        (SpatialOutputFieldV2::AffineTy, world.ty()),
        (SpatialOutputFieldV2::AabbMinX, aabb.min_x()),
        (SpatialOutputFieldV2::AabbMinY, aabb.min_y()),
        (SpatialOutputFieldV2::AabbMaxX, aabb.max_x()),
        (SpatialOutputFieldV2::AabbMaxY, aabb.max_y()),
    ];
    for (field, value) in values {
        if !value.is_in_domain() {
            return Err(output_error(
                SpatialOutputErrorKindV2::ScalarOutOfDomain,
                table,
                index,
                field,
            ));
        }
    }
    Ok(())
}

fn validate_determinant(
    table: SpatialOutputTableV2,
    index: u32,
    world: Affine2V2,
    supplied: i128,
) -> Result<(), SpatialResolveErrorV2> {
    let expected = world.determinant_raw();
    if expected == 0 || supplied != expected {
        return Err(output_error(
            SpatialOutputErrorKindV2::InvalidWorldDeterminant,
            table,
            index,
            SpatialOutputFieldV2::Determinant,
        ));
    }
    Ok(())
}
