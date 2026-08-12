use super::validator_support::*;
use crate::model::SpatialScalarV2;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;
use crate::vocabulary::SpatialExtentV2;

const GEOMETRY_FIELDS: [Field; 14] = [
    Field::BaseX,
    Field::BaseY,
    Field::BaseWidth,
    Field::BaseHeight,
    Field::AffineA,
    Field::AffineB,
    Field::AffineC,
    Field::AffineD,
    Field::AffineTx,
    Field::AffineTy,
    Field::AabbMinX,
    Field::AabbMinY,
    Field::AabbMaxX,
    Field::AabbMaxY,
];
const CONTENT_FIELDS: [Field; 10] = [
    Field::AffineA,
    Field::AffineB,
    Field::AffineC,
    Field::AffineD,
    Field::AffineTx,
    Field::AffineTy,
    Field::AabbMinX,
    Field::AabbMinY,
    Field::AabbMaxX,
    Field::AabbMaxY,
];

#[test]
fn every_applicable_scalar_rejects_both_domain_edges_even_for_empty_bounds() {
    for table in Table::ALL {
        let fields: &[Field] = if table == Table::Geometry {
            &GEOMETRY_FIELDS
        } else {
            &CONTENT_FIELDS
        };
        for field in fields {
            for raw in [SpatialScalarV2::MIN_RAW - 1, SpatialScalarV2::MAX_RAW + 1] {
                let (prepared, mut rows) = rich_case();
                set_scalar(&mut rows, table, 0, *field, raw);
                expect_output_error(
                    validate(prepared, &rows),
                    Kind::ScalarOutOfDomain,
                    output_location(table, 0, *field),
                );
            }
        }
    }

    let (prepared, mut rows) = rich_case();
    let mut hit = ShapeItemRow::read_hit(rows.hits[0]);
    hit.aabb = (true, [SpatialScalarV2::MAX_RAW + 1, 0, 0, 0]);
    rows.hits[0] = hit.build_hit();
    expect_output_error(
        validate(prepared, &rows),
        Kind::ScalarOutOfDomain,
        output_location(Table::Hit, 0, Field::AabbMinX),
    );
}

#[test]
fn geometry_extents_must_be_exact_integers_before_nonnegative_semantics() {
    for (field, values) in [(Field::BaseWidth, [1, -1]), (Field::BaseHeight, [1, -1])] {
        for value in values {
            let (prepared, mut rows) = rich_case();
            set_scalar(&mut rows, Table::Geometry, 1, field, value);
            expect_output_error(
                validate(prepared, &rows),
                Kind::ScalarOutOfDomain,
                output_location(Table::Geometry, 1, field),
            );
        }
    }

    let (prepared, mut rows) = rich_case();
    set_scalar(&mut rows, Table::Geometry, 0, Field::BaseWidth, -1);
    set_scalar(
        &mut rows,
        Table::Geometry,
        0,
        Field::BaseHeight,
        SpatialScalarV2::MAX_RAW + 1,
    );
    expect_output_error(
        validate(prepared, &rows),
        Kind::ScalarOutOfDomain,
        output_location(Table::Geometry, 0, Field::BaseWidth),
    );
}

#[test]
fn scalar_pass_is_table_then_record_then_applicable_field() {
    for pair in GEOMETRY_FIELDS.windows(2) {
        let (prepared, mut rows) = rich_case();
        set_scalar(
            &mut rows,
            Table::Geometry,
            0,
            pair[0],
            SpatialScalarV2::MAX_RAW + 1,
        );
        set_scalar(
            &mut rows,
            Table::Geometry,
            0,
            pair[1],
            SpatialScalarV2::MIN_RAW - 1,
        );
        expect_output_error(
            validate(prepared, &rows),
            Kind::ScalarOutOfDomain,
            output_location(Table::Geometry, 0, pair[0]),
        );
    }
    for pair in CONTENT_FIELDS.windows(2) {
        let (prepared, mut rows) = rich_case();
        set_scalar(
            &mut rows,
            Table::Paint,
            0,
            pair[0],
            SpatialScalarV2::MAX_RAW + 1,
        );
        set_scalar(
            &mut rows,
            Table::Paint,
            0,
            pair[1],
            SpatialScalarV2::MIN_RAW - 1,
        );
        expect_output_error(
            validate(prepared, &rows),
            Kind::ScalarOutOfDomain,
            output_location(Table::Paint, 0, pair[0]),
        );
    }

    let (prepared, mut rows) = rich_case();
    set_scalar(
        &mut rows,
        Table::Geometry,
        0,
        Field::AabbMaxY,
        SpatialScalarV2::MAX_RAW + 1,
    );
    set_scalar(
        &mut rows,
        Table::Geometry,
        1,
        Field::BaseX,
        SpatialScalarV2::MAX_RAW + 1,
    );
    set_scalar(
        &mut rows,
        Table::Clip,
        0,
        Field::AffineA,
        SpatialScalarV2::MAX_RAW + 1,
    );
    expect_output_error(
        validate(prepared, &rows),
        Kind::ScalarOutOfDomain,
        output_location(Table::Geometry, 0, Field::AabbMaxY),
    );

    let (prepared, mut rows) = rich_case();
    set_scalar(
        &mut rows,
        Table::Semantic,
        1,
        Field::AabbMaxY,
        SpatialScalarV2::MAX_RAW + 1,
    );
    set_scalar(&mut rows, Table::Geometry, 0, Field::BaseWidth, -S);
    expect_output_error(
        validate(prepared, &rows),
        Kind::ScalarOutOfDomain,
        output_location(Table::Semantic, 1, Field::AabbMaxY),
    );

    for (pair, earlier_last) in Table::ALL.windows(2).zip([3, 2, 2, 2]) {
        let earlier_field = if pair[0] == Table::Geometry {
            Field::BaseX
        } else {
            Field::AffineA
        };
        let later_field = if pair[1] == Table::Geometry {
            Field::BaseX
        } else {
            Field::AffineA
        };
        let (prepared, mut rows) = rich_case();
        set_scalar(
            &mut rows,
            pair[0],
            earlier_last,
            earlier_field,
            SpatialScalarV2::MAX_RAW + 1,
        );
        set_scalar(
            &mut rows,
            pair[1],
            0,
            later_field,
            SpatialScalarV2::MIN_RAW - 1,
        );
        expect_output_error(
            validate(prepared, &rows),
            Kind::ScalarOutOfDomain,
            output_location(pair[0], earlier_last as u32, earlier_field),
        );
    }
}

#[test]
fn negative_extent_pass_is_record_major_width_then_height_and_accepts_zero() {
    for (field, extent) in [
        (Field::BaseWidth, SpatialExtentV2::Width),
        (Field::BaseHeight, SpatialExtentV2::Height),
    ] {
        let (prepared, mut rows) = rich_case();
        set_scalar(&mut rows, Table::Geometry, 1, field, -S);
        expect_output_error(
            validate(prepared, &rows),
            Kind::NegativeBaseExtent(extent),
            output_location(Table::Geometry, 1, field),
        );
    }

    let (prepared, mut rows) = rich_case();
    set_scalar(&mut rows, Table::Geometry, 0, Field::BaseWidth, -S);
    set_scalar(&mut rows, Table::Geometry, 0, Field::BaseHeight, -S);
    expect_output_error(
        validate(prepared, &rows),
        Kind::NegativeBaseExtent(SpatialExtentV2::Width),
        output_location(Table::Geometry, 0, Field::BaseWidth),
    );

    let (prepared, mut rows) = rich_case();
    set_scalar(&mut rows, Table::Geometry, 0, Field::BaseHeight, -S);
    set_scalar(&mut rows, Table::Geometry, 1, Field::BaseWidth, -S);
    expect_output_error(
        validate(prepared, &rows),
        Kind::NegativeBaseExtent(SpatialExtentV2::Height),
        output_location(Table::Geometry, 0, Field::BaseHeight),
    );

    let (prepared, mut rows) = rich_case();
    let mut geometry = GeometryRow::read(rows.geometry[0]);
    geometry.width = 0;
    geometry.height = 0;
    geometry.aabb = (false, [0, 0, 0, 0]);
    rows.geometry[0] = geometry.build();
    validate(prepared, &rows).expect("zero geometry is a closed point");
}

pub(super) fn set_scalar(
    rows: &mut CandidateTables,
    table: Table,
    index: usize,
    field: Field,
    raw: i64,
) {
    match table {
        Table::Geometry => {
            let mut row = GeometryRow::read(rows.geometry[index]);
            match field {
                Field::BaseX => row.x = raw,
                Field::BaseY => row.y = raw,
                Field::BaseWidth => row.width = raw,
                Field::BaseHeight => row.height = raw,
                _ => set_composite(&mut row.world, &mut row.aabb, field, raw),
            }
            rows.geometry[index] = row.build();
        }
        Table::Clip => {
            let mut row = ClipRow::read(rows.clips[index]);
            set_composite(&mut row.world, &mut row.aabb, field, raw);
            rows.clips[index] = row.build();
        }
        Table::Paint => {
            let mut row = PaintRow::read(rows.paints[index]);
            set_composite(&mut row.world, &mut row.aabb, field, raw);
            rows.paints[index] = row.build();
        }
        Table::Hit => {
            let mut row = ShapeItemRow::read_hit(rows.hits[index]);
            set_composite(&mut row.world, &mut row.aabb, field, raw);
            rows.hits[index] = row.build_hit();
        }
        Table::Semantic => {
            let mut row = ShapeItemRow::read_semantic(rows.semantics[index]);
            set_composite(&mut row.world, &mut row.aabb, field, raw);
            rows.semantics[index] = row.build_semantic();
        }
    }
}

fn set_composite(world: &mut [i64; 6], aabb: &mut (bool, [i64; 4]), field: Field, raw: i64) {
    let index = match field {
        Field::AffineA => Some(0),
        Field::AffineB => Some(1),
        Field::AffineC => Some(2),
        Field::AffineD => Some(3),
        Field::AffineTx => Some(4),
        Field::AffineTy => Some(5),
        _ => None,
    };
    if let Some(index) = index {
        world[index] = raw;
        return;
    }
    let index = match field {
        Field::AabbMinX => 0,
        Field::AabbMinY => 1,
        Field::AabbMaxX => 2,
        Field::AabbMaxY => 3,
        _ => panic!("not a scalar field"),
    };
    aabb.1[index] = raw;
}
