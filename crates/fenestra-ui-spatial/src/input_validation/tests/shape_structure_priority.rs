use super::local_transform_support::{
    Placement, expect_transform, identity_values, input, node, node_field, root, set_field,
    transform,
};
use super::shape_structure_support::{
    circle, expect_content, expect_invalid_range, expect_reference, fixture, fixture_with_paths,
    limits, path_shape, point, polygon, rect, validate,
};
use super::validated_path_support::{line_to, move_to, path};
use crate::content_diagnostic::{SpatialContentReferenceV2, SpatialPathGrammarErrorV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialPathVerbFieldV2, SpatialShapeFieldV2};
use crate::model::SpatialScalarV2;
use crate::numeric_error::SpatialTransformErrorKindV2;
use crate::path::SpatialPathVerbV2;
use crate::vocabulary::SpatialTransformScalarFieldV2;

#[test]
fn owner_precedes_every_applicable_variant_on_the_same_record() {
    for shape in [
        rect(0, 0),
        circle(0, 0),
        polygon(0, 0, 1, u32::MAX),
        path_shape(0, 0, u32::MAX),
    ] {
        let fixture = fixture(vec![shape], Vec::new());

        expect_reference(
            validate(&fixture, limits()),
            SpatialContentReferenceV2::Owner,
            SpatialErrorLocationV2::Shape {
                index: 0,
                field: SpatialShapeFieldV2::Owner,
            },
        );
    }
}

#[test]
fn sentinel_missing_and_extreme_owners_use_the_current_shape_ordinal() {
    for owner in [0, 3, u32::MAX] {
        let fixture = fixture(vec![rect(0, 1), rect(1, owner)], Vec::new());

        expect_reference(
            validate(&fixture, limits()),
            SpatialContentReferenceV2::Owner,
            SpatialErrorLocationV2::Shape {
                index: 1,
                field: SpatialShapeFieldV2::Owner,
            },
        );
    }
}

#[test]
fn an_earlier_path_reference_beats_later_record_faults_and_trailing_points() {
    let fixture = fixture_with_paths(
        vec![path_shape(0, 1, 1), polygon(1, 1, 1, 0), rect(2, 0)],
        vec![point(0, 0)],
        vec![path(0, 0, 2)],
        vec![move_to(0, 0), line_to(1, 1)],
    );

    expect_reference(
        validate(&fixture, limits()),
        SpatialContentReferenceV2::Path,
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::Path,
        },
    );
}

#[test]
fn a_later_record_failure_precedes_leftover_polygon_payload() {
    let fixture = fixture(
        vec![polygon(0, 1, 0, 1), path_shape(1, 2, 0)],
        vec![point(0, 0), point(1, 1)],
    );

    expect_reference(
        validate(&fixture, limits()),
        SpatialContentReferenceV2::Path,
        SpatialErrorLocationV2::Shape {
            index: 1,
            field: SpatialShapeFieldV2::Path,
        },
    );
}

#[test]
fn missing_and_extreme_path_references_use_the_shape_field() {
    for missing in [1, u32::MAX] {
        let fixture = fixture_with_paths(
            vec![rect(0, 1), path_shape(1, 2, missing)],
            Vec::new(),
            vec![path(0, 0, 2)],
            vec![move_to(0, 0), line_to(1, 1)],
        );

        expect_reference(
            validate(&fixture, limits()),
            SpatialContentReferenceV2::Path,
            SpatialErrorLocationV2::Shape {
                index: 1,
                field: SpatialShapeFieldV2::Path,
            },
        );
    }
}

#[test]
fn an_earlier_polygon_end_beats_a_later_invalid_owner() {
    let fixture = fixture(
        vec![polygon(0, 1, 0, 2), polygon(1, 0, 1, u32::MAX)],
        vec![point(0, 0)],
    );

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
    );
}

#[test]
fn an_earlier_polygon_end_beats_a_later_missing_path() {
    let fixture = fixture_with_paths(
        vec![polygon(0, 1, 0, 2), path_shape(1, 2, 1)],
        vec![point(0, 0)],
        vec![path(0, 0, 2)],
        vec![move_to(0, 0), line_to(1, 1)],
    );

    expect_invalid_range(
        validate(&fixture, limits()),
        SpatialErrorLocationV2::Shape {
            index: 0,
            field: SpatialShapeFieldV2::PolygonPointLength,
        },
    );
}

#[test]
fn validated_path_k1_precedes_shape_structure() {
    let fixture = fixture_with_paths(
        vec![rect(u32::MAX, 0)],
        Vec::new(),
        vec![path(0, 0, 1)],
        vec![SpatialPathVerbV2::Close],
    );

    expect_content(
        validate(&fixture, limits()),
        SpatialContentErrorKindV2::InvalidPathGrammar(SpatialPathGrammarErrorV2::FirstNotMove),
        SpatialErrorLocationV2::PathVerb {
            path: 0,
            verb: 0,
            field: SpatialPathVerbFieldV2::Kind,
        },
    );
}

#[test]
fn local_transform_validation_precedes_shape_structure() {
    let field = SpatialTransformScalarFieldV2::AffineA;
    let mut values = identity_values();
    set_field(&mut values, field, SpatialScalarV2::MAX_RAW + 1);
    let fixture = input(vec![root(), node(Placement::Free, 1, 0, transform(values))])
        .with_paths(Vec::new(), Vec::new())
        .with_shapes(vec![rect(u32::MAX, 0)], Vec::new());

    expect_transform(
        validate(&fixture, limits()),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(field),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: node_field(field),
        },
    );
}
