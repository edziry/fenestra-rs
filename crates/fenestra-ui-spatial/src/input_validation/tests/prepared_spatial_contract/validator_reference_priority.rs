use super::validator_support::*;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;

#[test]
fn reference_pass_is_table_then_record() {
    let (prepared, mut rows) = rich_case();
    let mut clip = ClipRow::read(rows.clips[2]);
    clip.shape = 4;
    rows.clips[2] = clip.build();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.clip = Some(1);
    rows.paints[0] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Clip, 2, Field::Shape),
    );

    let (prepared, mut rows) = rich_case();
    let mut paint = PaintRow::read(rows.paints[2]);
    paint.clip = Some(0);
    rows.paints[2] = paint.build();
    let mut hit = ShapeItemRow::read_hit(rows.hits[0]);
    hit.clip = Some(1);
    rows.hits[0] = hit.build_hit();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, 2, Field::Clip),
    );

    let (prepared, mut rows) = rich_case();
    let mut hit = ShapeItemRow::read_hit(rows.hits[2]);
    hit.clip = Some(1);
    rows.hits[2] = hit.build_hit();
    let mut semantic = ShapeItemRow::read_semantic(rows.semantics[0]);
    semantic.clip = Some(1);
    rows.semantics[0] = semantic.build_semantic();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Hit, 2, Field::Clip),
    );

    let (prepared, mut rows) = rich_case();
    let mut p0 = PaintRow::read(rows.paints[0]);
    p0.clip = Some(1);
    rows.paints[0] = p0.build();
    let mut p1 = PaintRow::read(rows.paints[1]);
    p1.reference = image(0);
    rows.paints[1] = p1.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, 0, Field::Clip),
    );

    for table in [Table::Clip, Table::Hit, Table::Semantic] {
        let (prepared, mut rows) = rich_case();
        poison_shape(&mut rows, table, 0);
        let last = if table == Table::Semantic { 1 } else { 2 };
        poison_shape(&mut rows, table, last);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidReference,
            output_location(table, 0, Field::Shape),
        );
    }
}

#[test]
fn reference_pass_uses_applicable_field_order_within_each_row() {
    for table in [Table::Paint, Table::Hit, Table::Semantic] {
        let (prepared, mut rows) = rich_case();
        poison_owner_shape_clip(&mut rows, table);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidReference,
            output_location(table, 0, Field::Owner),
        );
    }

    for table in [Table::Hit, Table::Semantic] {
        let (prepared, mut rows) = rich_case();
        poison_shape_clip(&mut rows, table);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidReference,
            output_location(table, 0, Field::Shape),
        );
    }

    let (prepared, mut rows) = rich_case();
    let mut clip = ClipRow::read(rows.clips[2]);
    clip.parent = Some(0);
    clip.shape = u32::MAX;
    rows.clips[2] = clip.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Clip, 2, Field::Parent),
    );

    let (prepared, mut rows) = rich_case();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.reference = coverage(0, u32::MAX);
    paint.clip = Some(u32::MAX);
    rows.paints[0] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, 0, Field::Brush),
    );
}

#[test]
fn expected_optional_references_cannot_be_replaced_by_none() {
    let (prepared, mut rows) = rich_case();
    let mut clip = ClipRow::read(rows.clips[2]);
    clip.parent = None;
    rows.clips[2] = clip.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Clip, 2, Field::Parent),
    );

    for table in [Table::Paint, Table::Hit, Table::Semantic] {
        let (prepared, mut rows) = rich_case();
        remove_terminal_clip(&mut rows, table);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidReference,
            output_location(table, 0, Field::Clip),
        );
    }
}

fn poison_shape(rows: &mut CandidateTables, table: Table, index: usize) {
    match table {
        Table::Clip => {
            let mut row = ClipRow::read(rows.clips[index]);
            row.shape = if row.shape == 0 { 1 } else { 0 };
            rows.clips[index] = row.build();
        }
        Table::Hit => {
            let mut row = ShapeItemRow::read_hit(rows.hits[index]);
            row.shape = if row.shape == 0 { 1 } else { 0 };
            rows.hits[index] = row.build_hit();
        }
        Table::Semantic => {
            let mut row = ShapeItemRow::read_semantic(rows.semantics[index]);
            row.shape = if row.shape == 0 { 1 } else { 0 };
            rows.semantics[index] = row.build_semantic();
        }
        _ => unreachable!(),
    }
}

fn remove_terminal_clip(rows: &mut CandidateTables, table: Table) {
    match table {
        Table::Paint => {
            let mut row = PaintRow::read(rows.paints[0]);
            row.clip = None;
            rows.paints[0] = row.build();
        }
        Table::Hit => {
            let mut row = ShapeItemRow::read_hit(rows.hits[0]);
            row.clip = None;
            rows.hits[0] = row.build_hit();
        }
        Table::Semantic => {
            let mut row = ShapeItemRow::read_semantic(rows.semantics[0]);
            row.clip = None;
            rows.semantics[0] = row.build_semantic();
        }
        _ => unreachable!(),
    }
}

fn poison_owner_shape_clip(rows: &mut CandidateTables, table: Table) {
    match table {
        Table::Paint => {
            let mut row = PaintRow::read(rows.paints[0]);
            row.owner = u32::MAX;
            row.reference = coverage(u32::MAX, u32::MAX);
            row.clip = Some(u32::MAX);
            rows.paints[0] = row.build();
        }
        Table::Hit => {
            let mut row = ShapeItemRow::read_hit(rows.hits[0]);
            row.owner = u32::MAX;
            row.shape = u32::MAX;
            row.clip = Some(u32::MAX);
            rows.hits[0] = row.build_hit();
        }
        Table::Semantic => {
            let mut row = ShapeItemRow::read_semantic(rows.semantics[0]);
            row.owner = u32::MAX;
            row.shape = u32::MAX;
            row.clip = Some(u32::MAX);
            rows.semantics[0] = row.build_semantic();
        }
        _ => unreachable!(),
    }
}

fn poison_shape_clip(rows: &mut CandidateTables, table: Table) {
    match table {
        Table::Hit => {
            let mut row = ShapeItemRow::read_hit(rows.hits[0]);
            row.shape = u32::MAX;
            row.clip = Some(u32::MAX);
            rows.hits[0] = row.build_hit();
        }
        Table::Semantic => {
            let mut row = ShapeItemRow::read_semantic(rows.semantics[0]);
            row.shape = u32::MAX;
            row.clip = Some(u32::MAX);
            rows.semantics[0] = row.build_semantic();
        }
        _ => unreachable!(),
    }
}
