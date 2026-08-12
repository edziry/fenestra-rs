use super::super::super::model::PreparedSpatialState;
use super::common::{ordinal, output_error};
use crate::aabb::SpatialAabbV2;
use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
use crate::output_view::SpatialOutputV2;
use crate::resolve_error::{SpatialOutputErrorKindV2, SpatialResolveErrorV2};

pub(super) fn validate_clip_chains(
    state: &PreparedSpatialState,
    supplied: SpatialOutputV2<'_>,
    primitives: &[SpatialAabbV2],
) -> Result<Vec<Option<SpatialAabbV2>>, SpatialResolveErrorV2> {
    let rows = supplied.clips();
    let mut effective = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().copied().enumerate() {
        let current = match row.parent() {
            None => Some(primitives[index]),
            Some(parent) => resolve_parent(
                state,
                rows,
                &effective,
                primitives[index],
                index,
                parent.get(),
            )?,
        };
        effective.push(current);
    }
    Ok(effective)
}

fn resolve_parent(
    state: &PreparedSpatialState,
    rows: &[crate::output_geometry::SpatialClipOutputRecordV2],
    effective: &[Option<SpatialAabbV2>],
    primitive: SpatialAabbV2,
    index: usize,
    parent: u32,
) -> Result<Option<SpatialAabbV2>, SpatialResolveErrorV2> {
    let Ok(parent_index) = usize::try_from(parent) else {
        return Ok(None);
    };
    if parent_index >= rows.len() {
        return Ok(None);
    }
    if parent_index >= index {
        return Err(clip_error(index));
    }
    let Some(parent_bounds) = effective[parent_index] else {
        return Ok(None);
    };
    let parent_owner = rows[parent_index].owner().get();
    let current_owner = rows[index].owner().get();
    if has_valid_owner(state, parent_owner)
        && has_valid_owner(state, current_owner)
        && !is_same_or_ancestor(state, parent_owner, current_owner)
    {
        return Err(clip_error(index));
    }
    Ok(Some(primitive.intersection(parent_bounds)))
}

fn has_valid_owner(state: &PreparedSpatialState, owner: u32) -> bool {
    usize::try_from(owner)
        .ok()
        .and_then(|index| state.topology.get(index))
        .is_some()
}

fn is_same_or_ancestor(state: &PreparedSpatialState, ancestor: u32, mut node: u32) -> bool {
    loop {
        if node == ancestor {
            return true;
        }
        let Some(current) = usize::try_from(node)
            .ok()
            .and_then(|index| state.topology.get(index))
        else {
            return false;
        };
        let Some(parent) = current.parent else {
            return false;
        };
        node = parent;
    }
}

fn clip_error(index: usize) -> SpatialResolveErrorV2 {
    output_error(
        SpatialOutputErrorKindV2::InvalidClipChain,
        SpatialOutputTableV2::Clip,
        ordinal(index),
        SpatialOutputFieldV2::Parent,
    )
}
