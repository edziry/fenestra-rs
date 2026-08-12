//! Placement dependency-graph validation and stable dry ordering.

use super::local_bounds::LocalBoundsProof;
use super::make_resolve_error;
use crate::aggregate_input::SpatialInputV2;
use crate::error::{SpatialDependencyErrorKindV2, SpatialErrorLocationV2};
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::SpatialAnchorTargetV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::SpatialPlacementV2;

mod execution;
mod order;
mod targets;

pub(super) use execution::{BasePlacementProof, execute_dependency_graph};

#[cfg(test)]
pub(super) use execution::map_layout_execution_error;

#[cfg(test)]
mod facts;

#[derive(Clone, Copy)]
enum DependencyUnitKind {
    Free { node: u32 },
    Island { index: u32, host: u32 },
}

struct DependencyUnitPlan {
    ordinal: u32,
    kind: DependencyUnitKind,
    produced: Vec<u32>,
    incoming: Vec<usize>,
}

struct IslandUnitSeed {
    index: u32,
    ordinal: u32,
    host: u32,
    members: Vec<u32>,
}

pub(super) struct DependencyGraphProof<'a> {
    bounds: LocalBoundsProof<'a>,
    units: Vec<DependencyUnitPlan>,
    order: Vec<usize>,
    edge_count: u128,
}

impl<'a> DependencyGraphProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.bounds.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.bounds.limits()
    }

    fn take_prepared_island(
        &mut self,
        index: u32,
    ) -> fenestra_ui_layout::prototype::PreparedLayoutInputV1 {
        self.bounds.take_prepared_island(index)
    }

    pub(in crate::input_validation) fn into_parts(self) -> LocalBoundsProof<'a> {
        self.bounds
    }
}

pub(super) fn prepare_dependency_graph(
    bounds: LocalBoundsProof<'_>,
) -> Result<DependencyGraphProof<'_>, SpatialResolveErrorV2> {
    let input = bounds.input();
    let limits = bounds.limits();
    let nodes = input.topology().nodes();

    targets::validate_targets(nodes)?;

    let free_count = nodes
        .iter()
        .skip(1)
        .filter(|node| matches!(node.placement(), SpatialPlacementV2::Free(_)))
        .count();
    let island_count = bounds.dependency_islands().count();
    let vertex_count = free_count as u128 + island_count as u128;
    validate_dependency_fact(SpatialLimitKindV2::DependencyVertices, vertex_count, limits)?;

    let islands = bounds
        .dependency_islands()
        .map(|island| IslandUnitSeed {
            index: island.index(),
            ordinal: island.stable_ordinal(),
            host: island.host(),
            members: island.members().collect(),
        })
        .collect::<Vec<_>>();
    let capacity = free_count
        .checked_add(island_count)
        .expect("dependency units partition the nonroot node set");
    let (mut units, producers) = build_units(nodes, islands, capacity);
    build_incoming(nodes, &producers, &mut units);

    let edge_count = units.iter().map(|unit| unit.incoming.len() as u128).sum();
    validate_dependency_fact(SpatialLimitKindV2::DependencyEdges, edge_count, limits)?;

    let order = order::stable_order(&units).map_err(|ordinal| {
        dependency_error(
            SpatialDependencyErrorKindV2::Cycle,
            SpatialErrorLocationV2::Dependency { ordinal },
        )
    })?;

    Ok(DependencyGraphProof {
        bounds,
        units,
        order,
        edge_count,
    })
}

pub(super) fn validate_dependency_fact(
    kind: SpatialLimitKindV2,
    observed: u128,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    match kind {
        SpatialLimitKindV2::DependencyVertices | SpatialLimitKindV2::DependencyEdges => {}
        _ => unreachable!("non-dependency limit in dependency validation"),
    }
    let maximum = limits.limit(kind) as u128;
    if observed > maximum {
        return Err(SpatialResolveErrorV2::limit_exceeded(
            kind,
            SpatialErrorLocationV2::Input,
            observed,
            maximum,
        ));
    }
    Ok(())
}

fn build_units(
    nodes: &[crate::topology::SpatialNodeV2],
    islands: Vec<IslandUnitSeed>,
    capacity: usize,
) -> (Vec<DependencyUnitPlan>, Vec<Option<usize>>) {
    let mut islands = islands.into_iter().peekable();
    let mut units = Vec::with_capacity(capacity);
    let mut producers = vec![None; nodes.len()];

    for (node_index, node) in nodes.iter().copied().enumerate().skip(1) {
        let ordinal = trusted_ordinal(node_index);
        let (kind, produced) = match node.placement() {
            SpatialPlacementV2::Free(_) => {
                (DependencyUnitKind::Free { node: ordinal }, vec![ordinal])
            }
            SpatialPlacementV2::Layout(_)
                if islands
                    .peek()
                    .is_some_and(|island| island.ordinal == ordinal) =>
            {
                let island = islands.next().expect("the matching island is present");
                (
                    DependencyUnitKind::Island {
                        index: island.index,
                        host: island.host,
                    },
                    island.members,
                )
            }
            SpatialPlacementV2::Layout(_) => continue,
            SpatialPlacementV2::Root => {
                unreachable!("phase two retained only one root at node zero")
            }
        };

        let unit_index = units.len();
        for &produced_node in &produced {
            let slot = producers
                .get_mut(trusted_reference(produced_node))
                .expect("island planning retained only existing members");
            assert!(
                slot.replace(unit_index).is_none(),
                "dependency units produce disjoint node sets"
            );
        }
        units.push(DependencyUnitPlan {
            ordinal,
            kind,
            produced,
            incoming: Vec::new(),
        });
    }

    assert!(islands.next().is_none(), "every island became one unit");
    assert_eq!(units.len(), capacity, "every dependency unit was derived");
    (units, producers)
}

fn build_incoming(
    nodes: &[crate::topology::SpatialNodeV2],
    producers: &[Option<usize>],
    units: &mut [DependencyUnitPlan],
) {
    for unit in units {
        match unit.kind {
            DependencyUnitKind::Free { node } => {
                let input = nodes
                    .get(trusted_reference(node))
                    .expect("a free unit retains one existing node");
                let parent = input
                    .parent()
                    .expect("phase two validated every nonroot parent")
                    .get();
                push_producer(&mut unit.incoming, producers, parent);

                let SpatialPlacementV2::Free(free) = input.placement() else {
                    unreachable!("a free unit retains a free placement")
                };
                match free.target() {
                    SpatialAnchorTargetV2::Viewport => {}
                    SpatialAnchorTargetV2::Parent => {
                        push_producer(&mut unit.incoming, producers, parent);
                    }
                    SpatialAnchorTargetV2::Node(target) => {
                        push_producer(&mut unit.incoming, producers, target.get());
                    }
                }
            }
            DependencyUnitKind::Island { host, .. } => {
                push_producer(&mut unit.incoming, producers, host);
            }
        }
        unit.incoming.sort_unstable();
        unit.incoming.dedup();
    }
}

fn push_producer(incoming: &mut Vec<usize>, producers: &[Option<usize>], node: u32) {
    if node == 0 {
        return;
    }
    incoming.push(
        producers[trusted_reference(node)]
            .expect("every nonroot node has exactly one dependency producer"),
    );
}

fn dependency_error(
    kind: SpatialDependencyErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Dependency(kind), location)
}

fn trusted_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the dependency node capacity")
}

fn trusted_reference(index: u32) -> usize {
    usize::try_from(index).expect("phase one validated the dependency reference capacity")
}
