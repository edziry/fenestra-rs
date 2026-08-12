use super::validator_scalars_extents::set_scalar;
use super::validator_support::*;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;
use crate::vocabulary::SpatialExtentV2;

#[test]
fn every_table_rejects_stale_and_singular_stored_determinants() {
    for table in Table::ALL {
        for singular in [false, true] {
            let (prepared, mut rows) = rich_case();
            set_determinant_fault(&mut rows, table, 0, singular);
            expect_output_error(
                validate(prepared, &rows),
                Kind::InvalidWorldDeterminant,
                output_location(table, 0, Field::Determinant),
            );
        }
    }
}

#[test]
fn determinant_pass_is_table_then_record_and_follows_all_extents() {
    let (prepared, mut rows) = rich_case();
    set_determinant_fault(&mut rows, Table::Geometry, 1, false);
    set_determinant_fault(&mut rows, Table::Clip, 0, false);
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidWorldDeterminant,
        output_location(Table::Geometry, 1, Field::Determinant),
    );

    let (prepared, mut rows) = rich_case();
    set_determinant_fault(&mut rows, Table::Paint, 0, false);
    set_determinant_fault(&mut rows, Table::Paint, 1, false);
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidWorldDeterminant,
        output_location(Table::Paint, 0, Field::Determinant),
    );

    let (prepared, mut rows) = rich_case();
    set_scalar(&mut rows, Table::Geometry, 3, Field::BaseHeight, -S);
    set_determinant_fault(&mut rows, Table::Geometry, 0, false);
    expect_output_error(
        validate(prepared, &rows),
        Kind::NegativeBaseExtent(SpatialExtentV2::Height),
        output_location(Table::Geometry, 3, Field::BaseHeight),
    );

    for (pair, earlier_last) in Table::ALL.windows(2).zip([3, 2, 2, 2]) {
        let (prepared, mut rows) = rich_case();
        set_determinant_fault(&mut rows, pair[0], earlier_last, false);
        set_determinant_fault(&mut rows, pair[1], 0, false);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidWorldDeterminant,
            output_location(pair[0], earlier_last as u32, Field::Determinant),
        );
    }
}

#[test]
fn complete_determinant_pass_precedes_the_first_aabb_mismatch() {
    let (prepared, mut rows) = rich_case();
    set_determinant_fault(&mut rows, Table::Semantic, 1, false);
    let mut geometry = GeometryRow::read(rows.geometry[0]);
    geometry.aabb.1[0] += 1;
    rows.geometry[0] = geometry.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidWorldDeterminant,
        output_location(Table::Semantic, 1, Field::Determinant),
    );
}

fn set_determinant_fault(rows: &mut CandidateTables, table: Table, index: usize, singular: bool) {
    let transform = |world: &mut [i64; 6], determinant: &mut i128| {
        if singular {
            *world = [S, 0, S, 0, 0, 0];
            *determinant = 0;
        } else {
            *determinant += 1;
        }
    };
    match table {
        Table::Geometry => {
            let mut row = GeometryRow::read(rows.geometry[index]);
            transform(&mut row.world, &mut row.determinant);
            rows.geometry[index] = row.build();
        }
        Table::Clip => {
            let mut row = ClipRow::read(rows.clips[index]);
            transform(&mut row.world, &mut row.determinant);
            rows.clips[index] = row.build();
        }
        Table::Paint => {
            let mut row = PaintRow::read(rows.paints[index]);
            transform(&mut row.world, &mut row.determinant);
            rows.paints[index] = row.build();
        }
        Table::Hit => {
            let mut row = ShapeItemRow::read_hit(rows.hits[index]);
            transform(&mut row.world, &mut row.determinant);
            rows.hits[index] = row.build_hit();
        }
        Table::Semantic => {
            let mut row = ShapeItemRow::read_semantic(rows.semantics[index]);
            transform(&mut row.world, &mut row.determinant);
            rows.semantics[index] = row.build_semantic();
        }
    }
}
