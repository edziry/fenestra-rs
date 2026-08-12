use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutPaddingV1};
use fenestra_ui_spatial::prototype::{
    SpatialBrushContentV2, SpatialBrushKeyV2, SpatialBrushV2, SpatialClipKeyV2, SpatialClipV2,
    SpatialContainerV2, SpatialCoverageV2, SpatialFillRuleV2, SpatialGradientStopV2, SpatialHitV2,
    SpatialImageKeyV2, SpatialImageV2, SpatialInputPolicyV2, SpatialNodeKeyV2, SpatialNodeV2,
    SpatialPaintContentV2, SpatialPaintV2, SpatialPathKeyV2, SpatialPathV2, SpatialPathVerbV2,
    SpatialPlacementV2, SpatialPointV2, SpatialRgba8V2, SpatialScalarV2, SpatialSemanticGeometryV2,
    SpatialShapeGeometryV2, SpatialShapeKeyV2, SpatialShapeV2, SpatialViewportV2,
};

use crate::*;

#[derive(Clone, Copy)]
struct SliceIdentity<T> {
    pointer: *const T,
    length: usize,
}

struct FixtureIdentities {
    nodes: SliceIdentity<SpatialNodeV2>,
    polygon_points: SliceIdentity<SpatialPointV2>,
    path_verbs: SliceIdentity<SpatialPathVerbV2>,
    paths: SliceIdentity<SpatialPathV2>,
    shapes: SliceIdentity<SpatialShapeV2>,
    clips: SliceIdentity<SpatialClipV2>,
    gradient_stops: SliceIdentity<SpatialGradientStopV2>,
    brushes: SliceIdentity<SpatialBrushV2>,
    images: SliceIdentity<SpatialImageV2>,
    image_bytes: SliceIdentity<u8>,
    paint_items: SliceIdentity<SpatialPaintV2>,
    hit_items: SliceIdentity<SpatialHitV2>,
    semantic_items: SliceIdentity<SpatialSemanticGeometryV2>,
}

#[test]
fn owner_survives_fixture_scope_and_preserves_every_box_identity() {
    let (owner, identities) = owned_fixture();
    let input = owner.as_input();

    assert_eq!(
        input.topology().viewport(),
        SpatialViewportV2::new(-101, 202)
    );
    assert_identity(input.topology().nodes(), identities.nodes);
    assert_identity(input.geometry().polygon_points(), identities.polygon_points);
    assert_identity(input.geometry().path_verbs(), identities.path_verbs);
    assert_identity(input.geometry().paths(), identities.paths);
    assert_identity(input.geometry().shapes(), identities.shapes);
    assert_identity(input.geometry().clips(), identities.clips);
    assert_identity(
        input.resources().gradient_stops(),
        identities.gradient_stops,
    );
    assert_identity(input.resources().brushes(), identities.brushes);
    assert_identity(input.resources().images(), identities.images);
    assert_identity(
        input.resources().images()[0].bytes(),
        identities.image_bytes,
    );
    assert_identity(input.items().paint_items(), identities.paint_items);
    assert_identity(input.items().hit_items(), identities.hit_items);
    assert_identity(input.items().semantic_items(), identities.semantic_items);

    assert_eq!(input.topology().nodes()[0].key().get(), 0);
    assert_eq!(input.geometry().polygon_points()[0].x().raw(), -1);
    assert_eq!(input.resources().images()[0].bytes(), &[41, 42, 43, 44]);
    assert_eq!(input.items().semantic_items()[0].item_ordinal(), 12);
}

fn owned_fixture() -> (SpatialOwnedInputV2, FixtureIdentities) {
    let nodes = repeat_box(
        SpatialNodeV2::new(
            SpatialNodeKeyV2::new(0),
            None,
            SpatialPlacementV2::Root,
            SpatialContainerV2::new(LayoutAxisV1::Row, LayoutPaddingV1::new(1, 2, 3, 4), 5),
        ),
        1,
    );
    let polygon_points = repeat_box(point(-1, 2), 2);
    let path_verbs = repeat_box(SpatialPathVerbV2::MoveTo { to: point(3, 4) }, 3);
    let paths = repeat_box(SpatialPathV2::new(SpatialPathKeyV2::new(5), 6, 7), 4);
    let shapes = repeat_box(
        SpatialShapeV2::new(
            SpatialShapeKeyV2::new(8),
            SpatialNodeKeyV2::new(9),
            SpatialShapeGeometryV2::Rect {
                origin: point(10, 11),
                width: scalar(12),
                height: scalar(13),
            },
        ),
        5,
    );
    let clips = repeat_box(
        SpatialClipV2::new(
            SpatialClipKeyV2::new(14),
            SpatialNodeKeyV2::new(15),
            Some(SpatialClipKeyV2::new(16)),
            SpatialShapeKeyV2::new(17),
            SpatialFillRuleV2::EvenOdd,
        ),
        6,
    );
    let gradient_stops = repeat_box(
        SpatialGradientStopV2::new(18, SpatialRgba8V2::new(19, 20, 21, 22)),
        7,
    );
    let brushes = repeat_box(
        SpatialBrushV2::new(
            SpatialBrushKeyV2::new(23),
            SpatialBrushContentV2::Solid {
                color: SpatialRgba8V2::new(24, 25, 26, 27),
            },
        ),
        8,
    );
    let images = (0..9)
        .map(|ordinal| {
            SpatialImageV2::new(
                SpatialImageKeyV2::new(ordinal),
                1,
                1,
                4,
                vec![41, 42, 43, 44].into_boxed_slice(),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let paint_items = repeat_box(
        SpatialPaintV2::new(
            SpatialNodeKeyV2::new(28),
            10,
            SpatialPaintContentV2::CoveragePaint {
                coverage: SpatialCoverageV2::Fill {
                    shape: SpatialShapeKeyV2::new(29),
                    rule: SpatialFillRuleV2::NonZero,
                },
                brush: SpatialBrushKeyV2::new(30),
                opacity: 31,
                clip: None,
            },
        ),
        10,
    );
    let hit_items = repeat_box(
        SpatialHitV2::new(
            SpatialNodeKeyV2::new(32),
            11,
            SpatialCoverageV2::Fill {
                shape: SpatialShapeKeyV2::new(33),
                rule: SpatialFillRuleV2::EvenOdd,
            },
            Some(SpatialClipKeyV2::new(34)),
            SpatialInputPolicyV2::Ignore,
        ),
        11,
    );
    let semantic_items = repeat_box(
        SpatialSemanticGeometryV2::new(
            SpatialNodeKeyV2::new(35),
            12,
            SpatialShapeKeyV2::new(36),
            SpatialFillRuleV2::NonZero,
            Some(SpatialClipKeyV2::new(37)),
        ),
        12,
    );

    let identities = FixtureIdentities {
        nodes: identity(&nodes),
        polygon_points: identity(&polygon_points),
        path_verbs: identity(&path_verbs),
        paths: identity(&paths),
        shapes: identity(&shapes),
        clips: identity(&clips),
        gradient_stops: identity(&gradient_stops),
        brushes: identity(&brushes),
        images: identity(&images),
        image_bytes: identity(images[0].bytes()),
        paint_items: identity(&paint_items),
        hit_items: identity(&hit_items),
        semantic_items: identity(&semantic_items),
    };
    let owner = SpatialOwnedInputV2::new(
        SpatialViewportV2::new(-101, 202),
        nodes,
        polygon_points,
        path_verbs,
        paths,
        shapes,
        clips,
        gradient_stops,
        brushes,
        images,
        paint_items,
        hit_items,
        semantic_items,
    );
    (owner, identities)
}

fn repeat_box<T: Clone>(value: T, length: usize) -> Box<[T]> {
    vec![value; length].into_boxed_slice()
}

fn identity<T>(slice: &[T]) -> SliceIdentity<T> {
    SliceIdentity {
        pointer: slice.as_ptr(),
        length: slice.len(),
    }
}

fn assert_identity<T>(actual: &[T], expected: SliceIdentity<T>) {
    assert_eq!(actual.as_ptr(), expected.pointer);
    assert_eq!(actual.len(), expected.length);
}

fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}
