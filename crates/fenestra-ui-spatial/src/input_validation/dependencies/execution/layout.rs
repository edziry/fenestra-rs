use fenestra_ui_layout::prototype::{
    LayoutEngineV1, LayoutErrorKindV1, LayoutErrorLocationV1, LayoutOutputErrorKindV1,
    LayoutOutputFieldV1, PreparedLayoutInputV1, compute_prepared_layout_v1,
};

use super::super::{DependencyGraphProof, DependencyUnitKind};
use super::{
    BasePlacement, IslandExecutionInput, arithmetic, fixed, placement, set_placement, trusted_index,
};
use crate::error::SpatialErrorLocationV2;
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::resolve_error::{
    SpatialLayoutErrorKindV2, SpatialResolveErrorKindV2, SpatialResolveErrorV2,
};

pub(super) fn execute<E: LayoutEngineV1 + ?Sized>(
    input: IslandExecutionInput<'_, '_>,
    engine: &E,
    prepared: PreparedLayoutInputV1,
    placements: &mut [Option<BasePlacement>],
) -> Result<(), SpatialResolveErrorV2> {
    let output = compute_prepared_layout_v1(engine, prepared).map_err(|error| {
        map_layout_execution_error(input.graph, input.index, error.kind(), error.location())
    })?;
    let records = output.records();
    assert_eq!(
        records.len(),
        input.members.len() + 1,
        "prepared output validation retained the island record count"
    );

    let host = placement(placements, input.host);
    let root = records[0].bounds();
    for (actual, expected, field) in [
        (root.x(), 0, LayoutOutputFieldV1::X),
        (root.y(), 0, LayoutOutputFieldV1::Y),
        (root.width(), host.width, LayoutOutputFieldV1::Width),
        (root.height(), host.height, LayoutOutputFieldV1::Height),
    ] {
        if actual != expected {
            return Err(layout_error(
                SpatialLayoutErrorKindV2::SyntheticRootMismatch(field),
                SpatialErrorLocationV2::Island { index: input.index },
            ));
        }
    }

    for (&member, record) in input.members.iter().zip(&records[1..]) {
        let bounds = record.bounds();
        let relative_x = fixed(bounds.x());
        let relative_y = fixed(bounds.y());
        let width = fixed(bounds.width());
        let height = fixed(bounds.height());
        let x = arithmetic(
            host.x.checked_add(relative_x),
            SpatialArithmeticOperationV2::IslandTranslationX,
            member,
        )?;
        let y = arithmetic(
            host.y.checked_add(relative_y),
            SpatialArithmeticOperationV2::IslandTranslationY,
            member,
        )?;
        let far_x = arithmetic(
            x.checked_add(width),
            SpatialArithmeticOperationV2::BaseFarX,
            member,
        )?;
        let far_y = arithmetic(
            y.checked_add(height),
            SpatialArithmeticOperationV2::BaseFarY,
            member,
        )?;
        let parent = input.nodes[trusted_index(member)]
            .parent()
            .expect("phase two validated every island member parent")
            .get();
        let parent = placement(placements, parent);
        let local_x = arithmetic(
            x.checked_sub(parent.x),
            SpatialArithmeticOperationV2::ParentDeltaX,
            member,
        )?;
        let local_y = arithmetic(
            y.checked_sub(parent.y),
            SpatialArithmeticOperationV2::ParentDeltaY,
            member,
        )?;

        set_placement(
            placements,
            member,
            BasePlacement {
                x,
                y,
                width: bounds.width(),
                height: bounds.height(),
                far_x,
                far_y,
                local_x,
                local_y,
            },
        );
    }
    Ok(())
}

pub(in crate::input_validation) fn map_layout_execution_error(
    graph: &DependencyGraphProof<'_>,
    island: u32,
    kind: LayoutErrorKindV1,
    location: LayoutErrorLocationV1,
) -> SpatialResolveErrorV2 {
    let fallback = SpatialErrorLocationV2::Island { index: island };
    match kind {
        LayoutErrorKindV1::Input(_) => bridge_invariant(fallback),
        LayoutErrorKindV1::Engine(kind) => {
            let location = match location {
                LayoutErrorLocationV1::Input
                | LayoutErrorLocationV1::Viewport
                | LayoutErrorLocationV1::Output => fallback,
                LayoutErrorLocationV1::InputNode { index }
                | LayoutErrorLocationV1::OutputRecord { index } => {
                    record_owner(graph, island, index)
                        .map_or(fallback, |index| SpatialErrorLocationV2::Node { index })
                }
            };
            layout_error(SpatialLayoutErrorKindV2::Engine(kind), location)
        }
        LayoutErrorKindV1::Output(kind) => match (kind, location) {
            (LayoutOutputErrorKindV1::RecordCountMismatch, LayoutErrorLocationV1::Output) => {
                layout_error(SpatialLayoutErrorKindV2::Output(kind), fallback)
            }
            (
                LayoutOutputErrorKindV1::KeyMismatch
                | LayoutOutputErrorKindV1::Negative(_)
                | LayoutOutputErrorKindV1::FarEdgeArithmetic(_),
                LayoutErrorLocationV1::OutputRecord { index },
            ) => match record_owner(graph, island, index) {
                Some(index) => layout_error(
                    SpatialLayoutErrorKindV2::Output(kind),
                    SpatialErrorLocationV2::Node { index },
                ),
                None => bridge_invariant(fallback),
            },
            _ => bridge_invariant(fallback),
        },
    }
}

fn record_owner(graph: &DependencyGraphProof<'_>, island: u32, record: u32) -> Option<u32> {
    for unit in &graph.units {
        let DependencyUnitKind::Island { index, host } = unit.kind else {
            continue;
        };
        if index != island {
            continue;
        }
        return if record == 0 {
            Some(host)
        } else {
            unit.produced
                .get(usize::try_from(record).ok()? - 1)
                .copied()
        };
    }
    None
}

fn bridge_invariant(location: SpatialErrorLocationV2) -> SpatialResolveErrorV2 {
    layout_error(SpatialLayoutErrorKindV2::BridgeInvariant, location)
}

fn layout_error(
    kind: SpatialLayoutErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    super::super::super::make_resolve_error(SpatialResolveErrorKindV2::Layout(kind), location)
}
