use super::support::{cross_axis_empty_owned, requested_limits, zero_call_engine};
use super::validator_support::*;
use super::*;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;

#[test]
fn every_non_geometry_affine_component_matches_trusted_owner_geometry() {
    let fields = [
        Field::AffineA,
        Field::AffineB,
        Field::AffineC,
        Field::AffineD,
        Field::AffineTx,
        Field::AffineTy,
    ];
    for table in [Table::Clip, Table::Paint, Table::Hit, Table::Semantic] {
        for (component, field) in fields.into_iter().enumerate() {
            let (prepared, mut rows) = cross_axis_case();
            change_world(&mut rows, table, 0, component, 1);
            expect_output_error(
                validate(prepared, &rows),
                Kind::InvalidReference,
                output_location(table, 0, field),
            );
        }
    }

    for pair in fields.windows(2) {
        let (prepared, mut rows) = cross_axis_case();
        change_world(&mut rows, Table::Paint, 0, affine_index(pair[0]), 1);
        change_world(&mut rows, Table::Paint, 0, affine_index(pair[1]), 1);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidReference,
            output_location(Table::Paint, 0, pair[0]),
        );
    }
}

#[test]
fn trusted_source_owner_selects_geometry_before_candidate_owner_is_checked() {
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
    paint.world = GeometryRow::read(rows.geometry[2]).world;
    paint.determinant = affine(paint.world).determinant_raw();
    paint.aabb = (false, [2 * S, 14 * S, 5 * S, 18 * S]);
    rows.paints[0] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, 0, Field::AffineTx),
    );
}

#[test]
fn every_reference_rejects_wrong_and_maximum_keys_without_dereferencing() {
    for (field, wrong) in [(Field::Owner, 0), (Field::Parent, 0), (Field::Shape, 1)] {
        for value in [wrong, u32::MAX] {
            expect_clip_reference(field, value);
        }
    }
    for (index, field, wrong) in [
        (0, Field::Owner, 2),
        (0, Field::Shape, 1),
        (0, Field::Brush, 1),
        (0, Field::Clip, 1),
        (1, Field::Image, 0),
        (1, Field::Clip, 0),
    ] {
        for value in [wrong, u32::MAX] {
            expect_paint_reference(index, field, value);
        }
    }
    for (table, field, wrong) in [
        (Table::Hit, Field::Owner, 1),
        (Table::Hit, Field::Shape, 2),
        (Table::Hit, Field::Clip, 1),
        (Table::Semantic, Field::Owner, 2),
        (Table::Semantic, Field::Shape, 1),
        (Table::Semantic, Field::Clip, 1),
    ] {
        for value in [wrong, u32::MAX] {
            expect_item_reference(table, field, value);
        }
    }
}

#[test]
fn paint_variant_mismatch_uses_the_expected_source_variant_field() {
    let (prepared, mut rows) = rich_case();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.reference = image(0);
    rows.paints[0] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, 0, Field::Shape),
    );

    let (prepared, mut rows) = rich_case();
    let mut paint = PaintRow::read(rows.paints[1]);
    paint.reference = coverage(0, 0);
    rows.paints[1] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, 1, Field::Image),
    );
}

#[test]
fn reference_fields_use_the_documented_order_within_each_row() {
    let (prepared, mut rows) = rich_case();
    let mut clip = ClipRow::read(rows.clips[1]);
    clip.owner = u32::MAX;
    clip.parent = Some(u32::MAX);
    clip.shape = u32::MAX;
    rows.clips[1] = clip.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Clip, 1, Field::Owner),
    );

    let (prepared, mut rows) = rich_case();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.reference = coverage(u32::MAX, u32::MAX);
    paint.clip = Some(u32::MAX);
    rows.paints[0] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, 0, Field::Shape),
    );

    let (prepared, mut rows) = rich_case();
    let mut paint = PaintRow::read(rows.paints[1]);
    paint.reference = image(u32::MAX);
    paint.clip = Some(u32::MAX);
    rows.paints[1] = paint.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, 1, Field::Image),
    );
}

fn expect_clip_reference(field: Field, value: u32) {
    let (prepared, mut rows) = rich_case();
    let index = if field == Field::Parent { 2 } else { 0 };
    let mut row = ClipRow::read(rows.clips[index]);
    match field {
        Field::Owner => row.owner = value,
        Field::Parent => row.parent = Some(value),
        Field::Shape => row.shape = value,
        _ => unreachable!(),
    }
    rows.clips[index] = row.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Clip, index as u32, field),
    );
}

fn expect_paint_reference(index: usize, field: Field, value: u32) {
    let (prepared, mut rows) = rich_case();
    let mut row = PaintRow::read(rows.paints[index]);
    match field {
        Field::Owner => row.owner = value,
        Field::Shape => row.reference = coverage(value, 0),
        Field::Brush => row.reference = coverage(0, value),
        Field::Image => row.reference = image(value),
        Field::Clip => row.clip = Some(value),
        _ => unreachable!(),
    }
    rows.paints[index] = row.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Paint, index as u32, field),
    );
}

fn expect_item_reference(table: Table, field: Field, value: u32) {
    let (prepared, mut rows) = rich_case();
    let mut row = if table == Table::Hit {
        ShapeItemRow::read_hit(rows.hits[0])
    } else {
        ShapeItemRow::read_semantic(rows.semantics[0])
    };
    match field {
        Field::Owner => row.owner = value,
        Field::Shape => row.shape = value,
        Field::Clip => row.clip = Some(value),
        _ => unreachable!(),
    }
    if table == Table::Hit {
        rows.hits[0] = row.build_hit();
    } else {
        rows.semantics[0] = row.build_semantic();
    }
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(table, 0, field),
    );
}

fn change_world(
    rows: &mut CandidateTables,
    table: Table,
    index: usize,
    component: usize,
    delta: i64,
) {
    match table {
        Table::Clip => {
            let mut r = ClipRow::read(rows.clips[index]);
            r.world[component] += delta;
            r.determinant = affine(r.world).determinant_raw();
            rows.clips[index] = r.build();
        }
        Table::Paint => {
            let mut r = PaintRow::read(rows.paints[index]);
            r.world[component] += delta;
            r.determinant = affine(r.world).determinant_raw();
            rows.paints[index] = r.build();
        }
        Table::Hit => {
            let mut r = ShapeItemRow::read_hit(rows.hits[index]);
            r.world[component] += delta;
            r.determinant = affine(r.world).determinant_raw();
            rows.hits[index] = r.build_hit();
        }
        Table::Semantic => {
            let mut r = ShapeItemRow::read_semantic(rows.semantics[index]);
            r.world[component] += delta;
            r.determinant = affine(r.world).determinant_raw();
            rows.semantics[index] = r.build_semantic();
        }
        Table::Geometry => unreachable!(),
    }
}

fn affine_index(field: Field) -> usize {
    match field {
        Field::AffineA => 0,
        Field::AffineB => 1,
        Field::AffineC => 2,
        Field::AffineD => 3,
        Field::AffineTx => 4,
        Field::AffineTy => 5,
        _ => unreachable!(),
    }
}

fn cross_axis_case() -> (PreparedSpatialV2, CandidateTables) {
    let source = cross_axis_empty_owned();
    let prepared =
        prepare_spatial_v2(&zero_call_engine(), source.clone(), requested_limits()).unwrap();
    let reference = materialize_reference_spatial_v2(
        prepare_spatial_v2(&zero_call_engine(), source, requested_limits()).unwrap(),
    );
    (prepared, CandidateTables::from_snapshot(&reference))
}
