use crate::error::{HarnessError, HarnessErrorKind};

use super::types::{
    HeadlessProjectionFaultV1, NormalizedHeadlessProjectionV1, ProjectionRect, rect,
};

/// Applies one registered defect only to a normalized testkit projection.
pub fn inject_headless_projection_fault_v1(
    projection: &NormalizedHeadlessProjectionV1,
    fault: HeadlessProjectionFaultV1,
) -> Result<NormalizedHeadlessProjectionV1, HarnessError> {
    let mut faulted = projection.clone();
    match fault {
        HeadlessProjectionFaultV1::ComputedStyle => {
            let record = faulted.computed_styles.first_mut().ok_or_else(invalid)?;
            record.width = perturb(record.width);
            record.height = perturb(record.height);
        }
        HeadlessProjectionFaultV1::GeometryOrder => {
            ensure_pair(faulted.geometries.len())?;
            faulted.geometries.swap(0, 1);
        }
        HeadlessProjectionFaultV1::SemanticMembership => {
            if faulted.semantics.is_empty() {
                return Err(invalid());
            }
            faulted.semantics.remove(0);
        }
        HeadlessProjectionFaultV1::HitOrder => {
            ensure_pair(faulted.hit_regions.len())?;
            faulted.hit_regions.swap(0, 1);
        }
        HeadlessProjectionFaultV1::SceneOutput => {
            let record = faulted.scene_rectangles.first_mut().ok_or_else(invalid)?;
            record.rectangle = perturb_rect(record.rectangle);
            record.color[0] ^= 1;
        }
    }
    Ok(faulted)
}

fn perturb(value: i32) -> i32 {
    if value == i32::MAX {
        value - 1
    } else {
        value + 1
    }
}

fn perturb_rect(value: ProjectionRect) -> ProjectionRect {
    rect(perturb(value.x()), value.y(), value.width(), value.height())
}

fn ensure_pair(count: usize) -> Result<(), HarnessError> {
    if count < 2 { Err(invalid()) } else { Ok(()) }
}

fn invalid() -> HarnessError {
    HarnessError::new(HarnessErrorKind::InvalidOperation)
}
