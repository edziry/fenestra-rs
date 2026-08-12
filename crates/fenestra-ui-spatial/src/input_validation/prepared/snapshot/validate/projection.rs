use super::super::super::model::PreparedSpatialState;
use super::common::{ordinal, output_error};
use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
use crate::output_view::SpatialOutputV2;
use crate::resolve_error::{SpatialOutputErrorKindV2, SpatialResolveErrorV2};

pub(super) fn validate_projection_order(
    state: &PreparedSpatialState,
    supplied: SpatialOutputV2<'_>,
) -> Result<(), SpatialResolveErrorV2> {
    for (index, (row, trusted)) in supplied
        .paints()
        .iter()
        .zip(state.paints.iter())
        .enumerate()
    {
        validate_tuple(
            SpatialOutputTableV2::Paint,
            index,
            row.stack_ordinal(),
            row.item_ordinal(),
            trusted.owner,
            trusted.item_ordinal,
        )?;
    }
    for (index, (row, trusted)) in supplied.hits().iter().zip(state.hits.iter()).enumerate() {
        validate_tuple(
            SpatialOutputTableV2::Hit,
            index,
            row.stack_ordinal(),
            row.item_ordinal(),
            trusted.owner,
            trusted.item_ordinal,
        )?;
    }
    for (index, (row, trusted)) in supplied
        .semantics()
        .iter()
        .zip(state.semantics.iter())
        .enumerate()
    {
        validate_tuple(
            SpatialOutputTableV2::Semantic,
            index,
            row.stack_ordinal(),
            row.item_ordinal(),
            trusted.owner,
            trusted.item_ordinal,
        )?;
    }
    Ok(())
}

fn validate_tuple(
    table: SpatialOutputTableV2,
    index: usize,
    stack: u32,
    item: u32,
    expected_stack: u32,
    expected_item: u32,
) -> Result<(), SpatialResolveErrorV2> {
    if stack != expected_stack {
        return Err(projection_error(
            table,
            index,
            SpatialOutputFieldV2::StackOrdinal,
        ));
    }
    if item != expected_item {
        return Err(projection_error(
            table,
            index,
            SpatialOutputFieldV2::ItemOrdinal,
        ));
    }
    Ok(())
}

fn projection_error(
    table: SpatialOutputTableV2,
    index: usize,
    field: SpatialOutputFieldV2,
) -> SpatialResolveErrorV2 {
    output_error(
        SpatialOutputErrorKindV2::InvalidProjectionOrder,
        table,
        ordinal(index),
        field,
    )
}
