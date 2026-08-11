//! Layout-island derivation, capacities, and preparation-item planning.

use super::placement::PlacementInputProof;
use super::topology::trusted_node_ordinal;
use crate::error::SpatialErrorLocationV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::SpatialResolveErrorV2;
use crate::topology::SpatialPlacementV2;

pub(super) mod preflight;

#[cfg(test)]
pub(super) use preflight::{map_layout_preflight_error, prepare_layout_preflight};

struct IslandMemberPlan {
    node: u32,
    parent: u32,
}

struct IslandPlan {
    stable_key: u32,
    host: u32,
    members: Vec<IslandMemberPlan>,
}

enum PreparationItemPlan {
    Singleton { node: u32 },
    Island { index: u32 },
}

pub(super) struct IslandPlanningProof<'a> {
    placement: PlacementInputProof<'a>,
    islands: Vec<IslandPlan>,
    items: Vec<PreparationItemPlan>,
}

#[cfg(test)]
type IslandMemberFact = (u32, u32);

#[cfg(test)]
type IslandFact = (u32, u32, u32, Vec<IslandMemberFact>);

pub(super) fn prepare_island_plan(
    placement: PlacementInputProof<'_>,
) -> Result<IslandPlanningProof<'_>, SpatialResolveErrorV2> {
    let nodes = placement.input().topology().nodes();
    let limits = placement.limits();
    let mut hosted_islands = vec![None; nodes.len()];
    let mut member_assignments = vec![None; nodes.len()];
    let mut islands = Vec::new();

    for (node_index, node) in nodes.iter().copied().enumerate().skip(1) {
        if !matches!(node.placement(), SpatialPlacementV2::Layout(_)) {
            continue;
        }

        let parent_index = usize::try_from(
            node.parent()
                .expect("phase two validated every nonroot parent")
                .get(),
        )
        .expect("phase one validated the node row capacity");
        let parent_node = nodes
            .get(parent_index)
            .expect("phase two validated every parent reference");

        let (island_index, local_parent) = match parent_node.placement() {
            SpatialPlacementV2::Layout(_) => member_assignments[parent_index]
                .expect("a layout parent was assigned before its child"),
            SpatialPlacementV2::Root | SpatialPlacementV2::Free(_) => {
                let island_index = match hosted_islands[parent_index] {
                    Some(index) => index,
                    None => {
                        let index = islands.len();
                        islands.push(IslandPlan {
                            stable_key: trusted_node_ordinal(node_index),
                            host: trusted_node_ordinal(parent_index),
                            members: Vec::new(),
                        });
                        hosted_islands[parent_index] = Some(index);
                        index
                    }
                };
                (island_index, 0)
            }
        };

        let local_key = u32::try_from(islands[island_index].members.len() + 1)
            .expect("phase one validated the node row capacity");
        islands[island_index].members.push(IslandMemberPlan {
            node: trusted_node_ordinal(node_index),
            parent: local_parent,
        });
        member_assignments[node_index] = Some((island_index, local_key));
    }

    validate_island_fact(
        SpatialLimitKindV2::Islands,
        None,
        islands.len() as u128,
        limits,
    )?;
    for (index, island) in islands.iter().enumerate() {
        validate_island_fact(
            SpatialLimitKindV2::LayoutInputRecordsPerIsland,
            Some(trusted_island_index(index)),
            island.members.len() as u128 + 1,
            limits,
        )?;
    }
    let total = islands
        .iter()
        .map(|island| island.members.len() as u128 + 1)
        .sum();
    validate_island_fact(
        SpatialLimitKindV2::LayoutInputRecordsTotal,
        None,
        total,
        limits,
    )?;

    let mut items = Vec::new();
    for (node_index, node) in nodes.iter().copied().enumerate() {
        match node.placement() {
            SpatialPlacementV2::Root | SpatialPlacementV2::Free(_)
                if hosted_islands[node_index].is_none() =>
            {
                items.push(PreparationItemPlan::Singleton {
                    node: trusted_node_ordinal(node_index),
                });
            }
            SpatialPlacementV2::Layout(_) => {
                let (island_index, local_key) = member_assignments[node_index]
                    .expect("every layout node belongs to one island");
                if local_key == 1 {
                    items.push(PreparationItemPlan::Island {
                        index: trusted_island_index(island_index),
                    });
                }
            }
            SpatialPlacementV2::Root | SpatialPlacementV2::Free(_) => {}
        }
    }

    Ok(IslandPlanningProof {
        placement,
        islands,
        items,
    })
}

pub(super) fn validate_island_fact(
    kind: SpatialLimitKindV2,
    index: Option<u32>,
    observed: u128,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let location = match (kind, index) {
        (SpatialLimitKindV2::Islands | SpatialLimitKindV2::LayoutInputRecordsTotal, None) => {
            SpatialErrorLocationV2::Input
        }
        (SpatialLimitKindV2::LayoutInputRecordsPerIsland, Some(index)) => {
            SpatialErrorLocationV2::Island { index }
        }
        _ => unreachable!("invalid island limit location"),
    };
    let maximum = limits.limit(kind) as u128;

    if observed > maximum {
        return Err(SpatialResolveErrorV2::limit_exceeded(
            kind, location, observed, maximum,
        ));
    }

    Ok(())
}

fn trusted_island_index(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the island index capacity")
}

#[cfg(test)]
impl IslandPlanningProof<'_> {
    pub(super) fn island_facts(&self) -> Vec<IslandFact> {
        self.islands
            .iter()
            .enumerate()
            .map(|(index, island)| {
                (
                    trusted_island_index(index),
                    island.stable_key,
                    island.host,
                    island
                        .members
                        .iter()
                        .map(|member| (member.node, member.parent))
                        .collect(),
                )
            })
            .collect()
    }

    pub(super) fn item_facts(&self) -> Vec<(u32, Option<u32>, Vec<u32>)> {
        self.items
            .iter()
            .map(|item| match *item {
                PreparationItemPlan::Singleton { node } => (node, None, vec![node]),
                PreparationItemPlan::Island { index } => {
                    let island = &self.islands[index as usize];
                    let mut owners = Vec::with_capacity(island.members.len() + 1);
                    owners.push(island.host);
                    owners.extend(island.members.iter().map(|member| member.node));
                    (island.stable_key, Some(index), owners)
                }
            })
            .collect()
    }
}
