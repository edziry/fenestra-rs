use std::sync::Arc;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};
use fenestra_ui_spatial::prototype::{
    Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialAnchorV2,
    SpatialContainerV2, SpatialFreePlacementV2, SpatialImageKeyV2, SpatialImageV2,
    SpatialLayoutPlacementV2, SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialNodeV2,
    SpatialOffsetV2, SpatialOwnedInputV2, SpatialPlacementV2, SpatialPointV2, SpatialScalarV2,
    SpatialViewportV2,
};

pub fn canonical_source(viewport: SpatialViewportV2) -> Arc<SpatialOwnedInputV2> {
    owned(
        viewport,
        vec![
            root(0),
            layout(1, 0, 11, 7),
            layout(2, 0, 12, 8),
            layout(3, 0, 13, 9),
            layout(4, 0, 14, 10),
        ],
        true,
    )
}

pub fn three_node_source(viewport: SpatialViewportV2, malformed: bool) -> Arc<SpatialOwnedInputV2> {
    owned(
        viewport,
        vec![
            root(u32::from(malformed) * 91),
            layout(1, 0, 11, 7),
            layout(2, 0, 12, 8),
            layout(3, 0, 13, 9),
        ],
        false,
    )
}

pub fn malformed_source(viewport: SpatialViewportV2) -> Arc<SpatialOwnedInputV2> {
    owned(
        viewport,
        vec![
            root(91),
            layout(1, 0, 11, 7),
            layout(2, 0, 12, 8),
            layout(3, 0, 13, 9),
            layout(4, 0, 14, 10),
        ],
        false,
    )
}

pub fn free_source(viewport: SpatialViewportV2) -> Arc<SpatialOwnedInputV2> {
    let start = SpatialAnchorComponentV2::Start;
    let free = SpatialFreePlacementV2::new(
        13,
        17,
        SpatialAnchorV2::new(start, start),
        SpatialAnchorTargetV2::Viewport,
        SpatialAnchorV2::new(start, start),
        SpatialOffsetV2::new(scalar(5), scalar(6)),
        identity(),
    );
    owned(
        viewport,
        vec![
            root(0),
            SpatialNodeV2::new(
                SpatialNodeKeyV2::new(1),
                Some(SpatialNodeKeyV2::new(0)),
                SpatialPlacementV2::Free(free),
                container(),
            ),
        ],
        false,
    )
}

pub fn layout_source(
    viewport: SpatialViewportV2,
    dimensions: &[(i32, i32)],
) -> Arc<SpatialOwnedInputV2> {
    let mut nodes = Vec::with_capacity(dimensions.len() + 1);
    nodes.push(root(0));
    nodes.extend(
        dimensions
            .iter()
            .enumerate()
            .map(|(index, &(width, height))| {
                layout(
                    u32::try_from(index + 1).expect("fixture key should fit"),
                    0,
                    width,
                    height,
                )
            }),
    );
    owned(viewport, nodes, false)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceIdentity {
    pub owner: usize,
    pub nodes: (usize, usize),
    pub image_bytes: (usize, usize),
}

impl SourceIdentity {
    pub fn capture(source: &Arc<SpatialOwnedInputV2>) -> Self {
        let input = source.as_input();
        let nodes = input.topology().nodes();
        let bytes = input.resources().images()[0].bytes();
        Self {
            owner: Arc::as_ptr(source) as usize,
            nodes: (nodes.as_ptr() as usize, nodes.len()),
            image_bytes: (bytes.as_ptr() as usize, bytes.len()),
        }
    }

    pub fn assert_source(self, source: &Arc<SpatialOwnedInputV2>) {
        let input = source.as_input();
        let nodes = input.topology().nodes();
        let bytes = input.resources().images()[0].bytes();
        assert_eq!(Arc::as_ptr(source) as usize, self.owner);
        assert_eq!((nodes.as_ptr() as usize, nodes.len()), self.nodes);
        assert_eq!((bytes.as_ptr() as usize, bytes.len()), self.image_bytes);
    }
}

fn owned(
    viewport: SpatialViewportV2,
    nodes: Vec<SpatialNodeV2>,
    with_image: bool,
) -> Arc<SpatialOwnedInputV2> {
    let images = if with_image {
        vec![SpatialImageV2::new(
            SpatialImageKeyV2::new(0),
            1,
            1,
            4,
            vec![11, 22, 33, 255].into_boxed_slice(),
        )]
    } else {
        Vec::new()
    };
    Arc::new(SpatialOwnedInputV2::new(
        viewport,
        nodes.into_boxed_slice(),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        images.into_boxed_slice(),
        Box::new([]),
        Box::new([]),
        Box::new([]),
    ))
}

fn root(key: u32) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        None,
        SpatialPlacementV2::Root,
        container(),
    )
}

fn layout(key: u32, parent: u32, width: i32, height: i32) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            fixed(width),
            fixed(height),
            identity(),
        )),
        container(),
    )
}

fn fixed(value: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(value, value, value)
}

fn container() -> SpatialContainerV2 {
    SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0)
}

fn scalar(value: i32) -> SpatialScalarV2 {
    SpatialScalarV2::new(i64::from(value) * SpatialScalarV2::SCALE)
}

fn identity() -> SpatialLocalTransformV2 {
    let zero = SpatialScalarV2::new(0);
    let one = SpatialScalarV2::new(SpatialScalarV2::SCALE);
    SpatialLocalTransformV2::new(
        Affine2V2::new(one, zero, zero, one, zero, zero),
        SpatialPointV2::new(zero, zero),
    )
}
