use super::validated_path_support::{
    expect_content, expect_valid, fixture, limits, line_to, move_to, path, point, validate,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialPathVerbFieldV2;
use crate::model::SpatialScalarV2;
use crate::path::SpatialPathVerbV2;

#[test]
fn every_k1_path_scalar_maps_both_domain_sides_at_a_local_ordinal() {
    for raw in [SpatialScalarV2::MIN_RAW - 1, SpatialScalarV2::MAX_RAW + 1] {
        for (verb, field) in scalar_cases(raw) {
            let verbs = vec![move_to(0, 0), line_to(1, 1), move_to(2, 2), verb];
            let fixture = fixture(vec![path(0, 0, 2), path(1, 2, 2)], verbs);

            expect_content(
                validate(&fixture, limits(2, usize::MAX)),
                SpatialContentErrorKindV2::ScalarOutOfDomain,
                SpatialErrorLocationV2::PathVerb {
                    path: 1,
                    verb: 1,
                    field,
                },
            );
        }
    }
}

#[test]
fn every_scalar_accepts_both_inclusive_domain_edges() {
    let minimum = SpatialScalarV2::MIN_RAW;
    let maximum = SpatialScalarV2::MAX_RAW;
    let verbs = vec![
        move_to(minimum, maximum),
        line_to(maximum, minimum),
        SpatialPathVerbV2::QuadraticTo {
            control: point(minimum, maximum),
            to: point(maximum, minimum),
        },
        SpatialPathVerbV2::CubicTo {
            control1: point(minimum, maximum),
            control2: point(maximum, minimum),
            to: point(minimum, maximum),
        },
    ];
    let fixture = fixture(vec![path(0, 0, 4)], verbs);

    expect_valid(validate(&fixture, limits(4, 1)));
}

fn scalar_cases(raw: i64) -> Vec<(SpatialPathVerbV2, SpatialPathVerbFieldV2)> {
    use SpatialPathVerbFieldV2::{
        Control1X, Control1Y, Control2X, Control2Y, ControlX, ControlY, ToX, ToY,
    };

    vec![
        (move_to(raw, 0), ToX),
        (move_to(0, raw), ToY),
        (line_to(raw, 0), ToX),
        (line_to(0, raw), ToY),
        (
            SpatialPathVerbV2::QuadraticTo {
                control: point(raw, 0),
                to: point(0, 0),
            },
            ControlX,
        ),
        (
            SpatialPathVerbV2::QuadraticTo {
                control: point(0, raw),
                to: point(0, 0),
            },
            ControlY,
        ),
        (
            SpatialPathVerbV2::QuadraticTo {
                control: point(0, 0),
                to: point(raw, 0),
            },
            ToX,
        ),
        (
            SpatialPathVerbV2::QuadraticTo {
                control: point(0, 0),
                to: point(0, raw),
            },
            ToY,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(raw, 0),
                control2: point(0, 0),
                to: point(0, 0),
            },
            Control1X,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, raw),
                control2: point(0, 0),
                to: point(0, 0),
            },
            Control1Y,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, 0),
                control2: point(raw, 0),
                to: point(0, 0),
            },
            Control2X,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, 0),
                control2: point(0, raw),
                to: point(0, 0),
            },
            Control2Y,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, 0),
                control2: point(0, 0),
                to: point(raw, 0),
            },
            ToX,
        ),
        (
            SpatialPathVerbV2::CubicTo {
                control1: point(0, 0),
                control2: point(0, 0),
                to: point(0, raw),
            },
            ToY,
        ),
    ]
}
