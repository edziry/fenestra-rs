use fenestra_ui_runtime::prototype::{RuntimeInitializationError, RuntimeInitializationErrorKind};
use fenestra_ui_spatial::prototype::{
    SpatialContentErrorKindV2, SpatialErrorLocationV2, SpatialGradientErrorV2,
    SpatialGradientStopFieldV2, SpatialNodeKeyV2, SpatialPathGrammarErrorV2,
    SpatialPathVerbFieldV2, SpatialResolveErrorKindV2,
};

use crate::support::spatial_ir::{
    MapperFault, OUTER_REGION, VIEWPORT, capacity, inner, limits, mapper_fixture,
};
use crate::{RuntimeSpatialErrorV2, RuntimeSpatialIrErrorKindV2, new_ir};

#[test]
fn mixed_placement_anchor_geometry_brush_and_coverage_branches_resolve_together() {
    let fixture = mapper_fixture(MapperFault::None);
    let runtime = new_ir(fixture.program, VIEWPORT, limits(), capacity())
        .expect("complete mixed mapper fixture should initialize");
    let committed = runtime.committed();
    let spatial = committed.spatial().expect("spatial state should exist");
    let root = committed.root();
    let outer_fragment = committed
        .fragment(root, OUTER_REGION)
        .expect("outer region should be live");
    let outer_nodes = committed
        .keyed_members(outer_fragment)
        .expect("outer members should be live")
        .map(|(_, node)| node)
        .collect::<Vec<_>>();
    let mut expected = Vec::new();
    for outer in outer_nodes {
        let nested = inner(&committed, outer);
        let leaf = committed
            .children(nested)
            .expect("nested member should own the layout-under-free node")[0];
        let anchors = committed
            .children(leaf)
            .expect("layout node should own both anchor variants");
        expected.extend([outer, nested, leaf, anchors[0], anchors[1]]);
    }

    assert_eq!(spatial.snapshot().output().geometry().len(), 11);
    for (index, node) in expected.into_iter().enumerate() {
        let key = SpatialNodeKeyV2::new(u32::try_from(index + 1).expect("fixture key fits"));
        assert_eq!(spatial.logical_node(key), Some(node));
        assert_eq!(spatial.spatial_key(node), Some(key));
    }
    let output = spatial.snapshot().output();
    assert_eq!(output.clips().len(), 4);
    assert_eq!(output.paints().len(), 6);
    assert_eq!(output.hits().len(), 2);
    assert_eq!(output.semantics().len(), 2);
    let transformed_leaf = output.geometry()[3];
    assert_eq!(transformed_leaf.world_from_local().a().raw(), 0);
    assert_eq!(
        transformed_leaf.world_from_local().b().raw(),
        fenestra_ui_spatial::prototype::SpatialScalarV2::SCALE
    );
    assert_eq!(
        transformed_leaf.world_from_local().c().raw(),
        -fenestra_ui_spatial::prototype::SpatialScalarV2::SCALE
    );
    assert_eq!(transformed_leaf.world_from_local().d().raw(), 0);
    assert_eq!(
        output
            .clips()
            .iter()
            .copied()
            .map(|clip| (
                clip.key().get(),
                clip.owner().get(),
                clip.parent().map(|key| key.get())
            ))
            .collect::<Vec<_>>(),
        vec![(0, 1, None), (1, 1, Some(0)), (2, 6, None), (3, 6, Some(2))]
    );
    assert_eq!(
        output
            .paints()
            .iter()
            .copied()
            .map(|paint| (paint.owner().get(), paint.item_ordinal()))
            .collect::<Vec<_>>(),
        vec![(1, 0), (1, 1), (1, 2), (6, 0), (6, 1), (6, 2)]
    );
}

#[test]
fn nested_path_verb_failure_maps_to_the_exact_authored_verb_record() {
    let fixture = mapper_fixture(MapperFault::PathVerb);
    let expected_span = fixture.spans.path_first_verb;
    let ir = expect_ir(initialization_error(new_ir(
        fixture.program,
        VIEWPORT,
        limits(),
        capacity(),
    )));
    let RuntimeSpatialIrErrorKindV2::Resolve(resolve) = ir.kind() else {
        panic!("path grammar failure should retain resolver detail");
    };

    assert_eq!(ir.span(), expected_span);
    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::InvalidPathGrammar(
            SpatialPathGrammarErrorV2::FirstNotMove,
        ))
    );
    assert_eq!(
        resolve.location(),
        SpatialErrorLocationV2::PathVerb {
            path: 0,
            verb: 0,
            field: SpatialPathVerbFieldV2::Kind,
        }
    );
}

#[test]
fn nested_gradient_stop_failure_maps_to_the_exact_offset_leaf() {
    let fixture = mapper_fixture(MapperFault::GradientStop);
    let expected_span = fixture.spans.gradient_last_offset;
    let ir = expect_ir(initialization_error(new_ir(
        fixture.program,
        VIEWPORT,
        limits(),
        capacity(),
    )));
    let RuntimeSpatialIrErrorKindV2::Resolve(resolve) = ir.kind() else {
        panic!("gradient failure should retain resolver detail");
    };

    assert_eq!(ir.span(), expected_span);
    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::InvalidGradient(
            SpatialGradientErrorV2::LastOffset,
        ))
    );
    assert_eq!(
        resolve.location(),
        SpatialErrorLocationV2::GradientStop {
            brush: 1,
            stop: 1,
            field: SpatialGradientStopFieldV2::Offset,
        }
    );
}

fn initialization_error(
    result: Result<fenestra_ui_runtime::prototype::UiRuntime, RuntimeInitializationError>,
) -> RuntimeInitializationError {
    match result {
        Ok(_) => panic!("faulted mapper fixture should fail"),
        Err(error) => error,
    }
}

fn expect_ir(error: RuntimeInitializationError) -> crate::RuntimeSpatialIrErrorV2 {
    let RuntimeInitializationErrorKind::Spatial(RuntimeSpatialErrorV2::Ir(ir)) = error.kind()
    else {
        panic!("mapper failures should use the IR error lane");
    };
    ir
}
