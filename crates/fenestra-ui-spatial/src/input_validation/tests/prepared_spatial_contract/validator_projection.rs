use super::validator_support::*;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;

#[test]
fn projection_tuples_are_checked_against_trusted_source_for_each_table() {
    for (table, count) in [Table::Paint, Table::Hit, Table::Semantic]
        .into_iter()
        .zip([3, 3, 2])
    {
        for index in 0..count {
            for (field, value) in [(Field::StackOrdinal, 99), (Field::ItemOrdinal, 99)] {
                let (prepared, mut rows) = rich_case();
                set_projection(&mut rows, table, index, field, value);
                expect_output_error(
                    validate(prepared, &rows),
                    Kind::InvalidProjectionOrder,
                    output_location(table, index as u32, field),
                );
            }
        }
    }

    let (prepared, mut rows) = rich_case();
    let mut paint = PaintRow::read(rows.paints[2]);
    paint.stack = 3;
    rows.paints[2] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidProjectionOrder,
        output_location(Table::Paint, 2, Field::StackOrdinal),
    );
}

#[test]
fn projection_is_table_then_record_and_stack_before_item() {
    let (prepared, mut rows) = rich_case();
    set_projection(&mut rows, Table::Paint, 1, Field::ItemOrdinal, 99);
    set_projection(&mut rows, Table::Hit, 0, Field::StackOrdinal, 99);
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidProjectionOrder,
        output_location(Table::Paint, 1, Field::ItemOrdinal),
    );

    let (prepared, mut rows) = rich_case();
    set_projection(&mut rows, Table::Hit, 0, Field::ItemOrdinal, 99);
    set_projection(&mut rows, Table::Hit, 1, Field::StackOrdinal, 99);
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidProjectionOrder,
        output_location(Table::Hit, 0, Field::ItemOrdinal),
    );

    let (prepared, mut rows) = rich_case();
    set_projection(&mut rows, Table::Semantic, 1, Field::StackOrdinal, 99);
    set_projection(&mut rows, Table::Semantic, 1, Field::ItemOrdinal, 99);
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidProjectionOrder,
        output_location(Table::Semantic, 1, Field::StackOrdinal),
    );

    for (earlier, later, earlier_last) in [
        (Table::Paint, Table::Hit, 2),
        (Table::Hit, Table::Semantic, 2),
    ] {
        let (prepared, mut rows) = rich_case();
        set_projection(&mut rows, earlier, earlier_last, Field::ItemOrdinal, 99);
        set_projection(&mut rows, later, 0, Field::StackOrdinal, 99);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidProjectionOrder,
            output_location(earlier, earlier_last as u32, Field::ItemOrdinal),
        );
    }
}

#[test]
fn candidate_owner_does_not_select_or_replace_the_trusted_projection_tuple() {
    let (prepared, mut rows) = rich_case();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.owner = 2;
    rows.paints[0] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, 0, Field::Owner),
    );

    let (prepared, mut rows) = rich_case();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.owner = 2;
    paint.stack = 2;
    rows.paints[0] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidProjectionOrder,
        output_location(Table::Paint, 0, Field::StackOrdinal),
    );
}

pub(super) fn set_projection(
    rows: &mut CandidateTables,
    table: Table,
    index: usize,
    field: Field,
    value: u32,
) {
    match table {
        Table::Paint => {
            let mut row = PaintRow::read(rows.paints[index]);
            if field == Field::StackOrdinal {
                row.stack = value
            } else {
                row.item = value
            }
            rows.paints[index] = row.build();
        }
        Table::Hit => {
            let mut row = ShapeItemRow::read_hit(rows.hits[index]);
            if field == Field::StackOrdinal {
                row.stack = value
            } else {
                row.item = value
            }
            rows.hits[index] = row.build_hit();
        }
        Table::Semantic => {
            let mut row = ShapeItemRow::read_semantic(rows.semantics[index]);
            if field == Field::StackOrdinal {
                row.stack = value
            } else {
                row.item = value
            }
            rows.semantics[index] = row.build_semantic();
        }
        _ => panic!("projection order applies only to item tables"),
    }
}

pub(super) fn set_owner(rows: &mut CandidateTables, table: Table, index: usize, owner: u32) {
    match table {
        Table::Clip => {
            let mut r = ClipRow::read(rows.clips[index]);
            r.owner = owner;
            rows.clips[index] = r.build();
        }
        Table::Paint => {
            let mut r = PaintRow::read(rows.paints[index]);
            r.owner = owner;
            rows.paints[index] = r.build();
        }
        Table::Hit => {
            let mut r = ShapeItemRow::read_hit(rows.hits[index]);
            r.owner = owner;
            rows.hits[index] = r.build_hit();
        }
        Table::Semantic => {
            let mut r = ShapeItemRow::read_semantic(rows.semantics[index]);
            r.owner = owner;
            rows.semantics[index] = r.build_semantic();
        }
        Table::Geometry => panic!("geometry has no owner"),
    }
}
