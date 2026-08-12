use super::{SpatialResolvedSnapshotV2, materialize::MaterializedTables};
use crate::output_view::SpatialOutputV2;
use crate::resolve_error::SpatialResolveErrorV2;

mod aabbs;
mod clips;
mod common;
mod numeric;
mod projection;
mod references;
mod structure;

pub(super) fn validate(
    mut prepared: super::super::PreparedSpatialV2,
    supplied: SpatialOutputV2<'_>,
) -> Result<SpatialResolvedSnapshotV2, SpatialResolveErrorV2> {
    let state = &prepared.state;
    structure::validate_counts(state, supplied)?;
    structure::validate_keys(supplied)?;
    numeric::validate_scalars(supplied)?;
    numeric::validate_extents(supplied)?;
    numeric::validate_determinants(supplied)?;
    let primitive_clips = aabbs::validate_aabbs(state, supplied)?;
    let effective_clips = clips::validate_clip_chains(state, supplied, &primitive_clips)?;
    projection::validate_projection_order(state, supplied)?;
    references::validate_references(state, supplied)?;

    prepared.state.effective_clip_aabbs = effective_clips
        .into_iter()
        .map(|bounds| bounds.expect("validated clip references make every bound available"))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Ok(snapshot_from_candidate(prepared, supplied))
}

fn snapshot_from_candidate(
    prepared: super::super::PreparedSpatialV2,
    supplied: SpatialOutputV2<'_>,
) -> SpatialResolvedSnapshotV2 {
    let tables = MaterializedTables {
        geometry: supplied.geometry().to_vec().into_boxed_slice(),
        clips: supplied.clips().to_vec().into_boxed_slice(),
        paints: supplied.paints().to_vec().into_boxed_slice(),
        hits: supplied.hits().to_vec().into_boxed_slice(),
        semantics: supplied.semantics().to_vec().into_boxed_slice(),
    };
    SpatialResolvedSnapshotV2 {
        prepared,
        geometry: tables.geometry,
        clips: tables.clips,
        paints: tables.paints,
        hits: tables.hits,
        semantics: tables.semantics,
    }
}
