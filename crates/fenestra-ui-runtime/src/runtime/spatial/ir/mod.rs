mod bindings;
mod checks;
mod counts;
mod geometry;
mod items;
mod live;
mod model;
mod provenance;
mod resources;
mod topology;

use std::sync::Arc;

use fenestra_ui_ir::prototype::ValidatedSpatialProgramV2;
use fenestra_ui_layout::prototype::LayoutEngineV1;
use fenestra_ui_spatial::prototype::{
    SpatialLimitsV2, SpatialOwnedInputV2, SpatialViewportV2, resolve_spatial_v2,
};

use super::build::generated_wrapper_matches;
use super::error::{RuntimeSpatialErrorV2, RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};
use super::types::SpatialPublication;
use super::view::RuntimeSpatialBuildViewV2;
use crate::runtime::state::RuntimeState;

pub(super) fn build_publication(
    program: &ValidatedSpatialProgramV2,
    state: &RuntimeState,
    viewport: SpatialViewportV2,
    limits: SpatialLimitsV2,
    layout_engine: &dyn LayoutEngineV1,
) -> Result<SpatialPublication, RuntimeSpatialErrorV2> {
    let program_span = program.program().span();
    let view = RuntimeSpatialBuildViewV2::new(state);
    let live = live::build_live(program, view).map_err(RuntimeSpatialErrorV2::Ir)?;
    let counts = counts::count_direct(program, &live).map_err(RuntimeSpatialErrorV2::Ir)?;
    counts::preflight_counts(program, counts, limits).map_err(RuntimeSpatialErrorV2::Ir)?;
    checks::check_representation(program, &live, &counts).map_err(RuntimeSpatialErrorV2::Ir)?;

    let topology =
        topology::materialize(program, &live, view).map_err(RuntimeSpatialErrorV2::Ir)?;
    let geometry = geometry::materialize(&live, view).map_err(RuntimeSpatialErrorV2::Ir)?;
    let resources =
        resources::materialize(program, &live, view).map_err(RuntimeSpatialErrorV2::Ir)?;
    let items = items::materialize(&live, view, &geometry, &resources)
        .map_err(RuntimeSpatialErrorV2::Ir)?;

    let logical_nodes = topology.logical_nodes;
    let mut provenance = provenance::Provenance {
        program: program_span,
        nodes: topology.provenance,
        path_verbs: geometry.path_verb_provenance,
        paths: geometry.path_provenance,
        polygon_points: geometry.polygon_point_provenance,
        shapes: geometry.shape_provenance,
        gradient_stops: resources.gradient_stop_provenance,
        brushes: resources.brush_provenance,
        images: resources.image_provenance,
        clips: geometry.clip_provenance,
        paints: items.paint_provenance,
        hits: items.hit_provenance,
        semantics: items.semantic_provenance,
        islands: Vec::new(),
    };
    let source = Arc::new(SpatialOwnedInputV2::new(
        viewport,
        topology.nodes.into_boxed_slice(),
        geometry.polygon_points.into_boxed_slice(),
        geometry.path_verbs.into_boxed_slice(),
        geometry.paths.into_boxed_slice(),
        geometry.shapes.into_boxed_slice(),
        geometry.clips.into_boxed_slice(),
        resources.gradient_stops.into_boxed_slice(),
        resources.brushes.into_boxed_slice(),
        resources.images.into_boxed_slice(),
        items.paints.into_boxed_slice(),
        items.hits.into_boxed_slice(),
        items.semantics.into_boxed_slice(),
    ));
    if !generated_wrapper_matches(state, &source, &logical_nodes, viewport)
        || !provenance.validate(source.as_input())
    {
        return Err(invariant(program_span));
    }

    let snapshot =
        resolve_spatial_v2(layout_engine, Arc::clone(&source), limits).map_err(|error| {
            let Some(span) = provenance.span_for(error.location(), source.as_input()) else {
                return invariant(program_span);
            };
            RuntimeSpatialErrorV2::Ir(RuntimeSpatialIrErrorV2::new(
                RuntimeSpatialIrErrorKindV2::Resolve(error),
                span,
            ))
        })?;
    Ok(SpatialPublication {
        snapshot: Arc::new(snapshot),
        logical_nodes,
    })
}

fn invariant(span: fenestra_ui_ir::prototype::SourceSpan) -> RuntimeSpatialErrorV2 {
    RuntimeSpatialErrorV2::Ir(RuntimeSpatialIrErrorV2::new(
        RuntimeSpatialIrErrorKindV2::InvariantViolation,
        span,
    ))
}
