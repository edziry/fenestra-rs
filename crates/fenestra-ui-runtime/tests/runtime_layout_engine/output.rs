use fenestra_ui_runtime::prototype::{HeadlessRect, NodeId};

use super::headless_projection::{nodes, rect};
use super::layout_support::{DistinctBoundsEngineV1, exact_runtime_with_engine};

#[test]
fn projection_consumes_injected_bounds_for_geometry_clip_hit_and_scene() {
    let runtime = exact_runtime_with_engine(Box::new(DistinctBoundsEngineV1));
    let committed = runtime.committed();
    let fixture = nodes(&committed);
    let projection = committed
        .headless_projection()
        .expect("headless projection should exist");
    let expected = [
        (fixture.root, rect(5, 4, 90, 70)),
        (fixture.container, rect(8, 7, 70, 50)),
        (fixture.control, rect(10, 10, 20, 8)),
        (fixture.first, rect(12, 22, 25, 9)),
        (fixture.second, rect(40, 22, 25, 9)),
    ];

    assert_eq!(
        projection
            .geometries()
            .map(|record| (record.node(), record.bounds(), record.clip()))
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|(node, bounds)| (*node, *bounds, *bounds))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        projection
            .hit_regions()
            .map(|record| (record.node(), record.clip()))
            .collect::<Vec<_>>(),
        expected_records(&expected, &[2, 3, 4])
    );
    assert_eq!(
        projection
            .scene_rectangles()
            .map(|record| (record.node(), record.rectangle()))
            .collect::<Vec<_>>(),
        expected_records(&expected, &[0, 1, 2, 3, 4])
    );
}

fn expected_records(
    records: &[(NodeId, HeadlessRect)],
    indices: &[usize],
) -> Vec<(NodeId, HeadlessRect)> {
    indices.iter().map(|index| records[*index]).collect()
}
