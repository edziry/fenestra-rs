use fenestra_ui_layout::prototype::{
    LayoutConstraintFieldV1, LayoutEngineErrorKindV1, LayoutErrorKindV1, LayoutErrorLocationV1,
    LayoutExtentV1, LayoutInputErrorKindV1, LayoutLimitKindV1, LayoutOutputErrorKindV1,
    LayoutPaddingSideV1,
};

use super::island_support::expect_plan;
use super::layout_preflight_support::{
    VIEWPORT, fixed, free, input, invalid_container, layout, limits, negative_constraint, root,
    valid_container,
};
use crate::error::{SpatialContainerErrorKindV2, SpatialErrorLocationV2};
use crate::vocabulary::SpatialNodeFieldV2;

#[test]
fn supported_bridge_cases_derive_host_and_member_owners_from_the_plan() {
    let fixture = one_island();
    let plan = expect_plan(prepare_island_plan!(&fixture, VIEWPORT, limits(1, 2, 2)));

    super::layout_preflight_support::expect_input::<()>(
        Err(map_layout_preflight_error!(
            plan,
            0,
            LayoutErrorKindV1::Input(LayoutInputErrorKindV1::NegativeConstraint {
                extent: LayoutExtentV1::Width,
                field: LayoutConstraintFieldV1::Preferred,
            }),
            LayoutErrorLocationV1::InputNode { index: 1 }
        )),
        negative_constraint(LayoutExtentV1::Width, LayoutConstraintFieldV1::Preferred),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::LayoutWidthPreferred,
        },
    );
    super::layout_preflight_support::expect_input::<()>(
        Err(map_layout_preflight_error!(
            plan,
            0,
            LayoutErrorKindV1::Input(LayoutInputErrorKindV1::NegativePadding(
                LayoutPaddingSideV1::Left,
            )),
            LayoutErrorLocationV1::InputNode { index: 0 }
        )),
        invalid_container(SpatialContainerErrorKindV2::NegativePadding(
            LayoutPaddingSideV1::Left,
        )),
        SpatialErrorLocationV2::NodeField {
            index: 0,
            field: SpatialNodeFieldV2::PaddingLeft,
        },
    );
}

#[test]
fn every_nonauthored_layout_input_kind_becomes_an_island_bridge_invariant() {
    let fixture = one_island();
    let plan = expect_plan(prepare_island_plan!(&fixture, VIEWPORT, limits(1, 2, 2)));
    let cases = [
        (
            LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::Nodes),
            LayoutErrorLocationV1::Input,
        ),
        (
            LayoutInputErrorKindV1::EmptyInput,
            LayoutErrorLocationV1::Input,
        ),
        (
            LayoutInputErrorKindV1::InvalidRootKey,
            LayoutErrorLocationV1::InputNode { index: 0 },
        ),
        (
            LayoutInputErrorKindV1::RootHasParent,
            LayoutErrorLocationV1::InputNode { index: 0 },
        ),
        (
            LayoutInputErrorKindV1::NonDenseKey,
            LayoutErrorLocationV1::InputNode { index: 1 },
        ),
        (
            LayoutInputErrorKindV1::MissingParent,
            LayoutErrorLocationV1::InputNode { index: 1 },
        ),
        (
            LayoutInputErrorKindV1::ForwardParent,
            LayoutErrorLocationV1::InputNode { index: 1 },
        ),
        (
            LayoutInputErrorKindV1::InvalidPreorder,
            LayoutErrorLocationV1::InputNode { index: 1 },
        ),
        (
            LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::Depth),
            LayoutErrorLocationV1::InputNode { index: 1 },
        ),
        (
            LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::ChildrenPerNode),
            LayoutErrorLocationV1::InputNode { index: 0 },
        ),
        (
            LayoutInputErrorKindV1::NegativeViewport(LayoutExtentV1::Width),
            LayoutErrorLocationV1::Viewport,
        ),
        (
            LayoutInputErrorKindV1::NegativeViewport(LayoutExtentV1::Height),
            LayoutErrorLocationV1::Viewport,
        ),
    ];

    for (kind, location) in cases {
        expect_bridge(
            map_layout_preflight_error!(plan, 0, LayoutErrorKindV1::Input(kind), location),
            SpatialErrorLocationV2::Island { index: 0 },
        );
    }
}

#[test]
fn engine_and_output_categories_cannot_cross_the_preflight_bridge() {
    let fixture = one_island();
    let plan = expect_plan(prepare_island_plan!(&fixture, VIEWPORT, limits(1, 2, 2)));

    for kind in LayoutEngineErrorKindV1::ALL {
        expect_bridge(
            map_layout_preflight_error!(
                plan,
                0,
                LayoutErrorKindV1::Engine(kind),
                LayoutErrorLocationV1::InputNode { index: 1 }
            ),
            SpatialErrorLocationV2::Island { index: 0 },
        );
    }
    for kind in LayoutOutputErrorKindV1::ALL {
        expect_bridge(
            map_layout_preflight_error!(
                plan,
                0,
                LayoutErrorKindV1::Output(kind),
                LayoutErrorLocationV1::OutputRecord { index: 1 }
            ),
            SpatialErrorLocationV2::Island { index: 0 },
        );
    }
}

#[test]
fn every_authored_kind_rejects_wrong_or_out_of_range_locations() {
    let fixture = one_island();
    let plan = expect_plan(prepare_island_plan!(&fixture, VIEWPORT, limits(1, 2, 2)));
    let kinds = authored_input_kinds();
    assert_eq!(kinds.len(), 15);

    for kind in kinds {
        for location in [
            LayoutErrorLocationV1::Input,
            LayoutErrorLocationV1::Viewport,
            LayoutErrorLocationV1::Output,
            LayoutErrorLocationV1::OutputRecord { index: 0 },
            LayoutErrorLocationV1::InputNode { index: 2 },
        ] {
            expect_bridge(
                map_layout_preflight_error!(plan, 0, LayoutErrorKindV1::Input(kind), location),
                SpatialErrorLocationV2::Island { index: 0 },
            );
        }
    }
}

#[test]
fn every_dimension_kind_rejects_the_synthetic_host_record() {
    let fixture = one_island();
    let plan = expect_plan(prepare_island_plan!(&fixture, VIEWPORT, limits(1, 2, 2)));
    let kinds = dimension_input_kinds();
    assert_eq!(kinds.len(), 8);

    for kind in kinds {
        expect_bridge(
            map_layout_preflight_error!(
                plan,
                0,
                LayoutErrorKindV1::Input(kind),
                LayoutErrorLocationV1::InputNode { index: 0 }
            ),
            SpatialErrorLocationV2::Island { index: 0 },
        );
    }
}

#[test]
fn every_dimension_kind_rejects_a_singleton_synthetic_record() {
    let fixture = input(vec![root(valid_container())]);
    let plan = expect_plan(prepare_island_plan!(&fixture, VIEWPORT, limits(0, 0, 0)));
    let kinds = dimension_input_kinds();
    assert_eq!(kinds.len(), 8);

    for kind in kinds {
        expect_bridge(
            map_layout_preflight_error!(
                plan,
                0,
                LayoutErrorKindV1::Input(kind),
                LayoutErrorLocationV1::InputNode { index: 0 }
            ),
            SpatialErrorLocationV2::Node { index: 0 },
        );
    }
}

#[test]
fn fallback_uses_the_selected_items_dense_island_or_singleton_owner() {
    let valid = valid_container();
    let island_fixture = input(vec![
        root(valid),
        free(1, 0, 10, 10, valid),
        free(2, 0, 10, 10, valid),
        layout(3, 2, fixed(10), fixed(10), valid),
        free(4, 3, 10, 10, valid),
        layout(5, 4, fixed(10), fixed(10), valid),
        layout(6, 3, fixed(10), fixed(10), valid),
    ]);
    let island_plan = expect_plan(prepare_island_plan!(
        &island_fixture,
        VIEWPORT,
        limits(2, 3, 5)
    ));

    expect_bridge(
        map_layout_preflight_error!(
            island_plan,
            3,
            LayoutErrorKindV1::Input(LayoutInputErrorKindV1::EmptyInput),
            LayoutErrorLocationV1::Input
        ),
        SpatialErrorLocationV2::Island { index: 1 },
    );

    let singleton_fixture = input(vec![
        root(valid),
        layout(1, 0, fixed(10), fixed(10), valid),
        free(2, 0, 10, 10, valid),
    ]);
    let singleton_plan = expect_plan(prepare_island_plan!(
        &singleton_fixture,
        VIEWPORT,
        limits(1, 2, 2)
    ));

    expect_bridge(
        map_layout_preflight_error!(
            singleton_plan,
            1,
            LayoutErrorKindV1::Input(LayoutInputErrorKindV1::NegativePadding(
                LayoutPaddingSideV1::Left,
            )),
            LayoutErrorLocationV1::Input
        ),
        SpatialErrorLocationV2::Node { index: 2 },
    );
}

fn dimension_input_kinds() -> Vec<LayoutInputErrorKindV1> {
    let mut kinds = Vec::new();
    for extent in LayoutExtentV1::ALL {
        for field in LayoutConstraintFieldV1::ALL {
            kinds.push(LayoutInputErrorKindV1::NegativeConstraint { extent, field });
        }
    }
    for extent in LayoutExtentV1::ALL {
        kinds.push(LayoutInputErrorKindV1::InvertedConstraint(extent));
    }
    kinds
}

fn authored_input_kinds() -> Vec<LayoutInputErrorKindV1> {
    let mut kinds = dimension_input_kinds();
    for side in LayoutPaddingSideV1::ALL {
        kinds.push(LayoutInputErrorKindV1::NegativePadding(side));
    }
    for extent in LayoutExtentV1::ALL {
        kinds.push(LayoutInputErrorKindV1::PaddingExceedsExtent(extent));
    }
    kinds.push(LayoutInputErrorKindV1::NegativeGap);
    kinds
}

fn one_island() -> super::fixture::RawInputFixture {
    input(vec![
        root(valid_container()),
        layout(1, 0, fixed(10), fixed(10), valid_container()),
    ])
}

fn expect_bridge(
    error: crate::resolve_error::SpatialResolveErrorV2,
    location: SpatialErrorLocationV2,
) {
    super::layout_preflight_support::expect_bridge(error, location);
}
