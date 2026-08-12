//! Candidate table-shape validation passes.

use super::super::super::model::PreparedSpatialState;
use super::common::{count_error, ordinal, output_error};
use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
use crate::output_view::SpatialOutputV2;
use crate::resolve_error::{SpatialOutputErrorKindV2, SpatialResolveErrorV2};

pub(super) fn validate_counts(
    state: &PreparedSpatialState,
    supplied: SpatialOutputV2<'_>,
) -> Result<(), SpatialResolveErrorV2> {
    let counts = [
        (
            SpatialOutputTableV2::Geometry,
            state.base_geometry.len(),
            supplied.geometry().len(),
        ),
        (
            SpatialOutputTableV2::Clip,
            state.clips.len(),
            supplied.clips().len(),
        ),
        (
            SpatialOutputTableV2::Paint,
            state.paints.len(),
            supplied.paints().len(),
        ),
        (
            SpatialOutputTableV2::Hit,
            state.hits.len(),
            supplied.hits().len(),
        ),
        (
            SpatialOutputTableV2::Semantic,
            state.semantics.len(),
            supplied.semantics().len(),
        ),
    ];
    for (table, expected, actual) in counts {
        if actual != expected {
            return Err(count_error(table));
        }
    }
    Ok(())
}

pub(super) fn validate_keys(supplied: SpatialOutputV2<'_>) -> Result<(), SpatialResolveErrorV2> {
    validate_table_keys(
        SpatialOutputTableV2::Geometry,
        supplied.geometry().iter().map(|row| row.key().get()),
    )?;
    validate_table_keys(
        SpatialOutputTableV2::Clip,
        supplied.clips().iter().map(|row| row.key().get()),
    )?;
    validate_table_keys(
        SpatialOutputTableV2::Paint,
        supplied.paints().iter().map(|row| row.key()),
    )?;
    validate_table_keys(
        SpatialOutputTableV2::Hit,
        supplied.hits().iter().map(|row| row.key()),
    )?;
    validate_table_keys(
        SpatialOutputTableV2::Semantic,
        supplied.semantics().iter().map(|row| row.key()),
    )
}

fn validate_table_keys(
    table: SpatialOutputTableV2,
    keys: impl Iterator<Item = u32>,
) -> Result<(), SpatialResolveErrorV2> {
    for (index, key) in keys.enumerate() {
        let expected = ordinal(index);
        if key != expected {
            return Err(output_error(
                SpatialOutputErrorKindV2::KeyMismatch,
                table,
                expected,
                SpatialOutputFieldV2::Key,
            ));
        }
    }
    Ok(())
}
