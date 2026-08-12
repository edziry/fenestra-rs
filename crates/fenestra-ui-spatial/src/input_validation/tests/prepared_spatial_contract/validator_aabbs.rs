use super::support::{cross_axis_empty_owned, requested_limits, zero_call_engine};
use super::validator_support::*;
use super::*;
use crate::model::SpatialScalarV2;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;

const AABB_FIELDS: [Field; 5] = [
    Field::AabbEmpty,
    Field::AabbMinX,
    Field::AabbMinY,
    Field::AabbMaxX,
    Field::AabbMaxY,
];

#[test]
fn every_table_compares_canonical_derived_aabbs_in_exact_field_order() {
    for table in Table::ALL {
        for field in AABB_FIELDS {
            let (prepared, mut rows) = rich_case();
            mutate_aabb(&mut rows, table, 0, field);
            expect_output_error(
                validate(prepared, &rows),
                Kind::InvalidAabb,
                output_location(table, 0, field),
            );
        }
    }

    for pair in AABB_FIELDS.windows(2) {
        let (prepared, mut rows) = rich_case();
        mutate_aabb(&mut rows, Table::Paint, 0, pair[0]);
        mutate_aabb(&mut rows, Table::Paint, 0, pair[1]);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidAabb,
            output_location(Table::Paint, 0, pair[0]),
        );
    }

    let (prepared, mut rows) = rich_case();
    let mut geometry = GeometryRow::read(rows.geometry[0]);
    geometry.aabb = (false, [1, 1, 0, 0]);
    rows.geometry[0] = geometry.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidAabb,
        output_location(Table::Geometry, 0, Field::AabbMinX),
    );
}

#[test]
fn canonical_empty_and_closed_point_or_line_are_distinct_valid_values() {
    let (prepared, mut rows) = rich_case();
    let mut geometry = GeometryRow::read(rows.geometry[0]);
    geometry.width = 0;
    geometry.height = 0;
    geometry.aabb = (false, [0, 0, 0, 0]);
    rows.geometry[0] = geometry.build();
    validate(prepared, &rows).expect("closed geometry point is valid");

    let (prepared, mut rows) = rich_case();
    let mut geometry = GeometryRow::read(rows.geometry[0]);
    geometry.width = 0;
    geometry.height = S;
    geometry.aabb = (false, [0, 0, 0, S]);
    rows.geometry[0] = geometry.build();
    validate(prepared, &rows).expect("closed geometry line is valid");

    for field in [
        Field::AabbMinX,
        Field::AabbMinY,
        Field::AabbMaxX,
        Field::AabbMaxY,
    ] {
        let source = cross_axis_empty_owned();
        let prepared =
            prepare_spatial_v2(&zero_call_engine(), source.clone(), requested_limits()).unwrap();
        let reference = materialize_reference_spatial_v2(
            prepare_spatial_v2(&zero_call_engine(), source, requested_limits()).unwrap(),
        );
        let mut rows = CandidateTables::from_snapshot(&reference);
        mutate_aabb(&mut rows, Table::Hit, 0, field);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidAabb,
            output_location(Table::Hit, 0, field),
        );
    }
}

#[test]
fn aabb_pass_is_table_then_record_then_edge_and_precedes_clip_chains() {
    let (prepared, mut rows) = rich_case();
    mutate_aabb(&mut rows, Table::Geometry, 1, Field::AabbMaxY);
    mutate_aabb(&mut rows, Table::Clip, 0, Field::AabbMinX);
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidAabb,
        output_location(Table::Geometry, 1, Field::AabbMaxY),
    );

    let (prepared, mut rows) = rich_case();
    mutate_aabb(&mut rows, Table::Semantic, 1, Field::AabbMaxY);
    let mut clip = ClipRow::read(rows.clips[0]);
    clip.parent = Some(0);
    rows.clips[0] = clip.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidAabb,
        output_location(Table::Semantic, 1, Field::AabbMaxY),
    );

    for (pair, earlier_last) in Table::ALL.windows(2).zip([3, 2, 2, 2]) {
        let (prepared, mut rows) = rich_case();
        mutate_aabb(&mut rows, pair[0], earlier_last, Field::AabbMaxY);
        mutate_aabb(&mut rows, pair[1], 0, Field::AabbMinX);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidAabb,
            output_location(pair[0], earlier_last as u32, Field::AabbMaxY),
        );
    }
}

#[test]
fn geometry_derivation_overflow_maps_each_edge_to_output_invalid_aabb() {
    let cases = [
        (
            [-SpatialScalarV2::MAX_RAW, 0, 0, S, 0, 0],
            2 * S,
            S,
            Field::AabbMinX,
        ),
        (
            [S, 0, 0, -SpatialScalarV2::MAX_RAW, 0, 0],
            S,
            2 * S,
            Field::AabbMinY,
        ),
        (
            [SpatialScalarV2::MAX_RAW, 0, 0, S, 0, 0],
            2 * S,
            S,
            Field::AabbMaxX,
        ),
        (
            [S, 0, 0, SpatialScalarV2::MAX_RAW, 0, 0],
            S,
            2 * S,
            Field::AabbMaxY,
        ),
    ];
    for (world, width, height, field) in cases {
        let (prepared, mut rows) = rich_case();
        let mut geometry = GeometryRow::read(rows.geometry[0]);
        geometry.width = width;
        geometry.height = height;
        geometry.world = world;
        geometry.determinant = affine(world).determinant_raw();
        geometry.aabb = (false, [0, 0, 0, 0]);
        rows.geometry[0] = geometry.build();
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidAabb,
            output_location(Table::Geometry, 0, field),
        );
    }
}

#[test]
fn retained_content_bounds_derivation_overflow_maps_to_output_invalid_aabb() {
    let (prepared, mut rows) = rich_case();
    let mut clip = ClipRow::read(rows.clips[0]);
    clip.world = [SpatialScalarV2::MAX_RAW, 0, 0, S, 0, 0];
    clip.determinant = affine(clip.world).determinant_raw();
    clip.aabb = (false, [0, 0, 0, 0]);
    rows.clips[0] = clip.build();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidAabb,
        output_location(Table::Clip, 0, Field::AabbMaxX),
    );
}

#[test]
fn trusted_source_row_not_candidate_references_selects_local_bounds() {
    let (prepared, mut rows) = rich_case();
    let mut hit = ShapeItemRow::read_hit(rows.hits[0]);
    hit.shape = 2;
    rows.hits[0] = hit.build_hit();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Hit, 0, Field::Shape),
    );

    let (prepared, mut rows) = rich_case();
    let mut hit = ShapeItemRow::read_hit(rows.hits[0]);
    hit.shape = 2;
    hit.aabb = (false, [S, 12 * S, 3 * S, 12 * S]);
    rows.hits[0] = hit.build_hit();
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidAabb,
        output_location(Table::Hit, 0, Field::AabbMinX),
    );
}

fn mutate_aabb(rows: &mut CandidateTables, table: Table, index: usize, field: Field) {
    let mutate = |aabb: &mut (bool, [i64; 4])| match field {
        Field::AabbEmpty => aabb.0 = !aabb.0,
        Field::AabbMinX => aabb.1[0] += 1,
        Field::AabbMinY => aabb.1[1] += 1,
        Field::AabbMaxX => aabb.1[2] -= 1,
        Field::AabbMaxY => aabb.1[3] -= 1,
        _ => unreachable!(),
    };
    match table {
        Table::Geometry => {
            let mut r = GeometryRow::read(rows.geometry[index]);
            mutate(&mut r.aabb);
            rows.geometry[index] = r.build();
        }
        Table::Clip => {
            let mut r = ClipRow::read(rows.clips[index]);
            mutate(&mut r.aabb);
            rows.clips[index] = r.build();
        }
        Table::Paint => {
            let mut r = PaintRow::read(rows.paints[index]);
            mutate(&mut r.aabb);
            rows.paints[index] = r.build();
        }
        Table::Hit => {
            let mut r = ShapeItemRow::read_hit(rows.hits[index]);
            mutate(&mut r.aabb);
            rows.hits[index] = r.build_hit();
        }
        Table::Semantic => {
            let mut r = ShapeItemRow::read_semantic(rows.semantics[index]);
            mutate(&mut r.aabb);
            rows.semantics[index] = r.build_semantic();
        }
    }
}
