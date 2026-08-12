use super::{BasePlacement, arithmetic, fixed, placement, trusted_index};
use crate::model::{SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialScalarV2};
use crate::numeric::scalar_from_i128;
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::resolve_error::SpatialResolveErrorV2;
use crate::topology::{SpatialNodeV2, SpatialPlacementV2};

pub(super) fn resolve(
    node: u32,
    nodes: &[SpatialNodeV2],
    placements: &[Option<BasePlacement>],
) -> Result<BasePlacement, SpatialResolveErrorV2> {
    let input = nodes[trusted_index(node)];
    let SpatialPlacementV2::Free(free) = input.placement() else {
        unreachable!("a free dependency unit retains a Free placement")
    };
    let parent = input
        .parent()
        .expect("phase two validated every nonroot parent")
        .get();
    let parent_placement = placement(placements, parent);
    let target_placement = match free.target() {
        SpatialAnchorTargetV2::Viewport => placement(placements, 0),
        SpatialAnchorTargetV2::Parent => parent_placement,
        SpatialAnchorTargetV2::Node(target) => placement(placements, target.get()),
    };
    let width = fixed(free.width());
    let height = fixed(free.height());
    let target_anchor = free.target_anchor();
    let self_anchor = free.self_anchor();
    let offset = free.offset();

    let target_x = select(
        target_placement.x,
        target_placement.far_x,
        target_anchor.horizontal(),
    );
    let target_y = select(
        target_placement.y,
        target_placement.far_y,
        target_anchor.vertical(),
    );
    let self_x = select(SpatialScalarV2::new(0), width, self_anchor.horizontal());
    let self_y = select(SpatialScalarV2::new(0), height, self_anchor.vertical());

    let target_x = arithmetic(
        target_x.checked_add(offset.x()),
        SpatialArithmeticOperationV2::TargetOffsetX,
        node,
    )?;
    let target_y = arithmetic(
        target_y.checked_add(offset.y()),
        SpatialArithmeticOperationV2::TargetOffsetY,
        node,
    )?;
    let x = arithmetic(
        target_x.checked_sub(self_x),
        SpatialArithmeticOperationV2::SelfSubtractionX,
        node,
    )?;
    let y = arithmetic(
        target_y.checked_sub(self_y),
        SpatialArithmeticOperationV2::SelfSubtractionY,
        node,
    )?;
    let far_x = arithmetic(
        x.checked_add(width),
        SpatialArithmeticOperationV2::BaseFarX,
        node,
    )?;
    let far_y = arithmetic(
        y.checked_add(height),
        SpatialArithmeticOperationV2::BaseFarY,
        node,
    )?;
    let local_x = arithmetic(
        x.checked_sub(parent_placement.x),
        SpatialArithmeticOperationV2::ParentDeltaX,
        node,
    )?;
    let local_y = arithmetic(
        y.checked_sub(parent_placement.y),
        SpatialArithmeticOperationV2::ParentDeltaY,
        node,
    )?;

    Ok(BasePlacement {
        x,
        y,
        width: free.width(),
        height: free.height(),
        far_x,
        far_y,
        local_x,
        local_y,
    })
}

fn select(
    start: SpatialScalarV2,
    end: SpatialScalarV2,
    component: SpatialAnchorComponentV2,
) -> SpatialScalarV2 {
    match component {
        SpatialAnchorComponentV2::Start => start,
        SpatialAnchorComponentV2::Center => {
            let start = i128::from(start.raw());
            let end = i128::from(end.raw());
            scalar_from_i128(start + (end - start) / 2)
                .expect("the center of a validated base box stays in domain")
        }
        SpatialAnchorComponentV2::End => end,
    }
}
