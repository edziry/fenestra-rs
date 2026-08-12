use fenestra_ui_ir::prototype::{
    SpatialBrushContentV2, SpatialShapeGeometryV2, ValidatedSpatialProgramV2,
};
use fenestra_ui_spatial::prototype::{SpatialLimitsV2, preflight_spatial_direct_counts_v2};

use super::super::error::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};
use super::model::LiveProgram;

#[derive(Clone, Copy)]
pub(super) struct DirectCounts([u128; 12]);

impl DirectCounts {
    pub(super) const fn observed(self) -> [u128; 12] {
        self.0
    }
}

pub(super) fn count_direct(
    program: &ValidatedSpatialProgramV2,
    live: &LiveProgram<'_>,
) -> Result<DirectCounts, RuntimeSpatialIrErrorV2> {
    let mut counts = [0u128; 12];
    counts[0] = 1;

    for owner in live.expanded() {
        let declaration = owner.declaration();
        increment(&mut counts[0], 1, program)?;
        increment(&mut counts[1], widened(declaration.shapes().len()), program)?;
        increment(
            &mut counts[2],
            widened(declaration.brushes().len()),
            program,
        )?;
        increment(&mut counts[3], widened(declaration.clips().len()), program)?;
        increment(
            &mut counts[4],
            widened(declaration.paint_items().len()),
            program,
        )?;
        increment(
            &mut counts[5],
            widened(declaration.hit_items().len()),
            program,
        )?;
        increment(
            &mut counts[6],
            widened(declaration.semantic_items().len()),
            program,
        )?;

        for shape in declaration.shapes() {
            match shape.geometry() {
                SpatialShapeGeometryV2::Path { verbs } => {
                    increment(&mut counts[7], 1, program)?;
                    increment(&mut counts[8], widened(verbs.len()), program)?;
                }
                SpatialShapeGeometryV2::Polygon { points } => {
                    increment(&mut counts[9], widened(points.len()), program)?;
                }
                SpatialShapeGeometryV2::Rect { .. } | SpatialShapeGeometryV2::Circle { .. } => {}
            }
        }
        for brush in declaration.brushes() {
            if let SpatialBrushContentV2::LinearGradient { stops, .. } = brush.content() {
                increment(&mut counts[10], widened(stops.len()), program)?;
            }
        }
    }

    counts[11] = widened(program.program().images().len());
    Ok(DirectCounts(counts))
}

pub(super) fn preflight_counts(
    program: &ValidatedSpatialProgramV2,
    counts: DirectCounts,
    limits: SpatialLimitsV2,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    preflight_spatial_direct_counts_v2(counts.observed(), limits).map_err(|error| {
        RuntimeSpatialIrErrorV2::new(
            RuntimeSpatialIrErrorKindV2::Resolve(error),
            program.program().span(),
        )
    })
}

fn widened(value: usize) -> u128 {
    u128::try_from(value).expect("usize always fits in u128")
}

fn increment(
    count: &mut u128,
    amount: u128,
    program: &ValidatedSpatialProgramV2,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    *count = count.checked_add(amount).ok_or_else(|| {
        RuntimeSpatialIrErrorV2::new(
            RuntimeSpatialIrErrorKindV2::ArithmeticExhausted,
            program.program().span(),
        )
    })?;
    Ok(())
}
