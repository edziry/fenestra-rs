use std::sync::Arc;

use super::ownership::{assert_identities, identities};
use super::raster_support::*;
use crate::coverage::SpatialFillRuleV2;

#[test]
fn repeated_rasters_are_identical_and_preserve_every_snapshot_allocation() {
    use super::super::validated_shape_support::rect_values;

    let source = owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 1, 1),
        vec![rect_values(0, 1, 0, 0, S, S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(128, 64, 32, 255))],
        Vec::new(),
        vec![
            image(0, 1, 1, vec![0, 0, 0, 0]),
            image(1, 1, 1, vec![1, 1, 1, 1]),
        ],
        Vec::new(),
        vec![fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero)],
    );
    let weak = Arc::downgrade(&source);
    let source_identities = identities(&source);
    let snapshot = snapshot(source);
    let output = snapshot.output();
    let tables = [
        erased(output.geometry()),
        erased(output.clips()),
        erased(output.paints()),
        erased(output.hits()),
        erased(output.semantics()),
    ];
    let effective = erased(snapshot.effective_clip_aabbs());

    let first = snapshot
        .rasterize_reference(limits(1))
        .expect("first raster");
    let second = snapshot
        .rasterize_reference(limits(1))
        .expect("second raster");
    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(first.width(), second.width());
    assert_eq!(first.height(), second.height());
    assert_eq!(first.stride(), second.stride());
    assert_ne!(first.bytes().as_ptr(), second.bytes().as_ptr());

    let output = snapshot.output();
    assert_eq!(
        [
            erased(output.geometry()),
            erased(output.clips()),
            erased(output.paints()),
            erased(output.hits()),
            erased(output.semantics()),
        ],
        tables
    );
    assert_eq!(erased(snapshot.effective_clip_aabbs()), effective);
    assert_identities(snapshot.source_arc(), &source_identities);
    assert!(weak.upgrade().is_some());

    drop(snapshot);
    assert!(weak.upgrade().is_none());
    assert_raster(&first, 1, 1, &[128, 64, 32, 255]);
    assert_raster(&second, 1, 1, &[128, 64, 32, 255]);
}

#[test]
fn image_raster_preserves_the_exact_retained_image_allocation() {
    let source = owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 1, 1),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![image(0, 1, 1, vec![4, 3, 2, 5])],
        Vec::new(),
        vec![image_paint(
            1,
            0,
            0,
            source(0, 0, 1, 1),
            destination(0, 0, S, S),
            255,
            None,
        )],
    );
    let snapshot = snapshot(source);
    let before = identity(
        snapshot
            .finalized_image_paint_bytes(0)
            .expect("retained image bytes"),
    );
    let raster = snapshot
        .rasterize_reference(limits(1))
        .expect("image raster");
    let after = identity(
        snapshot
            .finalized_image_paint_bytes(0)
            .expect("same retained image bytes"),
    );
    assert_eq!(after, before);
    assert_raster(&raster, 1, 1, &[4, 3, 2, 5]);
}

#[test]
fn raster_method_body_does_not_revalidate_or_install_per_query_state() {
    let source = raster_source();
    assert!(source.contains("pub fn rasterize_reference("));
    assert_eq!(source.matches("vec![").count(), 1, "one output allocation");
    for forbidden in [
        "Vec::new(",
        "Vec::with_capacity(",
        "Box::new(",
        "Box::pin(",
        "String::new(",
        "String::with_capacity(",
        "format!(",
        ".collect(",
        "collect::<",
        "HashMap",
        "BTreeMap",
        "VecDeque",
        "BinaryHeap",
        "Rc::new(",
        "Arc::new(",
        "Cell<",
        "RefCell",
        "Mutex",
        "RwLock",
        "OnceLock",
        "LazyLock",
        "validate_rect_k1",
        "validate_circle_k1",
        "validate_polygon_k1",
        "validate_path_k1",
        "validate_stroke_k1",
        "prepare_prepared_brushes",
        "prepare_validated_images",
        "prepare_validated_paint_items",
        "prepare_spatial_v2",
        "resolve_spatial_v2",
        "materialize_reference_spatial_v2",
        "validate_spatial_output_v2",
    ] {
        assert!(
            !source.contains(forbidden),
            "raster query contains {forbidden}"
        );
    }
}

fn erased<T>(slice: &[T]) -> (*const (), usize) {
    (slice.as_ptr().cast(), slice.len())
}

fn identity<T>(slice: &[T]) -> (*const T, usize) {
    (slice.as_ptr(), slice.len())
}

fn raster_source() -> String {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/input_validation/prepared/snapshot");
    let mut source =
        std::fs::read_to_string(root.join("raster.rs")).expect("dedicated reference raster module");
    let directory = root.join("raster");
    if directory.is_dir() {
        let mut paths = std::fs::read_dir(directory)
            .expect("reference raster submodules")
            .map(|entry| entry.expect("raster source entry").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            source.push_str(&std::fs::read_to_string(path).expect("read raster source"));
        }
    }
    source
}
