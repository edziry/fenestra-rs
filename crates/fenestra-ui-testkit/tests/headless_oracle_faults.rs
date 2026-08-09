#[path = "headless/fixture_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::{
    HeadlessFixtureV1, HeadlessMismatchFieldV1, HeadlessMismatchKindV1, HeadlessMismatchLocationV1,
    HeadlessOracleV1, HeadlessProjectionFaultV1, NodePathV1, NormalizedHeadlessProjectionV1,
    compare_headless_projection_v1, inject_headless_projection_fault_v1,
};

use support::{control_path, root_path};

fn mismatch_path(location: &HeadlessMismatchLocationV1) -> Option<&NodePathV1> {
    match location {
        HeadlessMismatchLocationV1::Path(path) => Some(path),
        HeadlessMismatchLocationV1::End => None,
    }
}

fn initial_projection() -> NormalizedHeadlessProjectionV1 {
    let fixture = HeadlessFixtureV1::build().expect("registered headless fixture should validate");
    HeadlessOracleV1::new(&fixture)
        .expect("initial desired state should rebuild")
        .rebuild()
        .expect("clean rebuild should succeed")
}

fn changed_families(
    expected: &NormalizedHeadlessProjectionV1,
    observed: &NormalizedHeadlessProjectionV1,
) -> [bool; 5] {
    [
        expected.computed_styles() != observed.computed_styles(),
        expected.geometries() != observed.geometries(),
        expected.semantics() != observed.semantics(),
        expected.hit_regions() != observed.hit_regions(),
        expected.scene_rectangles() != observed.scene_rectangles(),
    ]
}

#[test]
fn each_registered_fault_changes_only_its_projection_family() {
    let expected = initial_projection();
    let cases = [
        (
            HeadlessProjectionFaultV1::ComputedStyle,
            HeadlessMismatchKindV1::ComputedStyle,
            HeadlessMismatchFieldV1::Width,
            root_path(),
            [true, false, false, false, false],
        ),
        (
            HeadlessProjectionFaultV1::GeometryOrder,
            HeadlessMismatchKindV1::Geometry,
            HeadlessMismatchFieldV1::Path,
            root_path(),
            [false, true, false, false, false],
        ),
        (
            HeadlessProjectionFaultV1::SemanticMembership,
            HeadlessMismatchKindV1::Semantics,
            HeadlessMismatchFieldV1::Path,
            control_path(),
            [false, false, true, false, false],
        ),
        (
            HeadlessProjectionFaultV1::HitOrder,
            HeadlessMismatchKindV1::HitRegions,
            HeadlessMismatchFieldV1::Path,
            control_path(),
            [false, false, false, true, false],
        ),
        (
            HeadlessProjectionFaultV1::SceneOutput,
            HeadlessMismatchKindV1::SceneRectangles,
            HeadlessMismatchFieldV1::Rectangle,
            root_path(),
            [false, false, false, false, true],
        ),
    ];

    for (fault, kind, field, path, changed) in cases {
        let observed = inject_headless_projection_fault_v1(&expected, fault)
            .expect("registered fault should apply");
        assert_eq!(observed.surface(), expected.surface());
        assert_eq!(changed_families(&expected, &observed), changed);

        let mismatch = compare_headless_projection_v1(&expected, &observed)
            .expect("fault should preserve the surface")
            .expect("registered fault should mismatch");
        assert_eq!(mismatch.kind(), kind);
        assert_eq!(mismatch.index(), 0);
        assert_eq!(mismatch.field(), field);
        assert_eq!(mismatch_path(mismatch.location()), Some(&path));
    }
}

#[test]
fn mismatch_family_priority_is_closed_for_every_fault_suffix() {
    let expected = initial_projection();
    assert_eq!(
        HeadlessProjectionFaultV1::ALL,
        [
            HeadlessProjectionFaultV1::ComputedStyle,
            HeadlessProjectionFaultV1::GeometryOrder,
            HeadlessProjectionFaultV1::SemanticMembership,
            HeadlessProjectionFaultV1::HitOrder,
            HeadlessProjectionFaultV1::SceneOutput,
        ]
    );
    assert_eq!(
        HeadlessMismatchKindV1::ALL,
        [
            HeadlessMismatchKindV1::ComputedStyle,
            HeadlessMismatchKindV1::Geometry,
            HeadlessMismatchKindV1::Semantics,
            HeadlessMismatchKindV1::HitRegions,
            HeadlessMismatchKindV1::SceneRectangles,
        ]
    );

    for first in 0..HeadlessProjectionFaultV1::ALL.len() {
        let mut observed = expected.clone();
        for fault in &HeadlessProjectionFaultV1::ALL[first..] {
            observed = inject_headless_projection_fault_v1(&observed, *fault)
                .expect("registered faults should compose");
        }
        let mismatch = compare_headless_projection_v1(&expected, &observed)
            .expect("faults should preserve the surface")
            .expect("fault suffix should mismatch");
        assert_eq!(mismatch.kind(), HeadlessMismatchKindV1::ALL[first]);
    }
}

#[test]
fn mismatch_field_priority_is_observable_without_exposing_values() {
    let expected = initial_projection();
    let computed =
        inject_headless_projection_fault_v1(&expected, HeadlessProjectionFaultV1::ComputedStyle)
            .expect("computed fault should apply");
    let scene =
        inject_headless_projection_fault_v1(&expected, HeadlessProjectionFaultV1::SceneOutput)
            .expect("scene fault should apply");

    let computed_mismatch = compare_headless_projection_v1(&expected, &computed)
        .expect("fault should preserve the surface")
        .expect("computed fault should mismatch");
    let scene_mismatch = compare_headless_projection_v1(&expected, &scene)
        .expect("fault should preserve the surface")
        .expect("scene fault should mismatch");

    assert_ne!(
        expected.computed_styles()[0].width(),
        computed.computed_styles()[0].width()
    );
    assert_ne!(
        expected.computed_styles()[0].height(),
        computed.computed_styles()[0].height()
    );
    assert_ne!(
        expected.scene_rectangles()[0].rectangle(),
        scene.scene_rectangles()[0].rectangle()
    );
    assert_ne!(
        expected.scene_rectangles()[0].color(),
        scene.scene_rectangles()[0].color()
    );
    assert_eq!(computed_mismatch.field(), HeadlessMismatchFieldV1::Width);
    assert_eq!(scene_mismatch.field(), HeadlessMismatchFieldV1::Rectangle);
}
