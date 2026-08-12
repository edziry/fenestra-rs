use super::resources::{
    add_basic_content_v2, add_clipped_content_v2, add_path_content_v2, add_rich_content_v2,
    add_split_content_v2, complete_direct_queries_v2,
};
use crate::baseline::literal_types::{
    AffineV2, AnchorComponentV2 as A, AnchorTargetV2 as T, AxisV2, FIXED_ONE_V2,
    LiteralObservationInputV2, NodeInputV2, PlacementInputV2, PointV2, ReceiptInputV2,
    SceneInputV2,
};

pub(super) fn build_direct_observations_v2(
    ordinal: usize,
    count: usize,
) -> Vec<LiteralObservationInputV2> {
    (0..count)
        .map(|step| LiteralObservationInputV2 {
            step: u8::try_from(step).expect("direct step should fit"),
            scene: build_scene_v2(ordinal, step),
        })
        .collect()
}

fn build_scene_v2(ordinal: usize, step: usize) -> SceneInputV2 {
    let viewport = if step == 0 { (192, 128) } else { (208, 136) };
    let mut scene = empty_scene(viewport);
    scene.nodes = nodes_v2(ordinal, step);
    match ordinal {
        5 => add_basic_content_v2(&mut scene, 2, 24, 12),
        6 => add_split_content_v2(&mut scene, 1),
        7 => add_clipped_content_v2(&mut scene, 3),
        8 => add_path_content_v2(&mut scene, 2),
        9 => add_rich_content_v2(&mut scene),
        11 => {
            add_basic_content_v2(&mut scene, 1, 0, 19);
            add_basic_content_v2(&mut scene, 2, 23, 0);
            add_basic_content_v2(&mut scene, 3, 0, 0);
        }
        _ => {
            let owner = u32::try_from(scene.nodes.len() - 1).expect("node owner should fit");
            add_basic_content_v2(&mut scene, owner, 18, 14);
        }
    }
    complete_direct_queries_v2(&mut scene);
    scene
}

fn empty_scene(viewport: (u32, u32)) -> SceneInputV2 {
    SceneInputV2 {
        viewport,
        receipt: ReceiptInputV2 {
            generation: None,
            mutation_count: 0,
            invalidation: 0,
        },
        nodes: Vec::new(),
        paths: Vec::new(),
        shapes: Vec::new(),
        clips: Vec::new(),
        brushes: Vec::new(),
        images: Vec::new(),
        paints: Vec::new(),
        hits: Vec::new(),
        semantics: Vec::new(),
        queries: Vec::new(),
    }
}

fn nodes_v2(ordinal: usize, step: usize) -> Vec<NodeInputV2> {
    let root = root();
    match ordinal {
        0 => vec![
            root,
            layout(1, 0, 72, 48),
            layout(2, 1, 44, 32),
            layout(3, 2, 18, 14),
        ],
        1 => vec![
            root,
            free(1, 0, 64, 44, 8, 6),
            free(2, 1, 38, 26, 5, 4),
            free(3, 2, 18, 14, 3, 2),
        ],
        2 => {
            let host = if step == 0 { (80, 56) } else { (88, 60) };
            vec![
                root,
                free(1, 0, host.0, host.1, 12, 8),
                layout(2, 1, 42, 28),
                layout(3, 2, 18, 14),
            ]
        }
        3 => {
            let host = if step == 0 { (52, 36) } else { (58, 40) };
            vec![
                root,
                layout(1, 0, host.0, host.1),
                free(2, 1, 30, 22, 5, 7),
                layout(3, 2, 18, 14),
            ]
        }
        4 => vec![root, layout(1, 0, 40, 20), free(2, 0, 41, 18, 41, 3)],
        5 => vec![root, layout(1, 0, 64, 40), layout(2, 1, 24, 12)],
        6 => vec![root, layout(1, 0, 48, 32)],
        7 => transformed_nodes(),
        8 => vec![root, free(1, 0, 52, 44, 6, 5), free(2, 1, 34, 30, 5, 4)],
        9 => vec![
            root,
            free(1, 0, 36, 28, 8, 8),
            free(2, 0, 36, 28, 20, 14),
            free(3, 0, 24, 20, 34, 22),
        ],
        10 => anchor_nodes(),
        11 => zero_extent_nodes(root),
        _ => unreachable!("only direct case ordinals are registered"),
    }
}

fn root() -> NodeInputV2 {
    NodeInputV2 {
        key: 0,
        path: None,
        parent: None,
        placement: PlacementInputV2::Root,
        axis: AxisV2::Horizontal,
        padding: [2, 2, 2, 2],
        gap: 2,
    }
}

fn layout(key: u32, parent: u32, width: i32, height: i32) -> NodeInputV2 {
    node(
        key,
        parent,
        PlacementInputV2::Layout {
            width,
            height,
            transform: AffineV2::IDENTITY,
        },
    )
}

fn free(key: u32, parent: u32, width: i32, height: i32, x: i32, y: i32) -> NodeInputV2 {
    node(
        key,
        parent,
        PlacementInputV2::Free {
            width,
            height,
            self_anchor: [A::Start, A::Start],
            target: T::Parent,
            target_anchor: [A::Start, A::Start],
            offset: point(x, y),
            transform: AffineV2::IDENTITY,
        },
    )
}

fn node(key: u32, parent: u32, placement: PlacementInputV2) -> NodeInputV2 {
    NodeInputV2 {
        key,
        path: None,
        parent: Some(parent),
        placement,
        axis: AxisV2::Vertical,
        padding: [1, 1, 1, 1],
        gap: 1,
    }
}

fn transformed_nodes() -> Vec<NodeInputV2> {
    let mut first = free(1, 0, 54, 46, 18, 14);
    let PlacementInputV2::Free { transform, .. } = &mut first.placement else {
        unreachable!()
    };
    transform.values[4] = 8 * FIXED_ONE_V2;
    transform.values[5] = 6 * FIXED_ONE_V2;
    let mut second = free(2, 1, 36, 30, 8, 6);
    let PlacementInputV2::Free { transform, .. } = &mut second.placement else {
        unreachable!()
    };
    transform.values = [0, FIXED_ONE_V2, -FIXED_ONE_V2, 0, 0, 0];
    let mut third = free(3, 2, 22, 18, 6, 5);
    let PlacementInputV2::Free { transform, .. } = &mut third.placement else {
        unreachable!()
    };
    transform.values = [2 * FIXED_ONE_V2, 0, 0, FIXED_ONE_V2, 0, 0];
    vec![root(), first, second, third]
}

fn anchor_nodes() -> Vec<NodeInputV2> {
    let mut later = free(1, 0, 20, 16, 9, -3);
    let PlacementInputV2::Free {
        target,
        target_anchor,
        ..
    } = &mut later.placement
    else {
        unreachable!()
    };
    *target = T::Node(3);
    *target_anchor = [A::Center, A::Center];
    let parent = free(2, 0, 18, 14, 6, 5);
    let mut viewport = free(3, 0, 28, 20, 30, 18);
    let PlacementInputV2::Free {
        target,
        target_anchor,
        ..
    } = &mut viewport.placement
    else {
        unreachable!()
    };
    *target = T::Viewport;
    *target_anchor = [A::End, A::End];
    vec![root(), later, parent, viewport]
}

fn zero_extent_nodes(root: NodeInputV2) -> Vec<NodeInputV2> {
    let mut zero_by_n = free(1, 0, 0, 19, 4, 4);
    zero_by_n.padding[0] = 0;
    zero_by_n.padding[1] = 0;
    let mut n_by_zero = free(2, 0, 23, 0, 12, 8);
    n_by_zero.padding[2] = 0;
    n_by_zero.padding[3] = 0;
    let mut zero_by_zero = free(3, 0, 0, 0, 20, 12);
    zero_by_zero.padding = [0; 4];
    vec![root, zero_by_n, n_by_zero, zero_by_zero]
}

fn point(x: i32, y: i32) -> PointV2 {
    PointV2 {
        x: i64::from(x) * FIXED_ONE_V2,
        y: i64::from(y) * FIXED_ONE_V2,
    }
}
