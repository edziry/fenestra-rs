use fenestra_ui_spatial::prototype::{
    ReferenceRasterLimitsV2, SpatialBrushKeyV2, SpatialImageKeyV2, SpatialNodeKeyV2,
    SpatialPaintOutputReferenceV2, SpatialPointV2, SpatialScalarV2, SpatialShapeKeyV2,
};

use crate::new_ir_with_engine;
use crate::spatial_support::engine::{EnginePlan, EngineSpy};
use crate::support::spatial_ir::{
    IMAGE_COLOR, LogicalNodes, STYLED_COLOR, STYLED_WIDTH, VIEWPORT, capacity, fixture, limits,
};

#[test]
fn generation_zero_expands_key_contexts_in_spatial_preorder_with_dense_maps() {
    let fixture = fixture();
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let runtime = new_ir_with_engine(
        fixture.program,
        VIEWPORT,
        limits(),
        capacity(),
        Box::new(engine),
    )
    .expect("validated spatial IR should initialize");
    let committed = runtime.committed();
    let logical = LogicalNodes::capture(&committed);
    let spatial = committed.spatial().expect("spatial state should exist");

    assert_eq!(committed.generation().get(), 0);
    assert_eq!(committed.node_count(), 11);
    assert_eq!(engine_state.calls(), 1);
    assert_eq!(spatial.logical_node(SpatialNodeKeyV2::new(0)), None);
    let preorder = [
        logical.first_outer,
        logical.first_inner,
        logical.second_outer,
        logical.second_inner,
    ];
    for (index, node) in preorder.into_iter().enumerate() {
        let key = SpatialNodeKeyV2::new(u32::try_from(index + 1).expect("fixture key fits"));
        assert_eq!(spatial.logical_node(key), Some(node));
        assert_eq!(spatial.spatial_key(node), Some(key));
    }
    assert_eq!(spatial.spatial_key(logical.root), None);

    let geometry = spatial.snapshot().output().geometry();
    assert_eq!(
        geometry
            .iter()
            .copied()
            .map(|row| (
                row.key().get(),
                logical_scalar(row.base_y()),
                logical_scalar(row.base_width()),
                logical_scalar(row.base_height()),
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 0, 40, 30),
            (1, 0, STYLED_WIDTH, 6),
            (2, 0, 4, 3),
            (3, 6, STYLED_WIDTH, 6),
            (4, 6, 4, 3),
        ]
    );
}

#[test]
fn owner_values_resources_and_item_tables_materialize_without_symbolic_key_leakage() {
    let fixture = fixture();
    let runtime = crate::new_ir(fixture.program, VIEWPORT, limits(), capacity())
        .expect("validated content should initialize");
    let committed = runtime.committed();
    let snapshot = committed
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let output = snapshot.output();

    assert_eq!(
        output
            .clips()
            .iter()
            .copied()
            .map(|row| (row.key().get(), row.owner().get(), row.shape().get()))
            .collect::<Vec<_>>(),
        vec![(0, 1, 0), (1, 3, 1)]
    );
    assert_eq!(
        output
            .paints()
            .iter()
            .copied()
            .map(|row| (row.key(), row.owner().get(), row.item_ordinal()))
            .collect::<Vec<_>>(),
        vec![(0, 1, 0), (1, 1, 1), (2, 3, 0), (3, 3, 1)]
    );
    assert_eq!(
        output.paints()[0].reference(),
        SpatialPaintOutputReferenceV2::Coverage {
            shape: SpatialShapeKeyV2::new(0),
            brush: SpatialBrushKeyV2::new(0),
        }
    );
    assert_eq!(
        output.paints()[1].reference(),
        SpatialPaintOutputReferenceV2::Image {
            image: SpatialImageKeyV2::new(0),
        }
    );
    assert_eq!(
        output.paints()[2].reference(),
        SpatialPaintOutputReferenceV2::Coverage {
            shape: SpatialShapeKeyV2::new(1),
            brush: SpatialBrushKeyV2::new(1),
        }
    );
    assert_eq!(
        output.paints()[3].reference(),
        SpatialPaintOutputReferenceV2::Image {
            image: SpatialImageKeyV2::new(0),
        }
    );
    assert_eq!(
        output
            .hits()
            .iter()
            .copied()
            .map(|row| (
                row.key(),
                row.owner().get(),
                row.shape().get(),
                row.item_ordinal()
            ))
            .collect::<Vec<_>>(),
        vec![(0, 1, 0, 0), (1, 3, 1, 0)]
    );
    assert_eq!(
        output
            .semantics()
            .iter()
            .copied()
            .map(|row| (
                row.key(),
                row.owner().get(),
                row.shape().get(),
                row.item_ordinal()
            ))
            .collect::<Vec<_>>(),
        vec![(0, 1, 0, 0), (1, 3, 1, 0)]
    );

    let raster = snapshot
        .rasterize_reference(ReferenceRasterLimitsV2::new(1_200))
        .expect("bounded fixture raster should resolve");
    assert_eq!(pixel(&raster, 0, 0), IMAGE_COLOR);
    assert_eq!(pixel(&raster, 2, 2), STYLED_COLOR);
    let hit = snapshot
        .hit_test(point(2, 2))
        .expect("bound input policy should accept the first outer node");
    assert_eq!((hit.owner().get(), hit.item_ordinal()), (1, 0));
}

pub(super) fn point(x: i32, y: i32) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

pub(super) fn pixel(
    raster: &fenestra_ui_spatial::prototype::ReferenceRasterV2,
    x: u32,
    y: u32,
) -> [u8; 4] {
    let offset = usize::try_from(u64::from(y) * raster.stride() + u64::from(x) * 4)
        .expect("fixture raster offset should fit");
    raster.bytes()[offset..offset + 4]
        .try_into()
        .expect("one pixel should contain four bytes")
}

fn scalar(value: i32) -> SpatialScalarV2 {
    SpatialScalarV2::new(i64::from(value) * SpatialScalarV2::SCALE)
}

fn logical_scalar(value: SpatialScalarV2) -> i32 {
    i32::try_from(value.raw() / SpatialScalarV2::SCALE).expect("fixture scalar should fit")
}
