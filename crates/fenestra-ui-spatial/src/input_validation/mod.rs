//! Private aggregate input validation kernel.

// Staged validation proofs remain private until the resolver orchestration lands.
#![allow(dead_code)]

use crate::aggregate_input::SpatialInputV2;
use crate::error::SpatialErrorLocationV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

mod islands;
mod path_structure;
mod placement;
mod shape_structure;
mod topology;
mod transforms;
mod validated_paths;

#[cfg(test)]
use islands::{
    map_layout_preflight_error, prepare_island_plan, prepare_layout_preflight, validate_island_fact,
};

#[cfg(test)]
use placement::prepare_placement_input;

#[cfg(test)]
use path_structure::prepare_path_structure;

#[cfg(test)]
use shape_structure::{prepare_shape_structure, validate_polygon_range};

#[cfg(test)]
use topology::{prepare_topology, prepare_topology_limits, validate_topology_fact};

#[cfg(test)]
use transforms::prepare_local_transforms;

#[cfg(test)]
use validated_paths::{map_path_k1_error, prepare_validated_paths};

const U32_ROW_CAPACITY: u128 = u32::MAX as u128 + 1;

struct DirectCountProof<'a> {
    input: SpatialInputV2<'a>,
    limits: SpatialLimitsV2,
}

fn prepare_direct_counts(
    input: SpatialInputV2<'_>,
    limits: SpatialLimitsV2,
) -> Result<DirectCountProof<'_>, SpatialResolveErrorV2> {
    let topology = input.topology();
    let geometry = input.geometry();
    let resources = input.resources();
    let items = input.items();
    let observed = [
        topology.nodes().len(),
        geometry.shapes().len(),
        resources.brushes().len(),
        geometry.clips().len(),
        items.paint_items().len(),
        items.hit_items().len(),
        items.semantic_items().len(),
        geometry.paths().len(),
        geometry.path_verbs().len(),
        geometry.polygon_points().len(),
        resources.gradient_stops().len(),
        resources.images().len(),
    ];

    for (kind, count) in SpatialLimitKindV2::DIRECT_ALL.into_iter().zip(observed) {
        validate_direct_count(kind, count, limits)?;
    }

    Ok(DirectCountProof { input, limits })
}

fn validate_direct_count(
    kind: SpatialLimitKindV2,
    observed: usize,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let caller_maximum = limits.limit(kind) as u128;
    let maximum = match kind {
        SpatialLimitKindV2::Nodes
        | SpatialLimitKindV2::Shapes
        | SpatialLimitKindV2::Brushes
        | SpatialLimitKindV2::Clips
        | SpatialLimitKindV2::PaintItems
        | SpatialLimitKindV2::HitItems
        | SpatialLimitKindV2::SemanticItems
        | SpatialLimitKindV2::Paths
        | SpatialLimitKindV2::Images => caller_maximum.min(U32_ROW_CAPACITY),
        SpatialLimitKindV2::PathVerbsTotal
        | SpatialLimitKindV2::PolygonPointsTotal
        | SpatialLimitKindV2::GradientStopsTotal => caller_maximum,
        _ => unreachable!("non-direct spatial limit in direct-count validation"),
    };
    let observed = observed as u128;

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

fn make_resolve_error(
    kind: SpatialResolveErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    SpatialResolveErrorV2::non_limit(kind, location)
}

#[cfg(test)]
mod tests;
