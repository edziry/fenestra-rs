use std::sync::Arc;

use super::super::fixture::RawInputFixture;
use super::super::validated_clip_support::{clip, root_clip};
use super::super::validated_shape_support::rect_values;
use super::super::world_transform_support::{VIEWPORT, free, identity, root};
use super::support::{requested_limits, zero_call_engine};
use super::validator_projection::{set_owner, set_projection};
use super::validator_support::*;
use super::*;
use crate::coverage::SpatialFillRuleV2;
use crate::model::SpatialAnchorTargetV2;
use crate::output_field::{SpatialOutputFieldV2 as Field, SpatialOutputTableV2 as Table};
use crate::resolve_error::SpatialOutputErrorKindV2 as Kind;

#[test]
fn self_and_forward_parents_fail_but_missing_parents_defer_to_references() {
    for (index, parent) in [(0, 0), (0, 1), (1, 2)] {
        let (prepared, mut rows) = rich_case();
        set_parent(&mut rows, index, Some(parent));
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidClipChain,
            output_location(Table::Clip, index as u32, Field::Parent),
        );
    }

    let (prepared, mut rows) = rich_case();
    set_parent(&mut rows, 0, Some(u32::MAX));
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Clip, 0, Field::Parent),
    );

    let (prepared, mut rows) = rich_case();
    set_parent(&mut rows, 0, Some(0));
    set_parent(&mut rows, 1, Some(1));
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidClipChain,
        output_location(Table::Clip, 0, Field::Parent),
    );
}

#[test]
fn complete_clip_pass_checks_later_rows_before_deferred_parent_references() {
    let (prepared, mut rows) = rich_case();
    set_parent(&mut rows, 0, Some(u32::MAX));
    set_parent(&mut rows, 1, Some(1));
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidClipChain,
        output_location(Table::Clip, 1, Field::Parent),
    );

    let (prepared, mut rows) = rich_case();
    set_parent(&mut rows, 0, Some(u32::MAX));
    set_projection(&mut rows, Table::Paint, 0, Field::StackOrdinal, 99);
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidProjectionOrder,
        output_location(Table::Paint, 0, Field::StackOrdinal),
    );
}

#[test]
fn same_and_ancestor_owners_are_valid_but_descendants_and_siblings_are_not() {
    let source = ancestry_owned();
    for child_owner in [1, 2] {
        let (prepared, mut rows) = ancestry_case(&source);
        set_owner(&mut rows, Table::Clip, 1, child_owner);
        if child_owner == 1 {
            expect_output_error(
                validate(prepared, &rows),
                Kind::InvalidReference,
                output_location(Table::Clip, 1, Field::Owner),
            );
        } else {
            validate(prepared, &rows).expect("source owner with ancestor parent is valid");
        }
    }

    for (parent_owner, child_owner) in [(2, 1), (3, 2)] {
        let (prepared, mut rows) = ancestry_case(&source);
        set_owner(&mut rows, Table::Clip, 0, parent_owner);
        set_owner(&mut rows, Table::Clip, 1, child_owner);
        expect_output_error(
            validate(prepared, &rows),
            Kind::InvalidClipChain,
            output_location(Table::Clip, 1, Field::Parent),
        );
    }
}

#[test]
fn canonical_empty_effective_parent_remains_available_for_ancestry() {
    let (prepared, mut rows) = rich_case();
    let mut clip = ClipRow::read(rows.clips[1]);
    clip.world = [S, 0, 0, S, 100 * S, 100 * S];
    clip.determinant = D;
    clip.aabb = (false, [100 * S, 100 * S, 110 * S, 110 * S]);
    clip.owner = 3;
    rows.clips[1] = clip.build();
    set_owner(&mut rows, Table::Clip, 2, 1);
    set_parent(&mut rows, 2, Some(1));
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidClipChain,
        output_location(Table::Clip, 2, Field::Parent),
    );
}

#[test]
fn invalid_owner_keeps_primitive_available_for_descendant_chain_reasoning() {
    let (prepared, mut rows) = rich_case();
    set_owner(&mut rows, Table::Clip, 0, u32::MAX);
    set_owner(&mut rows, Table::Clip, 1, 3);
    set_parent(&mut rows, 1, Some(0));
    set_owner(&mut rows, Table::Clip, 2, 1);
    set_parent(&mut rows, 2, Some(1));
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidClipChain,
        output_location(Table::Clip, 2, Field::Parent),
    );
}

#[test]
fn invalid_current_owner_with_valid_parent_defers_without_ancestry_indexing() {
    let (prepared, mut rows) = rich_case();
    set_owner(&mut rows, Table::Clip, 1, u32::MAX);
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Clip, 1, Field::Owner),
    );
}

#[test]
fn unavailable_parent_propagates_without_panicking_or_running_ancestry() {
    let (prepared, mut rows) = rich_case();
    set_parent(&mut rows, 0, Some(u32::MAX));
    set_owner(&mut rows, Table::Clip, 1, 3);
    set_parent(&mut rows, 1, Some(0));
    set_owner(&mut rows, Table::Clip, 2, 1);
    set_parent(&mut rows, 2, Some(1));
    expect_output_error(
        validate(prepared, &rows),
        Kind::InvalidReference,
        output_location(Table::Clip, 0, Field::Parent),
    );
}

pub(super) fn set_parent(rows: &mut CandidateTables, index: usize, parent: Option<u32>) {
    let mut row = ClipRow::read(rows.clips[index]);
    row.parent = parent;
    rows.clips[index] = row.build();
}

fn ancestry_case(
    source: &Arc<crate::owned_input::SpatialOwnedInputV2>,
) -> (PreparedSpatialV2, CandidateTables) {
    let prepared =
        prepare_spatial_v2(&zero_call_engine(), source.clone(), requested_limits()).unwrap();
    let reference = materialize_reference_spatial_v2(
        prepare_spatial_v2(&zero_call_engine(), source.clone(), requested_limits()).unwrap(),
    );
    (prepared, CandidateTables::from_snapshot(&reference))
}

fn ancestry_owned() -> Arc<crate::owned_input::SpatialOwnedInputV2> {
    let nodes = vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            10,
            10,
            identity(),
        ),
        free(
            2,
            1,
            SpatialAnchorTargetV2::Parent,
            0,
            0,
            10,
            10,
            identity(),
        ),
        free(
            3,
            1,
            SpatialAnchorTargetV2::Parent,
            0,
            0,
            10,
            10,
            identity(),
        ),
    ];
    Arc::new(
        RawInputFixture::with_nodes(nodes)
            .with_paths(Vec::new(), Vec::new())
            .with_shapes(
                vec![rect_values(0, 1, 0, 0, S, S), rect_values(1, 2, 0, 0, S, S)],
                Vec::new(),
            )
            .with_brushes(Vec::new(), Vec::new())
            .with_images(Vec::new())
            .with_clips(vec![
                root_clip(0, 1, 0),
                clip(1, 2, Some(0), 1, SpatialFillRuleV2::NonZero),
            ])
            .with_paint_items(Vec::new())
            .with_hit_items(Vec::new())
            .with_semantic_items(Vec::new())
            .into_owned(VIEWPORT),
    )
}
