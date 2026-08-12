use super::dependency_support::{
    VIEWPORT, dependency_limits, expect_dependency, fixture, free, layout, node_target, root,
};
use crate::error::{SpatialDependencyErrorKindV2, SpatialErrorLocationV2};
use crate::model::SpatialAnchorTargetV2;

#[test]
fn a_mutual_free_target_cycle_names_its_lowest_stable_ordinal() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, node_target(2)),
        free(2, 0, node_target(1)),
    ]);
    expect_cycle(&fixture, 1);
}

#[test]
fn a_free_host_and_its_island_form_a_real_two_unit_cycle() {
    let fixture = fixture(vec![root(), free(1, 0, node_target(2)), layout(2, 1)]);
    expect_cycle(&fixture, 1);
}

#[test]
fn an_earlier_blocked_dependent_is_not_misreported_as_the_cycle() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, node_target(3)),
        free(2, 0, SpatialAnchorTargetV2::Viewport),
        free(3, 0, node_target(4)),
        free(4, 0, node_target(3)),
    ]);
    expect_cycle(&fixture, 3);
}

#[test]
fn multiple_cyclic_components_choose_the_globally_lowest_component_minimum() {
    let fixture = fixture(vec![
        root(),
        free(1, 0, SpatialAnchorTargetV2::Viewport),
        free(2, 0, node_target(5)),
        free(3, 0, node_target(4)),
        free(4, 0, node_target(3)),
        free(5, 0, node_target(2)),
    ]);
    expect_cycle(&fixture, 2);
}

fn expect_cycle(fixture: &super::fixture::RawInputFixture, ordinal: u32) {
    expect_dependency(
        prepare_dependency_graph!(fixture, VIEWPORT, dependency_limits(usize::MAX, usize::MAX)),
        SpatialDependencyErrorKindV2::Cycle,
        SpatialErrorLocationV2::Dependency { ordinal },
    );
}
