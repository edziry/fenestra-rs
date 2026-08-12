use super::super::super::model::{PreparedCoverage, PreparedPaintContent, PreparedSpatialState};
use super::common::{ordinal, output_error};
use crate::model::Affine2V2;
use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
use crate::output_item::SpatialPaintOutputReferenceV2;
use crate::output_view::SpatialOutputV2;
use crate::resolve_error::{SpatialOutputErrorKindV2, SpatialResolveErrorV2};

pub(super) fn validate_references(
    state: &PreparedSpatialState,
    supplied: SpatialOutputV2<'_>,
) -> Result<(), SpatialResolveErrorV2> {
    for (index, (row, trusted)) in supplied.clips().iter().zip(state.clips.iter()).enumerate() {
        validate_world(
            supplied,
            trusted.owner,
            row.world_from_local(),
            SpatialOutputTableV2::Clip,
            index,
        )?;
        same(
            row.owner().get(),
            trusted.owner,
            SpatialOutputTableV2::Clip,
            index,
            SpatialOutputFieldV2::Owner,
        )?;
        same(
            row.parent().map(|key| key.get()),
            trusted.parent,
            SpatialOutputTableV2::Clip,
            index,
            SpatialOutputFieldV2::Parent,
        )?;
        same(
            row.shape().get(),
            trusted.shape,
            SpatialOutputTableV2::Clip,
            index,
            SpatialOutputFieldV2::Shape,
        )?;
    }
    for (index, (row, trusted)) in supplied
        .paints()
        .iter()
        .zip(state.paints.iter())
        .enumerate()
    {
        validate_world(
            supplied,
            trusted.owner,
            row.world_from_local(),
            SpatialOutputTableV2::Paint,
            index,
        )?;
        same(
            row.owner().get(),
            trusted.owner,
            SpatialOutputTableV2::Paint,
            index,
            SpatialOutputFieldV2::Owner,
        )?;
        validate_paint_reference(row.reference(), &trusted.content, index)?;
        same(
            row.clip().map(|key| key.get()),
            paint_clip(&trusted.content),
            SpatialOutputTableV2::Paint,
            index,
            SpatialOutputFieldV2::Clip,
        )?;
    }
    for (index, (row, trusted)) in supplied.hits().iter().zip(state.hits.iter()).enumerate() {
        validate_world(
            supplied,
            trusted.owner,
            row.world_from_local(),
            SpatialOutputTableV2::Hit,
            index,
        )?;
        same(
            row.owner().get(),
            trusted.owner,
            SpatialOutputTableV2::Hit,
            index,
            SpatialOutputFieldV2::Owner,
        )?;
        same(
            row.shape().get(),
            coverage_shape(&trusted.coverage),
            SpatialOutputTableV2::Hit,
            index,
            SpatialOutputFieldV2::Shape,
        )?;
        same(
            row.clip().map(|key| key.get()),
            trusted.clip,
            SpatialOutputTableV2::Hit,
            index,
            SpatialOutputFieldV2::Clip,
        )?;
    }
    for (index, (row, trusted)) in supplied
        .semantics()
        .iter()
        .zip(state.semantics.iter())
        .enumerate()
    {
        validate_world(
            supplied,
            trusted.owner,
            row.world_from_local(),
            SpatialOutputTableV2::Semantic,
            index,
        )?;
        same(
            row.owner().get(),
            trusted.owner,
            SpatialOutputTableV2::Semantic,
            index,
            SpatialOutputFieldV2::Owner,
        )?;
        same(
            row.shape().get(),
            trusted.shape,
            SpatialOutputTableV2::Semantic,
            index,
            SpatialOutputFieldV2::Shape,
        )?;
        same(
            row.clip().map(|key| key.get()),
            trusted.clip,
            SpatialOutputTableV2::Semantic,
            index,
            SpatialOutputFieldV2::Clip,
        )?;
    }
    Ok(())
}

fn validate_world(
    supplied: SpatialOutputV2<'_>,
    owner: u32,
    actual: Affine2V2,
    table: SpatialOutputTableV2,
    index: usize,
) -> Result<(), SpatialResolveErrorV2> {
    let expected = supplied.geometry()[owner as usize].world_from_local();
    for (field, actual, expected) in [
        (SpatialOutputFieldV2::AffineA, actual.a(), expected.a()),
        (SpatialOutputFieldV2::AffineB, actual.b(), expected.b()),
        (SpatialOutputFieldV2::AffineC, actual.c(), expected.c()),
        (SpatialOutputFieldV2::AffineD, actual.d(), expected.d()),
        (SpatialOutputFieldV2::AffineTx, actual.tx(), expected.tx()),
        (SpatialOutputFieldV2::AffineTy, actual.ty(), expected.ty()),
    ] {
        if actual != expected {
            return Err(reference_error(table, index, field));
        }
    }
    Ok(())
}

fn validate_paint_reference(
    actual: SpatialPaintOutputReferenceV2,
    expected: &PreparedPaintContent,
    index: usize,
) -> Result<(), SpatialResolveErrorV2> {
    match (expected, actual) {
        (
            PreparedPaintContent::Coverage {
                coverage, brush, ..
            },
            SpatialPaintOutputReferenceV2::Coverage {
                shape,
                brush: actual_brush,
            },
        ) => {
            same(
                shape.get(),
                coverage_shape(coverage),
                SpatialOutputTableV2::Paint,
                index,
                SpatialOutputFieldV2::Shape,
            )?;
            same(
                actual_brush.get(),
                *brush,
                SpatialOutputTableV2::Paint,
                index,
                SpatialOutputFieldV2::Brush,
            )
        }
        (
            PreparedPaintContent::Image { image, .. },
            SpatialPaintOutputReferenceV2::Image { image: actual },
        ) => same(
            actual.get(),
            *image,
            SpatialOutputTableV2::Paint,
            index,
            SpatialOutputFieldV2::Image,
        ),
        (PreparedPaintContent::Coverage { .. }, _) => Err(reference_error(
            SpatialOutputTableV2::Paint,
            index,
            SpatialOutputFieldV2::Shape,
        )),
        (PreparedPaintContent::Image { .. }, _) => Err(reference_error(
            SpatialOutputTableV2::Paint,
            index,
            SpatialOutputFieldV2::Image,
        )),
    }
}

fn coverage_shape(coverage: &PreparedCoverage) -> u32 {
    match coverage {
        PreparedCoverage::Fill { shape, .. } | PreparedCoverage::RoundStroke { shape, .. } => {
            *shape
        }
    }
}

fn paint_clip(content: &PreparedPaintContent) -> Option<u32> {
    match content {
        PreparedPaintContent::Coverage { clip, .. } | PreparedPaintContent::Image { clip, .. } => {
            *clip
        }
    }
}

fn same<T: PartialEq>(
    actual: T,
    expected: T,
    table: SpatialOutputTableV2,
    index: usize,
    field: SpatialOutputFieldV2,
) -> Result<(), SpatialResolveErrorV2> {
    if actual != expected {
        Err(reference_error(table, index, field))
    } else {
        Ok(())
    }
}

fn reference_error(
    table: SpatialOutputTableV2,
    index: usize,
    field: SpatialOutputFieldV2,
) -> SpatialResolveErrorV2 {
    output_error(
        SpatialOutputErrorKindV2::InvalidReference,
        table,
        ordinal(index),
        field,
    )
}
