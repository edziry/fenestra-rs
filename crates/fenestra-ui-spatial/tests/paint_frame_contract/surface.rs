use std::collections::BTreeSet;
use std::fs;

use super::source::{all_source, source_dir};
use super::surface_support::{
    assert_const_and_must_use, assert_struct_fields_private, implementation_blocks, names,
    public_constants, public_methods, struct_field_types,
};

const METHODS: [&str; 14] = [
    "viewport",
    "polygon_points",
    "path_verbs",
    "paths",
    "shapes",
    "clip_primitives",
    "gradient_stops",
    "brushes",
    "images",
    "paint_items",
    "resolved_clips",
    "effective_clip_aabbs",
    "resolved_paints",
    "rasterize_reference",
];

#[test]
fn paint_frame_is_the_only_additive_export_and_struct() {
    let source = all_source();
    let lib = fs::read_to_string(source_dir().join("lib.rs")).expect("read spatial lib");
    let observed_exports = prototype_exports(&lib);
    let observed_structs = public_structs(&source);

    assert_eq!(observed_exports.len(), 122);
    assert!(observed_exports.contains("SpatialPaintFrameV2"));
    assert_eq!(observed_structs.len(), 52);
    assert!(observed_structs.contains("SpatialPaintFrameV2"));
    assert_struct_fields_private(&source, "SpatialPaintFrameV2");
    assert!(!struct_field_types(&source, "SpatialPaintFrameV2").is_empty());
}

#[test]
fn paint_frame_and_snapshot_surfaces_are_exact_and_must_use() {
    let source = all_source();
    assert_eq!(
        public_methods(&source, "SpatialPaintFrameV2"),
        names(&METHODS)
    );
    assert!(public_constants(&source, "SpatialPaintFrameV2").is_empty());
    assert_eq!(
        public_methods(&source, "SpatialResolvedSnapshotV2"),
        names(&[
            "viewport",
            "output",
            "effective_clip_aabbs",
            "hit_test",
            "rasterize_reference",
            "paint_frame",
        ])
    );
    assert!(public_constants(&source, "SpatialResolvedSnapshotV2").is_empty());

    assert_const_and_must_use(&source, "SpatialPaintFrameV2", &[], &METHODS);
    assert_const_and_must_use(&source, "SpatialResolvedSnapshotV2", &[], &["paint_frame"]);
}

#[test]
fn paint_frame_has_no_constructor_or_forbidden_projection_vocabulary() {
    let source = all_source();
    let implementations = implementation_blocks(&source, "SpatialPaintFrameV2");
    let joined = implementations.join("\n");
    for forbidden in [
        "pub fn new",
        "pub const fn new",
        "pub fn topology",
        "pub fn nodes",
        "pub fn geometry",
        "pub fn hits",
        "pub fn semantics",
        "pub fn source",
        "pub fn snapshot",
        "pub fn prepared",
        "pub fn layout",
        "pub fn logical",
        "pub fn candidate",
    ] {
        assert!(
            !joined.contains(forbidden),
            "unexpected surface {forbidden}"
        );
    }
}

fn prototype_exports(source: &str) -> BTreeSet<&str> {
    let marker = "pub mod prototype {";
    let start = source.find(marker).expect("prototype module") + marker.len();
    let end = source.rfind('}').expect("prototype end");
    let prototype = &source[start..end];
    let mut exports = BTreeSet::new();
    for item in prototype.split("pub use crate::").skip(1) {
        let names = if let Some(list_start) = item.find("::{") {
            let list_end = item.find("};").expect("terminated grouped reexport");
            &item[list_start + 3..list_end]
        } else {
            let item_end = item.find(';').expect("terminated singleton reexport");
            item[..item_end].rsplit("::").next().expect("export name")
        };
        for name in names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            assert!(exports.insert(name), "duplicate export {name}");
        }
    }
    exports
}

fn public_structs(source: &str) -> BTreeSet<&str> {
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub struct "))
        .map(|declaration| {
            declaration
                .split(['<', '(', '{'])
                .next()
                .expect("struct name")
        })
        .collect()
}
