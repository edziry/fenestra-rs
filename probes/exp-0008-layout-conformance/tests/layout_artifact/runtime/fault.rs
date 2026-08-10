use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet};
use fenestra_ui_runtime::prototype::{HeadlessRect, HeadlessSurface};

use super::{
    RuntimeArtifactFaultV1, RuntimeArtifactV1, RuntimeExecutionLaneV1, RuntimeGeometryFieldV1,
    RuntimeHitFieldV1, RuntimeProjectionFieldV1, RuntimeProjectionLaneV1, RuntimeReceiptFieldV1,
    RuntimeSceneFieldV1,
};

pub(crate) fn inject_runtime_artifact_fault_v1(
    model: &RuntimeArtifactV1,
    fault: RuntimeArtifactFaultV1,
) -> RuntimeArtifactV1 {
    let mut faulted = model.clone();
    match fault {
        RuntimeArtifactFaultV1::Receipt {
            milestone,
            lane,
            field,
        } => mutate_receipt(&mut faulted, milestone, lane, field),
        RuntimeArtifactFaultV1::Projection {
            milestone,
            lane,
            field,
        } => mutate_projection(&mut faulted, milestone, lane, field),
        RuntimeArtifactFaultV1::Geometry {
            milestone,
            lane,
            record,
            field,
        } => {
            let geometry = &mut projection(&mut faulted, milestone, lane).geometries[record];
            match field {
                RuntimeGeometryFieldV1::Path => perturb_path(&mut geometry.path),
                RuntimeGeometryFieldV1::Bounds => geometry.bounds = perturb_rect(geometry.bounds),
                RuntimeGeometryFieldV1::Clip => geometry.clip = perturb_rect(geometry.clip),
            }
        }
        RuntimeArtifactFaultV1::Hit {
            milestone,
            lane,
            record,
            field,
        } => {
            let hit = &mut projection(&mut faulted, milestone, lane).hits[record];
            match field {
                RuntimeHitFieldV1::Path => perturb_path(&mut hit.path),
                RuntimeHitFieldV1::Clip => hit.clip = perturb_rect(hit.clip),
            }
        }
        RuntimeArtifactFaultV1::Scene {
            milestone,
            lane,
            record,
            field,
        } => {
            let scene = &mut projection(&mut faulted, milestone, lane).scenes[record];
            match field {
                RuntimeSceneFieldV1::Path => perturb_path(&mut scene.path),
                RuntimeSceneFieldV1::Rectangle => {
                    scene.rectangle = perturb_rect(scene.rectangle);
                }
                RuntimeSceneFieldV1::Color => scene.color[0] ^= 1,
            }
        }
    }
    faulted
}

fn mutate_receipt(
    model: &mut RuntimeArtifactV1,
    milestone: usize,
    lane: RuntimeExecutionLaneV1,
    field: RuntimeReceiptFieldV1,
) {
    let milestone = &mut model.milestones[milestone];
    let receipt = match lane {
        RuntimeExecutionLaneV1::Reference => &mut milestone.reference_receipt,
        RuntimeExecutionLaneV1::Candidate => &mut milestone.candidate_receipt,
    };
    match field {
        RuntimeReceiptFieldV1::ReceiptGeneration => {
            receipt.receipt_generation = Some(receipt.receipt_generation.map_or(0, perturb_u64));
        }
        RuntimeReceiptFieldV1::ProjectionGeneration => {
            receipt.projection_generation = perturb_u64(receipt.projection_generation);
        }
        RuntimeReceiptFieldV1::Invalidation => {
            receipt.invalidation = perturb_invalidation(receipt.invalidation);
        }
        RuntimeReceiptFieldV1::MutationCount => {
            receipt.mutation_count = perturb_usize(receipt.mutation_count);
        }
    }
}

fn mutate_projection(
    model: &mut RuntimeArtifactV1,
    milestone: usize,
    lane: RuntimeProjectionLaneV1,
    field: RuntimeProjectionFieldV1,
) {
    let projection = projection(model, milestone, lane);
    match field {
        RuntimeProjectionFieldV1::Surface => {
            projection.surface = HeadlessSurface::new(
                perturb_i32(projection.surface.width()),
                projection.surface.height(),
            );
        }
        RuntimeProjectionFieldV1::ComputedStyleCount => {
            projection.counts.computed_styles = perturb_usize(projection.counts.computed_styles);
        }
        RuntimeProjectionFieldV1::GeometryCount => {
            projection.counts.geometry = perturb_usize(projection.counts.geometry);
        }
        RuntimeProjectionFieldV1::SemanticsCount => {
            projection.counts.semantics = perturb_usize(projection.counts.semantics);
        }
        RuntimeProjectionFieldV1::HitRegionCount => {
            projection.counts.hit_regions = perturb_usize(projection.counts.hit_regions);
        }
        RuntimeProjectionFieldV1::SceneRectangleCount => {
            projection.counts.scene_rectangles = perturb_usize(projection.counts.scene_rectangles);
        }
    }
}

fn projection(
    model: &mut RuntimeArtifactV1,
    milestone: usize,
    lane: RuntimeProjectionLaneV1,
) -> &mut super::RuntimeProjectionV1 {
    let milestone = &mut model.milestones[milestone];
    match lane {
        RuntimeProjectionLaneV1::Oracle => &mut milestone.oracle_projection,
        RuntimeProjectionLaneV1::Reference => &mut milestone.reference_projection,
        RuntimeProjectionLaneV1::Candidate => &mut milestone.candidate_projection,
    }
}

fn perturb_path(path: &mut fenestra_ui_testkit::prototype::NodePathV1) {
    *path = path.clone().static_child(u16::MAX);
}

fn perturb_rect(rect: HeadlessRect) -> HeadlessRect {
    HeadlessRect::new(perturb_i32(rect.x()), rect.y(), rect.width(), rect.height())
}

fn perturb_invalidation(value: InvalidationSet) -> InvalidationSet {
    for class in [
        InvalidationClass::Structure,
        InvalidationClass::StyleMatch,
        InvalidationClass::Intrinsic,
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
        InvalidationClass::Surface,
    ] {
        if !value.contains(class) {
            return value.union(InvalidationSet::from_class(class));
        }
    }
    InvalidationSet::NONE
}

fn perturb_i32(value: i32) -> i32 {
    if value == i32::MAX {
        value - 1
    } else {
        value + 1
    }
}

fn perturb_u64(value: u64) -> u64 {
    if value == u64::MAX {
        value - 1
    } else {
        value + 1
    }
}

fn perturb_usize(value: usize) -> usize {
    if value == usize::MAX {
        value - 1
    } else {
        value + 1
    }
}
