//! Validation-only Layout V1 preparation for stable spatial items.

use fenestra_ui_layout::prototype::{
    LayoutConstraintFieldV1, LayoutDimensionV1, LayoutErrorKindV1, LayoutErrorLocationV1,
    LayoutExtentV1, LayoutInputErrorKindV1, LayoutInputV1, LayoutLimitsV1, LayoutNodeKeyV1,
    LayoutNodeV1, LayoutPaddingSideV1, LayoutStyleV1, LayoutViewportV1, PreparedLayoutInputV1,
    prepare_layout_v1,
};

use super::trusted_island_index;
use super::{IslandPlan, IslandPlanningProof, PreparationItemPlan};
use crate::error::{
    SpatialContainerErrorKindV2, SpatialErrorLocationV2, SpatialInputErrorKindV2,
    SpatialLayoutDimensionErrorKindV2,
};
use crate::input_validation::{make_resolve_error, topology::input_error};
use crate::resolve_error::{
    SpatialLayoutErrorKindV2, SpatialResolveErrorKindV2, SpatialResolveErrorV2,
};
use crate::topology::{SpatialNodeV2, SpatialPlacementV2};
use crate::vocabulary::SpatialNodeFieldV2;

struct PreparedIslandPlan {
    plan: IslandPlan,
    prepared: PreparedLayoutInputV1,
}

pub(in crate::input_validation) struct DependencyIslandInput<'a> {
    index: u32,
    plan: &'a IslandPlan,
}

impl DependencyIslandInput<'_> {
    pub(in crate::input_validation) const fn index(&self) -> u32 {
        self.index
    }

    pub(in crate::input_validation) const fn stable_ordinal(&self) -> u32 {
        self.plan.stable_key
    }

    pub(in crate::input_validation) const fn host(&self) -> u32 {
        self.plan.host
    }

    pub(in crate::input_validation) fn members(&self) -> impl Iterator<Item = u32> + '_ {
        self.plan.members.iter().map(|member| member.node)
    }
}

pub(in crate::input_validation) struct LayoutPreflightProof<'a> {
    placement: super::super::placement::PlacementInputProof<'a>,
    islands: Vec<PreparedIslandPlan>,
}

impl<'a> LayoutPreflightProof<'a> {
    pub(in crate::input_validation) fn input(&self) -> crate::aggregate_input::SpatialInputV2<'a> {
        self.placement.input()
    }

    pub(in crate::input_validation) fn limits(&self) -> crate::limits::SpatialLimitsV2 {
        self.placement.limits()
    }

    pub(in crate::input_validation) fn dependency_islands(
        &self,
    ) -> impl Iterator<Item = DependencyIslandInput<'_>> + '_ {
        self.islands
            .iter()
            .enumerate()
            .map(|(index, island)| DependencyIslandInput {
                index: trusted_island_index(index),
                plan: &island.plan,
            })
    }
}

struct LayoutDraft {
    viewport: LayoutViewportV1,
    nodes: Vec<LayoutNodeV1>,
    island: Option<usize>,
}

pub(in crate::input_validation) fn prepare_layout_preflight(
    plan: IslandPlanningProof<'_>,
) -> Result<LayoutPreflightProof<'_>, SpatialResolveErrorV2> {
    let mut prepared_islands = std::iter::repeat_with(|| None)
        .take(plan.islands.len())
        .collect::<Vec<_>>();

    for (item_ordinal, item) in plan.items.iter().enumerate() {
        let draft = build_draft(&plan, item);
        let record_count = draft.nodes.len();
        let limits = LayoutLimitsV1::new(record_count, record_count, record_count);
        let input = LayoutInputV1::new(draft.viewport, &draft.nodes);
        let prepared = prepare_layout_v1(input, limits).map_err(|error| {
            map_layout_preflight_error(&plan, item_ordinal, error.kind(), error.location())
        })?;

        if let Some(index) = draft.island {
            let slot = prepared_islands
                .get_mut(index)
                .expect("phase five derived a trusted island index");
            assert!(
                slot.replace(prepared).is_none(),
                "each island has exactly one preparation item"
            );
        }
    }

    let IslandPlanningProof {
        placement,
        islands,
        items: _,
    } = plan;
    let islands = islands
        .into_iter()
        .zip(prepared_islands)
        .map(|(plan, prepared)| PreparedIslandPlan {
            plan,
            prepared: prepared.expect("every island was prepared exactly once"),
        })
        .collect();

    Ok(LayoutPreflightProof { placement, islands })
}

pub(in crate::input_validation) fn map_layout_preflight_error(
    plan: &IslandPlanningProof<'_>,
    item_ordinal: usize,
    kind: LayoutErrorKindV1,
    location: LayoutErrorLocationV1,
) -> SpatialResolveErrorV2 {
    let item = plan
        .items
        .get(item_ordinal)
        .expect("layout preflight supplied a trusted item ordinal");
    let fallback = item_location(item);
    let LayoutErrorKindV1::Input(kind) = kind else {
        return bridge_invariant(fallback);
    };
    let LayoutErrorLocationV1::InputNode { index } = location else {
        return bridge_invariant(fallback);
    };
    let Some(owner) = record_owner(plan, item, index) else {
        return bridge_invariant(fallback);
    };

    match kind {
        LayoutInputErrorKindV1::NegativeConstraint { extent, field } if index > 0 => input_error(
            SpatialInputErrorKindV2::InvalidLayoutDimensions(
                SpatialLayoutDimensionErrorKindV2::NegativeConstraint { extent, field },
            ),
            SpatialErrorLocationV2::NodeField {
                index: owner,
                field: dimension_field(extent, field),
            },
        ),
        LayoutInputErrorKindV1::InvertedConstraint(extent) if index > 0 => input_error(
            SpatialInputErrorKindV2::InvalidLayoutDimensions(
                SpatialLayoutDimensionErrorKindV2::InvertedConstraint(extent),
            ),
            SpatialErrorLocationV2::Node { index: owner },
        ),
        LayoutInputErrorKindV1::NegativePadding(side) => input_error(
            SpatialInputErrorKindV2::InvalidContainer(
                SpatialContainerErrorKindV2::NegativePadding(side),
            ),
            SpatialErrorLocationV2::NodeField {
                index: owner,
                field: padding_field(side),
            },
        ),
        LayoutInputErrorKindV1::PaddingExceedsExtent(extent) => input_error(
            SpatialInputErrorKindV2::InvalidContainer(
                SpatialContainerErrorKindV2::PaddingExceedsExtent(extent),
            ),
            SpatialErrorLocationV2::Node { index: owner },
        ),
        LayoutInputErrorKindV1::NegativeGap => input_error(
            SpatialInputErrorKindV2::InvalidContainer(SpatialContainerErrorKindV2::NegativeGap),
            SpatialErrorLocationV2::NodeField {
                index: owner,
                field: SpatialNodeFieldV2::Gap,
            },
        ),
        _ => bridge_invariant(fallback),
    }
}

fn build_draft(plan: &IslandPlanningProof<'_>, item: &PreparationItemPlan) -> LayoutDraft {
    let topology = plan.placement.input().topology();
    let spatial_nodes = topology.nodes();
    let (host, island) = match *item {
        PreparationItemPlan::Singleton { node } => (node, None),
        PreparationItemPlan::Island { index } => {
            let island = &plan.islands[index as usize];
            (island.host, Some(index as usize))
        }
    };
    let host_node = spatial_node(spatial_nodes, host);
    let (width, height) = host_extent(
        host_node,
        topology.viewport().width(),
        topology.viewport().height(),
    );
    let mut nodes = Vec::new();
    nodes.push(LayoutNodeV1::new(
        LayoutNodeKeyV1::new(0),
        None,
        style(host_node, fixed(width), fixed(height)),
    ));

    if let Some(index) = island {
        for (member_index, member) in plan.islands[index].members.iter().enumerate() {
            let node = spatial_node(spatial_nodes, member.node);
            let SpatialPlacementV2::Layout(layout) = node.placement() else {
                unreachable!("phase five island members use Layout placement");
            };
            let key = u32::try_from(member_index + 1)
                .expect("phase one validated the layout record capacity");
            nodes.push(LayoutNodeV1::new(
                LayoutNodeKeyV1::new(key),
                Some(LayoutNodeKeyV1::new(member.parent)),
                style(node, layout.width(), layout.height()),
            ));
        }
    }

    LayoutDraft {
        viewport: LayoutViewportV1::new(width, height),
        nodes,
        island,
    }
}

fn spatial_node(nodes: &[SpatialNodeV2], key: u32) -> SpatialNodeV2 {
    nodes[key as usize]
}

fn host_extent(node: SpatialNodeV2, viewport_width: i32, viewport_height: i32) -> (i32, i32) {
    match node.placement() {
        SpatialPlacementV2::Root => (viewport_width, viewport_height),
        SpatialPlacementV2::Free(free) => (free.width(), free.height()),
        SpatialPlacementV2::Layout(_) => unreachable!("layout nodes cannot host their own island"),
    }
}

fn fixed(value: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(value, value, value)
}

fn style(
    node: SpatialNodeV2,
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
) -> LayoutStyleV1 {
    let container = node.container();
    LayoutStyleV1::new(
        container.axis(),
        width,
        height,
        container.padding(),
        container.gap(),
    )
}

fn record_owner(
    plan: &IslandPlanningProof<'_>,
    item: &PreparationItemPlan,
    record: u32,
) -> Option<u32> {
    match *item {
        PreparationItemPlan::Singleton { node } => (record == 0).then_some(node),
        PreparationItemPlan::Island { index } => {
            let island = plan.islands.get(index as usize)?;
            if record == 0 {
                Some(island.host)
            } else {
                island
                    .members
                    .get(record as usize - 1)
                    .map(|member| member.node)
            }
        }
    }
}

fn item_location(item: &PreparationItemPlan) -> SpatialErrorLocationV2 {
    match *item {
        PreparationItemPlan::Singleton { node } => SpatialErrorLocationV2::Node { index: node },
        PreparationItemPlan::Island { index } => SpatialErrorLocationV2::Island { index },
    }
}

fn dimension_field(extent: LayoutExtentV1, field: LayoutConstraintFieldV1) -> SpatialNodeFieldV2 {
    match (extent, field) {
        (LayoutExtentV1::Width, LayoutConstraintFieldV1::Minimum) => {
            SpatialNodeFieldV2::LayoutWidthMinimum
        }
        (LayoutExtentV1::Width, LayoutConstraintFieldV1::Preferred) => {
            SpatialNodeFieldV2::LayoutWidthPreferred
        }
        (LayoutExtentV1::Width, LayoutConstraintFieldV1::Maximum) => {
            SpatialNodeFieldV2::LayoutWidthMaximum
        }
        (LayoutExtentV1::Height, LayoutConstraintFieldV1::Minimum) => {
            SpatialNodeFieldV2::LayoutHeightMinimum
        }
        (LayoutExtentV1::Height, LayoutConstraintFieldV1::Preferred) => {
            SpatialNodeFieldV2::LayoutHeightPreferred
        }
        (LayoutExtentV1::Height, LayoutConstraintFieldV1::Maximum) => {
            SpatialNodeFieldV2::LayoutHeightMaximum
        }
    }
}

fn padding_field(side: LayoutPaddingSideV1) -> SpatialNodeFieldV2 {
    match side {
        LayoutPaddingSideV1::Left => SpatialNodeFieldV2::PaddingLeft,
        LayoutPaddingSideV1::Right => SpatialNodeFieldV2::PaddingRight,
        LayoutPaddingSideV1::Top => SpatialNodeFieldV2::PaddingTop,
        LayoutPaddingSideV1::Bottom => SpatialNodeFieldV2::PaddingBottom,
    }
}

fn bridge_invariant(location: SpatialErrorLocationV2) -> SpatialResolveErrorV2 {
    make_resolve_error(
        SpatialResolveErrorKindV2::Layout(SpatialLayoutErrorKindV2::BridgeInvariant),
        location,
    )
}

#[cfg(test)]
impl LayoutPreflightProof<'_> {
    pub(in crate::input_validation) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.islands
            .iter()
            .enumerate()
            .map(|(index, prepared)| {
                let mut owners = Vec::with_capacity(prepared.plan.members.len() + 1);
                owners.push(prepared.plan.host);
                owners.extend(prepared.plan.members.iter().map(|member| member.node));
                (trusted_island_index(index), owners)
            })
            .collect()
    }
}
