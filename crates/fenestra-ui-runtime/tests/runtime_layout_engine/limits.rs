use super::support::headless_layout_limits::{
    children_ceiling_parents, depth_ceiling_parents, initialize, node_ceiling_parents,
};

#[test]
fn runtime_reference_accepts_nodes_above_the_registered_probe_limit() {
    let parents = node_ceiling_parents();
    assert_eq!(parents.len(), 33);
    assert_eq!(shape_metrics(&parents), (3, 7));
    assert_geometry_count(&parents);
}

#[test]
fn runtime_reference_accepts_depth_above_the_registered_probe_limit() {
    let parents = depth_ceiling_parents();
    assert_eq!(parents.len(), 9);
    assert_eq!(shape_metrics(&parents), (9, 1));
    assert_geometry_count(&parents);
}

#[test]
fn runtime_reference_accepts_children_above_the_registered_probe_limit() {
    let parents = children_ceiling_parents();
    assert_eq!(parents.len(), 18);
    assert_eq!(shape_metrics(&parents), (2, 17));
    assert_geometry_count(&parents);
}

fn assert_geometry_count(parents: &[Option<usize>]) {
    let runtime = initialize(parents);
    let committed = runtime.committed();
    let projection = committed
        .headless_projection()
        .expect("large headless projection should exist");
    assert_eq!(projection.geometry_count(), parents.len());
    assert_eq!(projection.computed_style_count(), parents.len());
}

fn shape_metrics(parents: &[Option<usize>]) -> (usize, usize) {
    let mut depths = Vec::with_capacity(parents.len());
    let mut child_counts = vec![0usize; parents.len()];
    for parent in parents {
        let depth = match parent {
            Some(parent) => {
                child_counts[*parent] += 1;
                depths[*parent] + 1
            }
            None => 1,
        };
        depths.push(depth);
    }
    (
        depths.into_iter().max().unwrap_or(0),
        child_counts.into_iter().max().unwrap_or(0),
    )
}
