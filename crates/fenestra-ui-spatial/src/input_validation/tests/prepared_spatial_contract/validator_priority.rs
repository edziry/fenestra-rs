use super::validator_clips::set_parent;
use super::validator_counts_keys::set_key;
use super::validator_projection::set_projection;
use super::validator_scalars_extents::set_scalar;
use super::validator_support::*;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;
use crate::vocabulary::SpatialExtentV2;

#[test]
fn adjacent_global_passes_complete_before_the_next_pass_begins() {
    let (prepared, mut rows) = rich_case();
    set_key(&mut rows, Table::Semantic, 1, 99);
    set_scalar(
        &mut rows,
        Table::Geometry,
        0,
        Field::BaseX,
        crate::model::SpatialScalarV2::MAX_RAW + 1,
    );
    expect_output_error(
        validate(prepared, &rows),
        Kind::KeyMismatch,
        output_location(Table::Semantic, 1, Field::Key),
    );

    let (prepared, mut rows) = rich_case();
    set_scalar(
        &mut rows,
        Table::Semantic,
        1,
        Field::AabbMaxY,
        crate::model::SpatialScalarV2::MAX_RAW + 1,
    );
    let mut geometry = GeometryRow::read(rows.geometry[0]);
    geometry.width = -S;
    rows.geometry[0] = geometry.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::ScalarOutOfDomain,
        output_location(Table::Semantic, 1, Field::AabbMaxY),
    );

    let (prepared, mut rows) = rich_case();
    let mut geometry = GeometryRow::read(rows.geometry[3]);
    geometry.height = -S;
    rows.geometry[3] = geometry.build();
    let mut root = GeometryRow::read(rows.geometry[0]);
    root.determinant += 1;
    rows.geometry[0] = root.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::NegativeBaseExtent(SpatialExtentV2::Height),
        output_location(Table::Geometry, 3, Field::BaseHeight),
    );

    let (prepared, mut rows) = rich_case();
    let mut semantic = ShapeItemRow::read_semantic(rows.semantics[1]);
    semantic.determinant += 1;
    rows.semantics[1] = semantic.build_semantic();
    let mut root = GeometryRow::read(rows.geometry[0]);
    root.aabb.1[0] += 1;
    rows.geometry[0] = root.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidWorldDeterminant,
        output_location(Table::Semantic, 1, Field::Determinant),
    );

    let (prepared, mut rows) = rich_case();
    let mut semantic = ShapeItemRow::read_semantic(rows.semantics[1]);
    semantic.aabb.1[3] -= 1;
    rows.semantics[1] = semantic.build_semantic();
    set_parent(&mut rows, 0, Some(0));
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidAabb,
        output_location(Table::Semantic, 1, Field::AabbMaxY),
    );

    let (prepared, mut rows) = rich_case();
    set_parent(&mut rows, 2, Some(2));
    set_projection(&mut rows, Table::Paint, 0, Field::StackOrdinal, 99);
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidClipChain,
        output_location(Table::Clip, 2, Field::Parent),
    );

    let (prepared, mut rows) = rich_case();
    set_projection(&mut rows, Table::Semantic, 1, Field::ItemOrdinal, 99);
    let mut clip = ClipRow::read(rows.clips[0]);
    clip.shape = 1;
    rows.clips[0] = clip.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidProjectionOrder,
        output_location(Table::Semantic, 1, Field::ItemOrdinal),
    );
}
