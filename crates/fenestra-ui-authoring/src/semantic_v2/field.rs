use fenestra_ui_ir::prototype::{
    SpatialClipAddressV2, SpatialCoverageRecipeV2, SpatialFieldV2, SpatialPointRecipeV2,
    SpatialTransformRecipeV2,
};

use crate::semantic::{InvalidRecord, Record};
use crate::vocabulary_v2::AnchorKindV2;

use super::catalog::SourceCatalog;
use super::value::FieldValue;

pub(super) fn push<T: FieldValue>(
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
    owner: u32,
    role: &str,
    field: SpatialFieldV2<T>,
) -> Result<(), InvalidRecord> {
    let anchor = catalog.anchor(field.span(), AnchorKindV2::SpatialField)?;
    records.push(Record::new(
        anchor,
        "spatial-field",
        format!(
            "owner={owner}|role={role}|value={}",
            field.value().field_value()
        ),
    )?);
    Ok(())
}

pub(super) fn point(
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
    owner: u32,
    x_role: &str,
    y_role: &str,
    point: SpatialPointRecipeV2,
) -> Result<(), InvalidRecord> {
    push(records, catalog, owner, x_role, point.x())?;
    push(records, catalog, owner, y_role, point.y())
}

pub(super) fn clip_address(
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
    owner: u32,
    address: SpatialClipAddressV2,
) -> Result<(), InvalidRecord> {
    push(records, catalog, owner, "clip-owner", address.owner())?;
    push(records, catalog, owner, "clip", address.clip())
}

pub(super) fn transform(
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
    owner: u32,
    transform: SpatialTransformRecipeV2,
) -> Result<(), InvalidRecord> {
    push(records, catalog, owner, "a", transform.a())?;
    push(records, catalog, owner, "b", transform.b())?;
    push(records, catalog, owner, "c", transform.c())?;
    push(records, catalog, owner, "d", transform.d())?;
    push(records, catalog, owner, "tx", transform.tx())?;
    push(records, catalog, owner, "ty", transform.ty())?;
    point(
        records,
        catalog,
        owner,
        "origin-x",
        "origin-y",
        transform.origin(),
    )
}

pub(super) fn coverage(
    records: &mut Vec<Record>,
    catalog: &SourceCatalog<'_>,
    owner: u32,
    coverage: SpatialCoverageRecipeV2,
) -> Result<(), InvalidRecord> {
    match coverage {
        SpatialCoverageRecipeV2::Fill { shape, .. } => {
            push(records, catalog, owner, "shape", shape)
        }
        SpatialCoverageRecipeV2::RoundStroke { shape, width } => {
            push(records, catalog, owner, "shape", shape)?;
            push(records, catalog, owner, "stroke-width", width)
        }
    }
}
