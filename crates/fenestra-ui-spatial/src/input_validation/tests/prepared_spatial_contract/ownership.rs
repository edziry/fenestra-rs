use std::sync::Arc;

use super::support::{requested_limits, rich_engine, rich_owned};
use super::*;

type SliceIdentity = (*const (), usize);

pub(super) struct SourceIdentities {
    nodes: SliceIdentity,
    polygon_points: SliceIdentity,
    path_verbs: SliceIdentity,
    paths: SliceIdentity,
    shapes: SliceIdentity,
    clips: SliceIdentity,
    gradient_stops: SliceIdentity,
    brushes: SliceIdentity,
    images: SliceIdentity,
    first_image_bytes: SliceIdentity,
    second_image_bytes: SliceIdentity,
    paints: SliceIdentity,
    hits: SliceIdentity,
    semantics: SliceIdentity,
}

#[test]
fn successful_preparation_moves_the_exact_source_arc_into_lifetime_free_state() {
    let source = rich_owned();
    let weak = Arc::downgrade(&source);
    let identities = identities(&source);
    let engine = rich_engine();

    let prepared = prepare_spatial_v2(&engine, source, requested_limits())
        .expect("rich owned input prepares successfully");

    let upgraded = weak.upgrade().expect("prepared value retains the source");
    assert!(Arc::ptr_eq(prepared.source_arc(), &upgraded));
    assert_identities(prepared.source_arc(), &identities);
    assert_eq!(engine.call_count(), 1);

    drop(upgraded);
    drop(prepared);
    assert!(weak.upgrade().is_none());
}

pub(super) fn identities(
    source: &Arc<crate::owned_input::SpatialOwnedInputV2>,
) -> SourceIdentities {
    let input = source.as_input();
    SourceIdentities {
        nodes: identity(input.topology().nodes()),
        polygon_points: identity(input.geometry().polygon_points()),
        path_verbs: identity(input.geometry().path_verbs()),
        paths: identity(input.geometry().paths()),
        shapes: identity(input.geometry().shapes()),
        clips: identity(input.geometry().clips()),
        gradient_stops: identity(input.resources().gradient_stops()),
        brushes: identity(input.resources().brushes()),
        images: identity(input.resources().images()),
        first_image_bytes: identity(input.resources().images()[0].bytes()),
        second_image_bytes: identity(input.resources().images()[1].bytes()),
        paints: identity(input.items().paint_items()),
        hits: identity(input.items().hit_items()),
        semantics: identity(input.items().semantic_items()),
    }
}

pub(super) fn assert_identities(
    source: &Arc<crate::owned_input::SpatialOwnedInputV2>,
    expected: &SourceIdentities,
) {
    let input = source.as_input();
    assert_eq!(identity(input.topology().nodes()), expected.nodes);
    assert_eq!(
        identity(input.geometry().polygon_points()),
        expected.polygon_points
    );
    assert_eq!(identity(input.geometry().path_verbs()), expected.path_verbs);
    assert_eq!(identity(input.geometry().paths()), expected.paths);
    assert_eq!(identity(input.geometry().shapes()), expected.shapes);
    assert_eq!(identity(input.geometry().clips()), expected.clips);
    assert_eq!(
        identity(input.resources().gradient_stops()),
        expected.gradient_stops
    );
    assert_eq!(identity(input.resources().brushes()), expected.brushes);
    assert_eq!(identity(input.resources().images()), expected.images);
    assert_eq!(
        identity(input.resources().images()[0].bytes()),
        expected.first_image_bytes
    );
    assert_eq!(
        identity(input.resources().images()[1].bytes()),
        expected.second_image_bytes
    );
    assert_eq!(identity(input.items().paint_items()), expected.paints);
    assert_eq!(identity(input.items().hit_items()), expected.hits);
    assert_eq!(identity(input.items().semantic_items()), expected.semantics);
}

fn identity<T>(slice: &[T]) -> SliceIdentity {
    (slice.as_ptr().cast(), slice.len())
}
