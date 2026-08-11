use crate::*;

#[test]
fn content_diagnostic_leaf_inventories_are_exact() {
    assert_eq!(
        SpatialKeyedContentTableV2::ALL,
        [
            SpatialKeyedContentTableV2::Path,
            SpatialKeyedContentTableV2::Shape,
            SpatialKeyedContentTableV2::Brush,
            SpatialKeyedContentTableV2::Image,
            SpatialKeyedContentTableV2::Clip,
        ]
    );
    assert_eq!(
        SpatialPayloadTableV2::ALL,
        [
            SpatialPayloadTableV2::PathVerb,
            SpatialPayloadTableV2::PolygonPoint,
            SpatialPayloadTableV2::GradientStop,
        ]
    );
    assert_eq!(
        SpatialContentReferenceV2::ALL,
        [
            SpatialContentReferenceV2::Path,
            SpatialContentReferenceV2::Shape,
            SpatialContentReferenceV2::Brush,
            SpatialContentReferenceV2::Image,
            SpatialContentReferenceV2::Clip,
            SpatialContentReferenceV2::Owner,
        ]
    );
    assert_eq!(
        SpatialOrderedItemTableV2::ALL,
        [
            SpatialOrderedItemTableV2::Paint,
            SpatialOrderedItemTableV2::Hit,
            SpatialOrderedItemTableV2::Semantic,
        ]
    );

    assert_eq!(
        SpatialPathGrammarErrorV2::ALL,
        [
            SpatialPathGrammarErrorV2::Empty,
            SpatialPathGrammarErrorV2::FirstNotMove,
            SpatialPathGrammarErrorV2::EmptySubpath,
            SpatialPathGrammarErrorV2::DrawingWithoutSubpath,
            SpatialPathGrammarErrorV2::CloseWithoutSegment,
            SpatialPathGrammarErrorV2::TrailingMove,
        ]
    );
    assert_eq!(
        SpatialShapeErrorV2::ALL,
        [
            SpatialShapeErrorV2::NegativeExtent,
            SpatialShapeErrorV2::NegativeRadius,
            SpatialShapeErrorV2::PolygonTooShort,
            SpatialShapeErrorV2::PolygonRepeatedFirst,
            SpatialShapeErrorV2::PolygonAdjacentEqual,
        ]
    );
    assert_eq!(
        SpatialStrokeErrorV2::ALL,
        [
            SpatialStrokeErrorV2::NegativeWidth,
            SpatialStrokeErrorV2::ZeroWidth,
        ]
    );
    assert_eq!(
        SpatialGradientErrorV2::ALL,
        [
            SpatialGradientErrorV2::CoincidentEndpoints,
            SpatialGradientErrorV2::TooFewStops,
            SpatialGradientErrorV2::FirstOffset,
            SpatialGradientErrorV2::LastOffset,
            SpatialGradientErrorV2::DecreasingOffset,
        ]
    );
    assert_eq!(
        SpatialImageErrorV2::ALL,
        [
            SpatialImageErrorV2::ZeroExtent,
            SpatialImageErrorV2::StrideMismatch,
            SpatialImageErrorV2::LengthMismatch,
            SpatialImageErrorV2::InvalidPremultipliedPixel,
            SpatialImageErrorV2::EmptySource,
            SpatialImageErrorV2::SourceOutOfBounds,
            SpatialImageErrorV2::NegativeDestinationExtent(SpatialExtentV2::Width),
            SpatialImageErrorV2::NegativeDestinationExtent(SpatialExtentV2::Height),
            SpatialImageErrorV2::EmptyDestination,
        ]
    );
    assert_eq!(
        SpatialClipErrorV2::ALL,
        [
            SpatialClipErrorV2::ForwardParent,
            SpatialClipErrorV2::ShapeOwnerMismatch,
            SpatialClipErrorV2::OwnerNotAncestor,
            SpatialClipErrorV2::ItemOwnerNotDescendant,
        ]
    );
}

#[test]
fn content_error_inventory_has_exact_type_composition_and_order() {
    let actual: [SpatialContentErrorKindV2; 52] = SpatialContentErrorKindV2::ALL;
    let expected = [
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Path),
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Shape),
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Brush),
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Image),
        SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Clip),
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::PathVerb),
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::PolygonPoint),
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::GradientStop),
        SpatialContentErrorKindV2::InvalidReference(SpatialContentReferenceV2::Path),
        SpatialContentErrorKindV2::InvalidReference(SpatialContentReferenceV2::Shape),
        SpatialContentErrorKindV2::InvalidReference(SpatialContentReferenceV2::Brush),
        SpatialContentErrorKindV2::InvalidReference(SpatialContentReferenceV2::Image),
        SpatialContentErrorKindV2::InvalidReference(SpatialContentReferenceV2::Clip),
        SpatialContentErrorKindV2::InvalidReference(SpatialContentReferenceV2::Owner),
        SpatialContentErrorKindV2::InvalidOrder(SpatialOrderedItemTableV2::Paint),
        SpatialContentErrorKindV2::InvalidOrder(SpatialOrderedItemTableV2::Hit),
        SpatialContentErrorKindV2::InvalidOrder(SpatialOrderedItemTableV2::Semantic),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialContentErrorKindV2::InvalidPathGrammar(SpatialPathGrammarErrorV2::Empty),
        SpatialContentErrorKindV2::InvalidPathGrammar(SpatialPathGrammarErrorV2::FirstNotMove),
        SpatialContentErrorKindV2::InvalidPathGrammar(SpatialPathGrammarErrorV2::EmptySubpath),
        SpatialContentErrorKindV2::InvalidPathGrammar(
            SpatialPathGrammarErrorV2::DrawingWithoutSubpath,
        ),
        SpatialContentErrorKindV2::InvalidPathGrammar(
            SpatialPathGrammarErrorV2::CloseWithoutSegment,
        ),
        SpatialContentErrorKindV2::InvalidPathGrammar(SpatialPathGrammarErrorV2::TrailingMove),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::NegativeExtent),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::NegativeRadius),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::PolygonTooShort),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::PolygonRepeatedFirst),
        SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::PolygonAdjacentEqual),
        SpatialContentErrorKindV2::InvalidStroke(SpatialStrokeErrorV2::NegativeWidth),
        SpatialContentErrorKindV2::InvalidStroke(SpatialStrokeErrorV2::ZeroWidth),
        SpatialContentErrorKindV2::InvalidGradient(SpatialGradientErrorV2::CoincidentEndpoints),
        SpatialContentErrorKindV2::InvalidGradient(SpatialGradientErrorV2::TooFewStops),
        SpatialContentErrorKindV2::InvalidGradient(SpatialGradientErrorV2::FirstOffset),
        SpatialContentErrorKindV2::InvalidGradient(SpatialGradientErrorV2::LastOffset),
        SpatialContentErrorKindV2::InvalidGradient(SpatialGradientErrorV2::DecreasingOffset),
        SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::ZeroExtent),
        SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::StrideMismatch),
        SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::LengthMismatch),
        SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::InvalidPremultipliedPixel),
        SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::EmptySource),
        SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::SourceOutOfBounds),
        SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::NegativeDestinationExtent(
            SpatialExtentV2::Width,
        )),
        SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::NegativeDestinationExtent(
            SpatialExtentV2::Height,
        )),
        SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::EmptyDestination),
        SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ForwardParent),
        SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ShapeOwnerMismatch),
        SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::OwnerNotAncestor),
        SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ItemOwnerNotDescendant),
        SpatialContentErrorKindV2::NonFlatAtMaximumDepth,
        SpatialContentErrorKindV2::LocalBoundsOutOfDomain(SpatialAxisV2::X),
        SpatialContentErrorKindV2::LocalBoundsOutOfDomain(SpatialAxisV2::Y),
    ];

    assert_eq!(actual, expected);
}

#[test]
fn image_and_content_errors_remain_exhaustively_matchable() {
    fn image_tag(value: SpatialImageErrorV2) -> u8 {
        match value {
            SpatialImageErrorV2::ZeroExtent => 0,
            SpatialImageErrorV2::StrideMismatch => 1,
            SpatialImageErrorV2::LengthMismatch => 2,
            SpatialImageErrorV2::InvalidPremultipliedPixel => 3,
            SpatialImageErrorV2::EmptySource => 4,
            SpatialImageErrorV2::SourceOutOfBounds => 5,
            SpatialImageErrorV2::NegativeDestinationExtent(_) => 6,
            SpatialImageErrorV2::EmptyDestination => 7,
        }
    }
    fn content_tag(value: SpatialContentErrorKindV2) -> u8 {
        match value {
            SpatialContentErrorKindV2::NonDenseKey(_) => 0,
            SpatialContentErrorKindV2::InvalidRange(_) => 1,
            SpatialContentErrorKindV2::InvalidReference(_) => 2,
            SpatialContentErrorKindV2::InvalidOrder(_) => 3,
            SpatialContentErrorKindV2::ScalarOutOfDomain => 4,
            SpatialContentErrorKindV2::InvalidPathGrammar(_) => 5,
            SpatialContentErrorKindV2::InvalidShape(_) => 6,
            SpatialContentErrorKindV2::InvalidStroke(_) => 7,
            SpatialContentErrorKindV2::InvalidGradient(_) => 8,
            SpatialContentErrorKindV2::InvalidImage(_) => 9,
            SpatialContentErrorKindV2::InvalidClip(_) => 10,
            SpatialContentErrorKindV2::NonFlatAtMaximumDepth => 11,
            SpatialContentErrorKindV2::LocalBoundsOutOfDomain(_) => 12,
        }
    }

    let _ = (
        image_tag as fn(SpatialImageErrorV2) -> u8,
        content_tag as fn(SpatialContentErrorKindV2) -> u8,
    );
}

#[test]
fn content_leaf_vocabularies_remain_exhaustively_matchable() {
    for value in SpatialKeyedContentTableV2::ALL {
        match value {
            SpatialKeyedContentTableV2::Path
            | SpatialKeyedContentTableV2::Shape
            | SpatialKeyedContentTableV2::Brush
            | SpatialKeyedContentTableV2::Image
            | SpatialKeyedContentTableV2::Clip => {}
        }
    }
    for value in SpatialPayloadTableV2::ALL {
        match value {
            SpatialPayloadTableV2::PathVerb
            | SpatialPayloadTableV2::PolygonPoint
            | SpatialPayloadTableV2::GradientStop => {}
        }
    }
    for value in SpatialContentReferenceV2::ALL {
        match value {
            SpatialContentReferenceV2::Path
            | SpatialContentReferenceV2::Shape
            | SpatialContentReferenceV2::Brush
            | SpatialContentReferenceV2::Image
            | SpatialContentReferenceV2::Clip
            | SpatialContentReferenceV2::Owner => {}
        }
    }
    for value in SpatialOrderedItemTableV2::ALL {
        match value {
            SpatialOrderedItemTableV2::Paint
            | SpatialOrderedItemTableV2::Hit
            | SpatialOrderedItemTableV2::Semantic => {}
        }
    }
    for value in SpatialPathGrammarErrorV2::ALL {
        match value {
            SpatialPathGrammarErrorV2::Empty
            | SpatialPathGrammarErrorV2::FirstNotMove
            | SpatialPathGrammarErrorV2::EmptySubpath
            | SpatialPathGrammarErrorV2::DrawingWithoutSubpath
            | SpatialPathGrammarErrorV2::CloseWithoutSegment
            | SpatialPathGrammarErrorV2::TrailingMove => {}
        }
    }
    for value in SpatialShapeErrorV2::ALL {
        match value {
            SpatialShapeErrorV2::NegativeExtent
            | SpatialShapeErrorV2::NegativeRadius
            | SpatialShapeErrorV2::PolygonTooShort
            | SpatialShapeErrorV2::PolygonRepeatedFirst
            | SpatialShapeErrorV2::PolygonAdjacentEqual => {}
        }
    }
    for value in SpatialStrokeErrorV2::ALL {
        match value {
            SpatialStrokeErrorV2::NegativeWidth | SpatialStrokeErrorV2::ZeroWidth => {}
        }
    }
    for value in SpatialGradientErrorV2::ALL {
        match value {
            SpatialGradientErrorV2::CoincidentEndpoints
            | SpatialGradientErrorV2::TooFewStops
            | SpatialGradientErrorV2::FirstOffset
            | SpatialGradientErrorV2::LastOffset
            | SpatialGradientErrorV2::DecreasingOffset => {}
        }
    }
    for value in SpatialClipErrorV2::ALL {
        match value {
            SpatialClipErrorV2::ForwardParent
            | SpatialClipErrorV2::ShapeOwnerMismatch
            | SpatialClipErrorV2::OwnerNotAncestor
            | SpatialClipErrorV2::ItemOwnerNotDescendant => {}
        }
    }
}
