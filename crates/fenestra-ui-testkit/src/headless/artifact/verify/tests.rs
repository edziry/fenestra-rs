use std::cell::Cell;

use fenestra_ui_ir::prototype::InputPolicy;
use fenestra_ui_runtime::prototype::{HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole};

use super::projection::{ProjectionDifferenceV1, first_projection_difference_v1};
use super::*;
use crate::headless::artifact::error::HeadlessArtifactCapacityKindV1;
use crate::headless::artifact::record::SemanticRecordV1;
use crate::headless::oracle::HeadlessMismatchFieldV1 as Field;
use crate::headless::runner::HeadlessResultV1;
use crate::semantic::NodePathV1;

#[test]
fn replay_failure_preserves_every_closed_cause() {
    let artifact = canonical_artifact();
    for cause in [
        HeadlessFailureCauseV1::Runtime,
        HeadlessFailureCauseV1::Projection,
        HeadlessFailureCauseV1::Oracle,
        HeadlessFailureCauseV1::Scheduler,
        HeadlessFailureCauseV1::Renderer,
        HeadlessFailureCauseV1::Trace,
    ] {
        let error = verify_with_fresh_v1(&artifact, || Err(cause))
            .expect_err("injected replay failure should be reported");
        assert_eq!(error.kind(), Kind::ReplayFailed(cause));
        assert_eq!(error.index(), None);
    }
}

#[test]
fn preflight_precedes_replay_and_replay_precedes_result() {
    let canonical = canonical_artifact();
    let called = Cell::new(false);
    let mut fixture = canonical.clone();
    fixture.metadata.fixture_revision += 1;
    let error = verify_with_fresh_v1(&fixture, || {
        called.set(true);
        Err(HeadlessFailureCauseV1::Trace)
    })
    .expect_err("fixture mismatch should precede replay");
    assert_eq!(error.kind(), Kind::FixtureMismatch);
    assert!(!called.get());

    let called = Cell::new(false);
    let mut capacity = canonical.clone();
    capacity.capacities.style -= 1;
    let error = verify_with_fresh_v1(&capacity, || {
        called.set(true);
        Err(HeadlessFailureCauseV1::Trace)
    })
    .expect_err("capacity mismatch should precede replay");
    assert_eq!(
        error.kind(),
        Kind::CapacityMismatch(HeadlessArtifactCapacityKindV1::Style)
    );
    assert!(!called.get());

    let mut result = canonical;
    result.result = HeadlessResultV1::Adapt;
    let error = verify_with_fresh_v1(&result, || Err(HeadlessFailureCauseV1::Renderer))
        .expect_err("replay failure should precede the stored result");
    assert_eq!(
        error.kind(),
        Kind::ReplayFailed(HeadlessFailureCauseV1::Renderer)
    );
}

#[test]
fn computed_fields_follow_path_then_wire_payload_order() {
    let expected = canonical_artifact();
    let mut stored = expected.clone();
    let original = expected.projection.computed[0].clone();
    {
        let record = &mut stored.projection.computed[0];
        record.path = record.path.clone().static_child(9);
        record.width += 1;
        record.height += 1;
        record.color[0] ^= 1;
        record.visible = !record.visible;
        record.input = other_input(record.input);
    }
    assert_projection_field(
        &stored,
        &expected,
        Kind::ComputedStyleMismatch,
        0,
        Field::Path,
    );

    stored.projection.computed[0].path = original.path.clone();
    assert_projection_field(
        &stored,
        &expected,
        Kind::ComputedStyleMismatch,
        0,
        Field::Width,
    );
    stored.projection.computed[0].width = original.width;
    assert_projection_field(
        &stored,
        &expected,
        Kind::ComputedStyleMismatch,
        0,
        Field::Height,
    );
    stored.projection.computed[0].height = original.height;
    assert_projection_field(
        &stored,
        &expected,
        Kind::ComputedStyleMismatch,
        0,
        Field::Color,
    );
    stored.projection.computed[0].color = original.color;
    assert_projection_field(
        &stored,
        &expected,
        Kind::ComputedStyleMismatch,
        0,
        Field::Visible,
    );
    stored.projection.computed[0].visible = original.visible;
    assert_projection_field(
        &stored,
        &expected,
        Kind::ComputedStyleMismatch,
        0,
        Field::Input,
    );
}

#[test]
fn geometry_semantic_hit_and_scene_fields_have_typed_priority() {
    let expected = canonical_artifact();

    let mut geometry = expected.clone();
    let original_geometry = expected.projection.geometry[0].clone();
    geometry.projection.geometry[0].bounds = changed_rect(original_geometry.bounds);
    geometry.projection.geometry[0].clip = changed_rect(original_geometry.clip);
    assert_projection_field(
        &geometry,
        &expected,
        Kind::GeometryMismatch,
        0,
        Field::Bounds,
    );
    geometry.projection.geometry[0].bounds = original_geometry.bounds;
    assert_projection_field(&geometry, &expected, Kind::GeometryMismatch, 0, Field::Clip);

    let mut semantic_expected = expected.clone();
    ensure_semantic(&mut semantic_expected);
    let mut semantic = semantic_expected.clone();
    let original_semantic = semantic_expected.projection.semantics[0].clone();
    semantic.projection.semantics[0].path = original_semantic.path.clone().static_child(9);
    semantic.projection.semantics[0].label = original_semantic.label.wrapping_add(1);
    assert_projection_field(
        &semantic,
        &semantic_expected,
        Kind::SemanticsMismatch,
        0,
        Field::Path,
    );
    semantic.projection.semantics[0].path = original_semantic.path;
    assert_projection_field(
        &semantic,
        &semantic_expected,
        Kind::SemanticsMismatch,
        0,
        Field::Label,
    );

    let mut hit = expected.clone();
    hit.projection.hits[0].rectangle = changed_rect(hit.projection.hits[0].rectangle);
    assert_projection_field(&hit, &expected, Kind::HitMismatch, 0, Field::Clip);

    let mut scene = expected.clone();
    let original_scene = expected.projection.scene[0].clone();
    scene.projection.scene[0].rectangle = changed_rect(original_scene.rectangle);
    scene.projection.scene[0].color[0] ^= 1;
    assert_projection_field(&scene, &expected, Kind::SceneMismatch, 0, Field::Rectangle);
    scene.projection.scene[0].rectangle = original_scene.rectangle;
    assert_projection_field(&scene, &expected, Kind::SceneMismatch, 0, Field::Color);
}

#[test]
fn family_order_and_cardinality_are_typed() {
    let expected = canonical_artifact();
    let mut stored = expected.clone();
    stored.projection.computed[0].width += 1;
    stored.projection.geometry[0].bounds = changed_rect(stored.projection.geometry[0].bounds);
    ensure_semantic(&mut stored);
    if expected.projection.semantics.is_empty() {
        stored.projection.semantics[0].label += 1;
    } else {
        stored.projection.semantics[0].label =
            expected.projection.semantics[0].label.wrapping_add(1);
    }
    stored.projection.hits[0].rectangle = changed_rect(stored.projection.hits[0].rectangle);
    stored.projection.scene[0].rectangle = changed_rect(stored.projection.scene[0].rectangle);
    assert_projection_field(
        &stored,
        &expected,
        Kind::ComputedStyleMismatch,
        0,
        Field::Width,
    );

    let mut shorter = expected.clone();
    let index = shorter.projection.scene.len() - 1;
    shorter.projection.scene.pop();
    assert_projection_field(&shorter, &expected, Kind::SceneMismatch, index, Field::Path);
}

fn canonical_artifact() -> HeadlessArtifactV1 {
    let run = run_headless_spine_v1().expect("registered fixed run should succeed");
    build_headless_artifact_v1(&run)
}

fn assert_projection_field(
    stored: &HeadlessArtifactV1,
    expected: &HeadlessArtifactV1,
    kind: Kind,
    index: usize,
    field: Field,
) {
    assert_eq!(
        first_projection_difference_v1(stored, expected),
        Some(ProjectionDifferenceV1 { kind, index, field })
    );
}

fn changed_rect(rectangle: HeadlessRect) -> HeadlessRect {
    HeadlessRect::new(
        rectangle.x().wrapping_add(1),
        rectangle.y(),
        rectangle.width(),
        rectangle.height(),
    )
}

fn ensure_semantic(artifact: &mut HeadlessArtifactV1) {
    if artifact.projection.semantics.is_empty() {
        artifact.projection.semantics.push(SemanticRecordV1 {
            path: NodePathV1::root(),
            role: HeadlessSemanticRole::Control,
            label: 1,
            action: HeadlessSemanticAction::Activate,
        });
    }
}

const fn other_input(input: InputPolicy) -> InputPolicy {
    match input {
        InputPolicy::Accept => InputPolicy::Ignore,
        InputPolicy::Ignore => InputPolicy::Accept,
    }
}
