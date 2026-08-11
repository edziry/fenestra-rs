//! Stable dependency execution and base-placement resolution.

use fenestra_ui_layout::prototype::LayoutEngineV1;

use super::{DependencyGraphProof, DependencyUnitKind};
use crate::aabb::SpatialAabbV2;
use crate::aggregate_input::SpatialInputV2;
use crate::error::SpatialErrorLocationV2;
use crate::limits::SpatialLimitsV2;
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::SpatialNodeV2;

mod free;
mod layout;

#[cfg(test)]
mod facts;

#[cfg(test)]
pub(in crate::input_validation) use layout::map_layout_execution_error;

#[derive(Clone, Copy)]
struct BasePlacement {
    x: SpatialScalarV2,
    y: SpatialScalarV2,
    width: i32,
    height: i32,
    far_x: SpatialScalarV2,
    far_y: SpatialScalarV2,
    local_x: SpatialScalarV2,
    local_y: SpatialScalarV2,
}

struct IslandExecutionInput<'graph, 'a> {
    graph: &'graph DependencyGraphProof<'a>,
    index: u32,
    host: u32,
    members: &'graph [u32],
    nodes: &'a [SpatialNodeV2],
}

impl BasePlacement {
    const fn origin(self) -> SpatialPointV2 {
        SpatialPointV2::new(self.x, self.y)
    }

    const fn local_origin(self) -> SpatialPointV2 {
        SpatialPointV2::new(self.local_x, self.local_y)
    }

    fn local_bounds(self) -> SpatialAabbV2 {
        let zero = SpatialScalarV2::new(0);
        SpatialAabbV2::from_edges(zero, zero, fixed(self.width), fixed(self.height))
            .expect("validated base extents form closed local bounds")
    }
}

pub(in crate::input_validation) struct BasePlacementProof<'a> {
    graph: DependencyGraphProof<'a>,
    placements: Vec<BasePlacement>,
}

impl<'a> BasePlacementProof<'a> {
    pub(in crate::input_validation) fn input(&self) -> SpatialInputV2<'a> {
        self.graph.input()
    }

    pub(in crate::input_validation) fn limits(&self) -> SpatialLimitsV2 {
        self.graph.limits()
    }

    pub(in crate::input_validation) fn local_placement_origin(
        &self,
        index: usize,
    ) -> SpatialPointV2 {
        self.placements
            .get(index)
            .copied()
            .expect("world composition visits every retained placement")
            .local_origin()
    }

    pub(in crate::input_validation) fn base_local_bounds(&self, index: usize) -> SpatialAabbV2 {
        self.placements
            .get(index)
            .copied()
            .expect("world projection visits every retained placement")
            .local_bounds()
    }

    pub(in crate::input_validation) fn shape_fill_bounds(&self, shape: u32) -> SpatialAabbV2 {
        self.graph.bounds.shape_fill_bounds(shape)
    }

    pub(in crate::input_validation) fn shape_clip_bounds(&self, shape: u32) -> SpatialAabbV2 {
        self.graph.bounds.shape_clip_bounds(shape)
    }

    pub(in crate::input_validation) fn paint_local_bounds(&self, index: usize) -> SpatialAabbV2 {
        self.graph.bounds.paint_local_bounds(index)
    }

    pub(in crate::input_validation) fn hit_local_bounds(&self, index: usize) -> SpatialAabbV2 {
        self.graph.bounds.hit_local_bounds(index)
    }
}

pub(in crate::input_validation) fn execute_dependency_graph<'a, E: LayoutEngineV1 + ?Sized>(
    mut graph: DependencyGraphProof<'a>,
    engine: &E,
) -> Result<BasePlacementProof<'a>, SpatialResolveErrorV2> {
    let input = graph.input();
    let nodes = input.topology().nodes();
    let viewport = input.topology().viewport();
    let zero = SpatialScalarV2::new(0);
    let root_width = fixed(viewport.width());
    let root_height = fixed(viewport.height());
    let mut placements = vec![None; nodes.len()];
    placements[0] = Some(BasePlacement {
        x: zero,
        y: zero,
        width: viewport.width(),
        height: viewport.height(),
        far_x: root_width,
        far_y: root_height,
        local_x: zero,
        local_y: zero,
    });

    for order_index in 0..graph.order.len() {
        let unit_index = graph.order[order_index];
        match graph.units[unit_index].kind {
            DependencyUnitKind::Free { node } => {
                let placement = free::resolve(node, nodes, &placements)?;
                set_placement(&mut placements, node, placement);
            }
            DependencyUnitKind::Island { index, host } => {
                let prepared = graph.take_prepared_island(index);
                let members = &graph.units[unit_index].produced;
                layout::execute(
                    IslandExecutionInput {
                        graph: &graph,
                        index,
                        host,
                        members,
                        nodes,
                    },
                    engine,
                    prepared,
                    &mut placements,
                )?;
            }
        }
    }

    let placements = placements
        .into_iter()
        .map(|placement| placement.expect("every spatial node has one dependency producer"))
        .collect();
    Ok(BasePlacementProof { graph, placements })
}

fn set_placement(placements: &mut [Option<BasePlacement>], node: u32, value: BasePlacement) {
    let slot = placements
        .get_mut(trusted_index(node))
        .expect("dependency execution retained only existing node keys");
    assert!(
        slot.replace(value).is_none(),
        "dependency units produce each node exactly once"
    );
}

fn placement(placements: &[Option<BasePlacement>], node: u32) -> BasePlacement {
    placements[trusted_index(node)]
        .expect("the stable dependency order resolves every input before its consumer")
}

fn arithmetic(
    value: Option<SpatialScalarV2>,
    operation: SpatialArithmeticOperationV2,
    node: u32,
) -> Result<SpatialScalarV2, SpatialResolveErrorV2> {
    value.ok_or_else(|| {
        super::super::make_resolve_error(
            SpatialResolveErrorKindV2::Arithmetic(operation),
            SpatialErrorLocationV2::Node { index: node },
        )
    })
}

fn fixed(value: i32) -> SpatialScalarV2 {
    SpatialScalarV2::checked_from_i32(value)
        .expect("validated integer extents fit the canonical fixed-point domain")
}

fn trusted_index(index: u32) -> usize {
    usize::try_from(index).expect("phase one validated the spatial node capacity")
}
