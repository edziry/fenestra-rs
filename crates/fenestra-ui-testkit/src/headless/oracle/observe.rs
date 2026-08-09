use fenestra_ui_ir::prototype::{InputPolicy, PropertyValue};
use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, NodeId};

use crate::error::{HarnessError, HarnessErrorKind, HarnessLimitKind};
use crate::identity::IdentityIndexV1;
use crate::observe::observe_snapshot_indexed_v1;
use crate::semantic::NodePathV1;

use super::ORACLE_LIMITS;
use super::types::{
    NormalizedHeadlessComputedStyleV1, NormalizedHeadlessGeometryV1, NormalizedHeadlessHitRegionV1,
    NormalizedHeadlessProjectionV1, NormalizedHeadlessSceneRectangleV1,
    NormalizedHeadlessSemanticV1, ObservedHeadlessProjectionV1,
};
use crate::headless::fixture::HeadlessFixtureV1;

/// Normalizes one committed runtime projection without changing vector order.
pub fn observe_headless_projection_v1(
    fixture: &HeadlessFixtureV1,
    snapshot: &CommittedRuntimeSnapshot,
) -> Result<ObservedHeadlessProjectionV1, HarnessError> {
    let indexed =
        observe_snapshot_indexed_v1(fixture.style().construction(), snapshot, ORACLE_LIMITS)?;
    let identities = indexed.identities();
    let logical = indexed.normalized();
    let projection = snapshot.headless_projection().ok_or_else(state_error)?;
    let spec = fixture.spec();
    let capacity = spec.capacity();

    ensure_count(
        projection.computed_style_count(),
        capacity.computed_styles(),
        HarnessLimitKind::NormalizedNodes,
    )?;
    ensure_count(
        projection.geometry_count(),
        capacity.geometry(),
        HarnessLimitKind::NormalizedNodes,
    )?;
    if projection.computed_style_count() != logical.node_count()
        || projection.geometry_count() != logical.node_count()
    {
        return Err(state_error());
    }

    let mut computed_styles = Vec::with_capacity(projection.computed_style_count());
    for record in projection.computed_styles() {
        computed_styles.push(NormalizedHeadlessComputedStyleV1 {
            path: path(identities, record.node())?,
            width: scalar(record.property(spec.width()))?,
            height: scalar(record.property(spec.height()))?,
            color: color(record.property(spec.color()))?,
            visible: boolean(record.property(spec.visible()))?,
            input: input(record.property(spec.input()))?,
        });
    }

    let mut geometries = Vec::with_capacity(projection.geometry_count());
    for record in projection.geometries() {
        geometries.push(NormalizedHeadlessGeometryV1 {
            path: path(identities, record.node())?,
            bounds: record.bounds(),
            clip: record.clip(),
        });
    }

    ensure_derived(projection.semantic_count(), capacity.semantics())?;
    let mut semantics = Vec::with_capacity(projection.semantic_count());
    for record in projection.semantics() {
        semantics.push(NormalizedHeadlessSemanticV1 {
            path: path(identities, record.node())?,
            role: record.role(),
            label: record.label(),
            action: record.action(),
        });
    }

    ensure_derived(projection.hit_region_count(), capacity.hit_regions())?;
    let mut hit_regions = Vec::with_capacity(projection.hit_region_count());
    for record in projection.hit_regions() {
        hit_regions.push(NormalizedHeadlessHitRegionV1 {
            path: path(identities, record.node())?,
            clip: record.clip(),
        });
    }

    ensure_derived(
        projection.scene_rectangle_count(),
        capacity.scene_rectangles(),
    )?;
    let mut scene_rectangles = Vec::with_capacity(projection.scene_rectangle_count());
    for record in projection.scene_rectangles() {
        scene_rectangles.push(NormalizedHeadlessSceneRectangleV1 {
            path: path(identities, record.node())?,
            rectangle: record.rectangle(),
            color: record.color(),
        });
    }

    Ok(ObservedHeadlessProjectionV1 {
        generation: projection.generation(),
        projection: NormalizedHeadlessProjectionV1 {
            surface: projection.surface(),
            computed_styles,
            geometries,
            semantics,
            hit_regions,
            scene_rectangles,
        },
    })
}

fn path(identities: &IdentityIndexV1, node: NodeId) -> Result<NodePathV1, HarnessError> {
    identities.node_path(node).cloned().ok_or_else(state_error)
}

fn scalar(value: Option<&PropertyValue>) -> Result<i32, HarnessError> {
    match value {
        Some(PropertyValue::ScalarI32(value)) => Ok(*value),
        _ => Err(state_error()),
    }
}

fn boolean(value: Option<&PropertyValue>) -> Result<bool, HarnessError> {
    match value {
        Some(PropertyValue::Bool(value)) => Ok(*value),
        _ => Err(state_error()),
    }
}

fn color(value: Option<&PropertyValue>) -> Result<[u8; 4], HarnessError> {
    match value {
        Some(PropertyValue::Rgba8(value)) => Ok(*value),
        _ => Err(state_error()),
    }
}

fn input(value: Option<&PropertyValue>) -> Result<InputPolicy, HarnessError> {
    match value {
        Some(PropertyValue::InputPolicy(value)) => Ok(*value),
        _ => Err(state_error()),
    }
}

fn ensure_count(count: usize, limit: usize, kind: HarnessLimitKind) -> Result<(), HarnessError> {
    if count > limit {
        Err(HarnessError::limit(kind))
    } else {
        Ok(())
    }
}

fn ensure_derived(count: usize, limit: usize) -> Result<(), HarnessError> {
    if count > limit {
        Err(state_error())
    } else {
        Ok(())
    }
}

fn state_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::StateMismatch)
}
