//! Exact world-transform composition after base placement resolves.

use super::dependencies::BasePlacementProof;
use super::make_resolve_error;
use super::topology::trusted_node_ordinal;
use crate::aggregate_input::SpatialInputV2;
use crate::error::SpatialErrorLocationV2;
use crate::limits::SpatialLimitsV2;
use crate::model::{Affine2V2, SpatialLocalTransformV2, SpatialPointV2};
use crate::numeric_error::{SpatialArithmeticOperationV2, SpatialTransformErrorKindV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::{SpatialNodeV2, SpatialPlacementV2};
use crate::vocabulary::SpatialTransformStageV2;

#[cfg(test)]
mod facts;

pub(super) struct WorldTransformProof<'a> {
    placements: BasePlacementProof<'a>,
    world: Vec<Affine2V2>,
}

impl<'a> WorldTransformProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.placements.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.placements.limits()
    }
}

pub(super) fn prepare_world_transforms(
    placements: BasePlacementProof<'_>,
) -> Result<WorldTransformProof<'_>, SpatialResolveErrorV2> {
    let input = placements.input();
    let nodes = input.topology().nodes();
    let mut world = Vec::with_capacity(nodes.len());
    world.push(Affine2V2::identity());

    for (index, node) in nodes.iter().copied().enumerate().skip(1) {
        let ordinal = trusted_node_ordinal(index);
        let local = local_transform(node);
        let origin = local.origin();
        let negative_origin = SpatialPointV2::new(
            origin
                .x()
                .checked_neg()
                .expect("the validated scalar domain is symmetric"),
            origin
                .y()
                .checked_neg()
                .expect("the validated scalar domain is symmetric"),
        );

        let inner_about = compose(
            local.affine(),
            Affine2V2::translation(negative_origin.x(), negative_origin.y()),
            SpatialTransformStageV2::About,
            ordinal,
        )?;
        let about = compose(
            Affine2V2::translation(origin.x(), origin.y()),
            inner_about,
            SpatialTransformStageV2::About,
            ordinal,
        )?;
        validate_determinant(about, SpatialTransformStageV2::About, ordinal)?;

        let local_origin = placements.local_placement_origin(index);
        let placed = compose(
            Affine2V2::translation(local_origin.x(), local_origin.y()),
            about,
            SpatialTransformStageV2::Placed,
            ordinal,
        )?;
        validate_determinant(placed, SpatialTransformStageV2::Placed, ordinal)?;

        let parent = node
            .parent()
            .expect("phase two validated every nonroot parent")
            .get();
        let parent_world = world[trusted_reference(parent)];
        let composed = compose(
            parent_world,
            placed,
            SpatialTransformStageV2::World,
            ordinal,
        )?;
        validate_determinant(composed, SpatialTransformStageV2::World, ordinal)?;
        world.push(composed);
    }

    Ok(WorldTransformProof { placements, world })
}

fn local_transform(node: SpatialNodeV2) -> SpatialLocalTransformV2 {
    match node.placement() {
        SpatialPlacementV2::Layout(layout) => layout.transform(),
        SpatialPlacementV2::Free(free) => free.transform(),
        SpatialPlacementV2::Root => {
            unreachable!("phase two retained Root placement only at the sentinel")
        }
    }
}

fn compose(
    left: Affine2V2,
    right: Affine2V2,
    stage: SpatialTransformStageV2,
    node: u32,
) -> Result<Affine2V2, SpatialResolveErrorV2> {
    left.checked_compose(right).map_err(|component| {
        make_resolve_error(
            SpatialResolveErrorKindV2::Arithmetic(SpatialArithmeticOperationV2::Affine {
                stage,
                component,
            }),
            SpatialErrorLocationV2::Node { index: node },
        )
    })
}

fn validate_determinant(
    transform: Affine2V2,
    stage: SpatialTransformStageV2,
    node: u32,
) -> Result<(), SpatialResolveErrorV2> {
    if transform.determinant_raw() == 0 {
        return Err(make_resolve_error(
            SpatialResolveErrorKindV2::Transform(
                SpatialTransformErrorKindV2::ComposedTransformSingular(stage),
            ),
            SpatialErrorLocationV2::Node { index: node },
        ));
    }
    Ok(())
}

fn trusted_reference(index: u32) -> usize {
    usize::try_from(index).expect("phase one validated the spatial node capacity")
}
