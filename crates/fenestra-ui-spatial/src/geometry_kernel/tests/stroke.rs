use super::*;

#[test]
fn paint_and_hit_stroke_share_scalar_semantic_priority_and_typed_proof() {
    let sources = [
        GeometryK1StrokeSource::Paint { index: 13 },
        GeometryK1StrokeSource::Hit { index: 17 },
    ];

    for source in sources {
        let location = match source {
            GeometryK1StrokeSource::Paint { index } => GeometryK1Location::Paint {
                index,
                field: GeometryK1Field::StrokeWidth,
            },
            GeometryK1StrokeSource::Hit { index } => GeometryK1Location::Hit {
                index,
                field: GeometryK1Field::StrokeWidth,
            },
        };

        expect_error(
            validate_stroke_k1(source, scalar(SpatialScalarV2::MIN_RAW - 1)),
            GeometryK1ErrorKind::ScalarOutOfDomain,
            location,
        );
        expect_error(
            validate_stroke_k1(source, scalar(SpatialScalarV2::MAX_RAW + 1)),
            GeometryK1ErrorKind::ScalarOutOfDomain,
            location,
        );
        expect_error(
            validate_stroke_k1(source, scalar(-1)),
            GeometryK1ErrorKind::InvalidStroke(GeometryK1StrokeKind::NegativeWidth),
            location,
        );
        expect_error(
            validate_stroke_k1(source, scalar(0)),
            GeometryK1ErrorKind::InvalidStroke(GeometryK1StrokeKind::ZeroWidth),
            location,
        );

        let proof: ValidatedStrokeK1 = expect_valid(validate_stroke_k1(source, scalar(1)));
        assert_eq!(proof.width(), scalar(1));
    }
}

#[test]
fn stroke_accepts_the_positive_canonical_maximum() {
    let proof: ValidatedStrokeK1 = expect_valid(validate_stroke_k1(
        GeometryK1StrokeSource::Paint { index: 13 },
        scalar(SpatialScalarV2::MAX_RAW),
    ));
    assert_eq!(proof.width(), scalar(SpatialScalarV2::MAX_RAW));
}
