use crate::*;

use super::round_trips::*;

#[test]
fn layout_record_getters_preserve_every_constructor_slot() {
    let point_value = point(11, 12, 100);
    assert_eq!(
        [point_value.x(), point_value.y()],
        [fixed(11, 100), fixed(12, 101)]
    );

    let padding = SpatialPaddingRecipeV2::new(
        integer(21, 110),
        integer(22, 111),
        integer(23, 112),
        integer(24, 113),
    );
    assert_eq!(
        [
            padding.left(),
            padding.right(),
            padding.top(),
            padding.bottom(),
        ],
        [
            integer(21, 110),
            integer(22, 111),
            integer(23, 112),
            integer(24, 113),
        ]
    );

    let dimension_value = dimension(31, 32, 33, 120);
    assert_eq!(
        [
            dimension_value.minimum(),
            dimension_value.preferred(),
            dimension_value.maximum(),
        ],
        [integer(31, 120), integer(32, 121), integer(33, 122)]
    );

    let transform_value = transform(40, 130);
    assert_eq!(
        [
            transform_value.a(),
            transform_value.b(),
            transform_value.c(),
            transform_value.d(),
            transform_value.tx(),
            transform_value.ty(),
        ],
        [
            fixed(40, 130),
            fixed(41, 131),
            fixed(42, 132),
            fixed(43, 133),
            fixed(44, 134),
            fixed(45, 135),
        ]
    );
    assert_eq!(transform_value.origin(), point(46, 47, 136));

    let viewport = viewport(140);
    assert_eq!(viewport.axis(), SpatialAxisV2::Column);
    assert_eq!(
        [
            viewport.left(),
            viewport.right(),
            viewport.top(),
            viewport.bottom(),
            viewport.gap(),
        ],
        [
            field(51, 140),
            field(52, 141),
            field(53, 142),
            field(54, 143),
            field(55, 144),
        ]
    );
    assert_eq!(viewport.span(), span(145));

    let container = container(150);
    assert_eq!(container.axis(), SpatialAxisV2::Row);
    assert_eq!(container.padding(), padding_from(61, 150));
    assert_eq!(container.gap(), integer(65, 154));

    let layout = layout(160);
    assert_eq!(layout.width(), dimension(71, 72, 73, 160));
    assert_eq!(layout.height(), dimension(74, 75, 76, 163));
    assert_eq!(layout.transform(), transform(77, 166));

    let free = free(180);
    assert_eq!(free.width(), integer(91, 180));
    assert_eq!(free.height(), integer(92, 181));
    assert_eq!(
        free.self_anchor(),
        [
            SpatialAnchorComponentV2::Start,
            SpatialAnchorComponentV2::Center,
        ]
    );
    assert_eq!(
        free.target(),
        SpatialAnchorTargetRecipeV2::Node(field(SpatialNodeSymbolV2::new(93), 182))
    );
    assert_eq!(
        free.target_anchor(),
        [
            SpatialAnchorComponentV2::End,
            SpatialAnchorComponentV2::Start,
        ]
    );
    assert_eq!(free.offset(), point(94, 95, 183));
    assert_eq!(free.transform(), transform(96, 185));
}
