use super::validator_support::*;
use crate::error::SpatialErrorLocationV2;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;

#[test]
fn every_table_count_is_checked_in_complete_table_order() {
    for table in Table::ALL {
        for remove in [true, false] {
            let (prepared, mut rows) = rich_case();
            change_count(&mut rows, table, remove);
            expect_output_error(
                validate(prepared, &rows),
                Kind::RecordCountMismatch,
                SpatialErrorLocationV2::Output { table },
            );
        }
    }

    for pair in Table::ALL.windows(2) {
        let (prepared, mut rows) = rich_case();
        change_count(&mut rows, pair[0], true);
        change_count(&mut rows, pair[1], true);
        expect_output_error(
            validate(prepared, &rows),
            Kind::RecordCountMismatch,
            SpatialErrorLocationV2::Output { table: pair[0] },
        );
    }
}

#[test]
fn complete_count_pass_precedes_the_latest_dense_key_fault() {
    let (prepared, mut rows) = rich_case();
    rows.semantics.pop();
    let mut geometry = GeometryRow::read(rows.geometry[0]);
    geometry.key = u32::MAX;
    rows.geometry[0] = geometry.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::RecordCountMismatch,
        SpatialErrorLocationV2::Output {
            table: Table::Semantic,
        },
    );
}

#[test]
fn dense_keys_use_trusted_table_and_row_ordinals() {
    for (table, count) in Table::ALL.into_iter().zip([4, 3, 3, 3, 2]) {
        for index in 0..count {
            let (prepared, mut rows) = rich_case();
            set_key(&mut rows, table, index, u32::MAX);
            expect_output_error(
                validate(prepared, &rows),
                Kind::KeyMismatch,
                output_location(table, index as u32, Field::Key),
            );
        }
    }

    let (prepared, mut rows) = rich_case();
    set_key(&mut rows, Table::Geometry, 1, 99);
    set_key(&mut rows, Table::Clip, 0, 99);
    expect_output_error(
        validate(prepared, &rows),
        Kind::KeyMismatch,
        output_location(Table::Geometry, 1, Field::Key),
    );

    let (prepared, mut rows) = rich_case();
    set_key(&mut rows, Table::Paint, 0, 99);
    set_key(&mut rows, Table::Paint, 1, 99);
    expect_output_error(
        validate(prepared, &rows),
        Kind::KeyMismatch,
        output_location(Table::Paint, 0, Field::Key),
    );

    for (pair, earlier_last) in Table::ALL.windows(2).zip([3, 2, 2, 2]) {
        let (prepared, mut rows) = rich_case();
        set_key(&mut rows, pair[0], earlier_last, u32::MAX);
        set_key(&mut rows, pair[1], 0, u32::MAX);
        expect_output_error(
            validate(prepared, &rows),
            Kind::KeyMismatch,
            output_location(pair[0], earlier_last as u32, Field::Key),
        );
    }
}

#[test]
fn dense_keys_are_rejected_in_supplied_order_without_sorting_or_repair() {
    let (prepared, mut rows) = rich_case();
    rows.paints.swap(0, 1);
    expect_output_error(
        validate(prepared, &rows),
        Kind::KeyMismatch,
        output_location(Table::Paint, 0, Field::Key),
    );
}

#[test]
fn complete_key_pass_precedes_the_first_scalar_fault() {
    let (prepared, mut rows) = rich_case();
    let last = rows.semantics.len() - 1;
    set_key(&mut rows, Table::Semantic, last, u32::MAX);
    let mut geometry = GeometryRow::read(rows.geometry[0]);
    geometry.x = crate::model::SpatialScalarV2::MAX_RAW + 1;
    rows.geometry[0] = geometry.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::KeyMismatch,
        output_location(Table::Semantic, 1, Field::Key),
    );
}

fn change_count(rows: &mut CandidateTables, table: Table, remove: bool) {
    macro_rules! change {
        ($rows:expr) => {
            if remove {
                let _ = $rows.pop();
            } else {
                $rows.push($rows[0]);
            }
        };
    }
    match table {
        Table::Geometry => change!(rows.geometry),
        Table::Clip => change!(rows.clips),
        Table::Paint => change!(rows.paints),
        Table::Hit => change!(rows.hits),
        Table::Semantic => change!(rows.semantics),
    }
}

pub(super) fn set_key(rows: &mut CandidateTables, table: Table, index: usize, key: u32) {
    match table {
        Table::Geometry => {
            let mut row = GeometryRow::read(rows.geometry[index]);
            row.key = key;
            rows.geometry[index] = row.build();
        }
        Table::Clip => {
            let mut row = ClipRow::read(rows.clips[index]);
            row.key = key;
            rows.clips[index] = row.build();
        }
        Table::Paint => {
            let mut row = PaintRow::read(rows.paints[index]);
            row.key = key;
            rows.paints[index] = row.build();
        }
        Table::Hit => {
            let mut row = ShapeItemRow::read_hit(rows.hits[index]);
            row.key = key;
            rows.hits[index] = row.build_hit();
        }
        Table::Semantic => {
            let mut row = ShapeItemRow::read_semantic(rows.semantics[index]);
            row.key = key;
            rows.semantics[index] = row.build_semantic();
        }
    }
}
