use fenestra_ui_ir::prototype::{
    SpatialAnchorComponentV2, SpatialAnchorTargetRecipeV2, SpatialAxisV2,
    SpatialFreePlacementRecipeV2, SpatialNodeDeclarationV2, SpatialNodeParentV2,
    SpatialPlacementRecipeV2,
};

use super::super::value::{fixed_lit, fixed_prop, i32_lit, i32_prop, node, point, span, template};
use super::layout::{container, identity, layout_placement, transform, zero_edges};

pub(super) fn stack() -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        node(1, 201),
        template(1, 203),
        SpatialNodeParentV2::Node(node(0, 202)),
        SpatialPlacementRecipeV2::Layout(layout_placement(
            [i32_lit(0, 211), i32_prop(0, 212), i32_lit(120, 213)],
            [i32_lit(0, 214), i32_prop(1, 215), i32_lit(90, 216)],
            transform(
                [
                    fixed_lit(65_536, 218),
                    fixed_lit(0, 219),
                    fixed_lit(0, 220),
                    fixed_lit(65_536, 221),
                    fixed_prop(3, 222),
                    fixed_lit(32_768, 223),
                ],
                point(fixed_lit(0, 224), fixed_lit(0, 225)),
            ),
        )),
        container(
            SpatialAxisV2::Row,
            [
                i32_lit(1, 205),
                i32_prop(2, 206),
                i32_lit(1, 207),
                i32_prop(2, 208),
            ],
            i32_lit(1, 209),
        ),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        span(200),
    )
}

pub(super) fn floating() -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        node(2, 227),
        template(2, 229),
        SpatialNodeParentV2::Node(node(1, 228)),
        SpatialPlacementRecipeV2::Free(SpatialFreePlacementRecipeV2::new(
            i32_prop(0, 237),
            i32_prop(1, 238),
            [
                SpatialAnchorComponentV2::Center,
                SpatialAnchorComponentV2::End,
            ],
            SpatialAnchorTargetRecipeV2::Node(node(5, 239)),
            [
                SpatialAnchorComponentV2::Start,
                SpatialAnchorComponentV2::Center,
            ],
            point(fixed_lit(-65_536, 240), fixed_prop(3, 241)),
            transform(
                [
                    fixed_prop(3, 247),
                    fixed_lit(0, 243),
                    fixed_lit(0, 244),
                    fixed_lit(65_536, 248),
                    fixed_lit(0, 245),
                    fixed_lit(0, 246),
                ],
                point(fixed_lit(131_072, 249), fixed_lit(131_072, 250)),
            ),
        )),
        container(
            SpatialAxisV2::Column,
            [
                i32_prop(2, 231),
                i32_lit(0, 232),
                i32_prop(2, 233),
                i32_lit(0, 234),
            ],
            i32_prop(2, 235),
        ),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        span(226),
    )
}

pub(super) fn floating_child() -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        node(3, 252),
        template(3, 254),
        SpatialNodeParentV2::Node(node(2, 253)),
        SpatialPlacementRecipeV2::Free(SpatialFreePlacementRecipeV2::new(
            i32_prop(0, 262),
            i32_prop(1, 263),
            [
                SpatialAnchorComponentV2::Start,
                SpatialAnchorComponentV2::Start,
            ],
            SpatialAnchorTargetRecipeV2::Parent,
            [SpatialAnchorComponentV2::End, SpatialAnchorComponentV2::End],
            point(fixed_lit(65_536, 264), fixed_lit(-65_536, 265)),
            transform(
                [
                    fixed_lit(0, 267),
                    fixed_lit(65_536, 268),
                    fixed_lit(-65_536, 269),
                    fixed_lit(0, 270),
                    fixed_lit(0, 271),
                    fixed_lit(0, 272),
                ],
                point(fixed_lit(393_216, 273), fixed_lit(327_680, 274)),
            ),
        )),
        container(SpatialAxisV2::Row, zero_edges(256), i32_lit(0, 260)),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        span(251),
    )
}

pub(super) fn guide() -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        node(5, 331),
        template(5, 333),
        SpatialNodeParentV2::Node(node(0, 332)),
        SpatialPlacementRecipeV2::Layout(layout_placement(
            [i32_lit(0, 341), i32_prop(0, 342), i32_lit(100, 343)],
            [i32_lit(0, 344), i32_prop(1, 345), i32_lit(80, 346)],
            identity(348),
        )),
        container(
            SpatialAxisV2::Row,
            [
                i32_lit(2, 335),
                i32_lit(2, 336),
                i32_lit(2, 337),
                i32_lit(2, 338),
            ],
            i32_prop(2, 339),
        ),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        span(330),
    )
}

pub(super) fn viewport_layer() -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        node(6, 357),
        template(6, 359),
        SpatialNodeParentV2::Node(node(5, 358)),
        SpatialPlacementRecipeV2::Free(SpatialFreePlacementRecipeV2::new(
            i32_prop(0, 367),
            i32_prop(1, 368),
            [
                SpatialAnchorComponentV2::End,
                SpatialAnchorComponentV2::Center,
            ],
            SpatialAnchorTargetRecipeV2::Viewport,
            [
                SpatialAnchorComponentV2::Start,
                SpatialAnchorComponentV2::Start,
            ],
            point(fixed_lit(131_072, 369), fixed_lit(196_608, 370)),
            transform(
                [
                    fixed_lit(65_536, 372),
                    fixed_lit(0, 373),
                    fixed_lit(0, 374),
                    fixed_lit(65_536, 375),
                    fixed_lit(32_768, 376),
                    fixed_lit(-32_768, 377),
                ],
                point(fixed_lit(0, 378), fixed_lit(0, 379)),
            ),
        )),
        container(SpatialAxisV2::Column, zero_edges(361), i32_lit(0, 365)),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        span(356),
    )
}
