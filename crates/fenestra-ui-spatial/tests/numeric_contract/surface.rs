use std::collections::BTreeSet;
use std::fs;

use crate::*;

use super::source::{all_source, source_dir};

const EXPECTED_EXPORTS: [&str; 36] = [
    "Affine2V2",
    "REGISTERED_SPATIAL_LIMITS_V2",
    "SpatialAabbV2",
    "SpatialAffineComponentV2",
    "SpatialAnchorComponentV2",
    "SpatialAnchorTargetKindV2",
    "SpatialAnchorTargetV2",
    "SpatialAnchorV2",
    "SpatialArithmeticOperationV2",
    "SpatialAxisV2",
    "SpatialContainerErrorKindV2",
    "SpatialContainerV2",
    "SpatialDependencyErrorKindV2",
    "SpatialErrorLocationV2",
    "SpatialExtentV2",
    "SpatialFreePlacementV2",
    "SpatialInputErrorKindV2",
    "SpatialLayoutDimensionErrorKindV2",
    "SpatialLayoutPlacementV2",
    "SpatialLimitKindV2",
    "SpatialLimitsV2",
    "SpatialLocalTransformV2",
    "SpatialNodeFieldV2",
    "SpatialNodeKeyV2",
    "SpatialNodeV2",
    "SpatialOffsetV2",
    "SpatialPlacementKindV2",
    "SpatialPlacementV2",
    "SpatialPointV2",
    "SpatialScalarV2",
    "SpatialTopologyInputV2",
    "SpatialTransformErrorKindV2",
    "SpatialTransformScalarFieldV2",
    "SpatialTransformStageV2",
    "SpatialViewportV2",
    "round_ratio_v2",
];

#[test]
fn numeric_slice_reexports_exactly_the_registered_surface() {
    let source = fs::read_to_string(source_dir().join("lib.rs")).expect("read lib.rs");
    let marker = "pub mod prototype {";
    let start = source.find(marker).expect("prototype module") + marker.len();
    let end = source.rfind('}').expect("prototype end");
    let prototype = &source[start..end];

    let mut observed = BTreeSet::new();
    for item in prototype.split("pub use crate::").skip(1) {
        let names = if let Some(list_start) = item.find("::{") {
            let list_end = item.find("};").expect("terminated grouped reexport");
            &item[list_start + 3..list_end]
        } else {
            let item_end = item.find(';').expect("terminated singleton reexport");
            item[..item_end]
                .rsplit("::")
                .next()
                .expect("singleton name")
        };
        for name in names.split(',').map(str::trim) {
            if !name.is_empty() {
                assert!(observed.insert(name), "duplicate reexport {name}");
            }
        }
    }

    assert_eq!(observed, EXPECTED_EXPORTS.into_iter().collect());
}

#[test]
fn numeric_methods_remain_const_must_use_and_exact() {
    let source = all_source();
    assert_eq!(
        public_methods(&source, "SpatialScalarV2"),
        names(&[
            "new",
            "raw",
            "is_in_domain",
            "checked_from_i32",
            "checked_add",
            "checked_sub",
            "checked_neg",
            "checked_mul",
            "checked_div",
        ])
    );
    assert_eq!(
        public_methods(&source, "Affine2V2"),
        names(&[
            "new",
            "a",
            "b",
            "c",
            "d",
            "tx",
            "ty",
            "identity",
            "translation",
            "scale",
            "quarter_turn_clockwise",
            "checked_compose",
            "checked_apply_point",
            "determinant_raw",
            "inverse_point",
            "checked_transform_aabb",
        ])
    );
    assert_eq!(
        public_methods(&source, "SpatialAabbV2"),
        names(&[
            "empty",
            "from_edges",
            "is_empty",
            "min_x",
            "min_y",
            "max_x",
            "max_y",
            "intersection",
        ])
    );
    assert_eq!(
        public_constants(&source, "SpatialScalarV2"),
        names(&["FRACTIONAL_BITS", "SCALE", "MIN_RAW", "MAX_RAW"])
    );
    assert!(public_constants(&source, "Affine2V2").is_empty());
    assert!(public_constants(&source, "SpatialAabbV2").is_empty());

    assert_const_and_must_use(
        &source,
        "SpatialScalarV2",
        &[
            "is_in_domain",
            "checked_from_i32",
            "checked_add",
            "checked_sub",
            "checked_neg",
            "checked_mul",
            "checked_div",
        ],
    );
    assert_const_and_must_use(
        &source,
        "Affine2V2",
        &[
            "identity",
            "translation",
            "scale",
            "quarter_turn_clockwise",
            "checked_compose",
            "checked_apply_point",
            "determinant_raw",
            "inverse_point",
            "checked_transform_aabb",
        ],
    );
    assert_const_and_must_use(
        &source,
        "SpatialAabbV2",
        &[
            "empty",
            "from_edges",
            "is_empty",
            "min_x",
            "min_y",
            "max_x",
            "max_y",
            "intersection",
        ],
    );

    let round_ratio = source
        .find("pub const fn round_ratio_v2(")
        .expect("round_ratio_v2");
    assert!(
        has_must_use(&source, round_ratio),
        "round_ratio_v2 must be must_use"
    );
    let scalar_impls = implementation_blocks(&source, "SpatialScalarV2").join("\n");
    for declaration in [
        "pub const FRACTIONAL_BITS: u32",
        "pub const SCALE: i64",
        "pub const MIN_RAW: i64",
        "pub const MAX_RAW: i64",
    ] {
        assert!(scalar_impls.contains(declaration), "missing {declaration}");
    }
}

#[test]
fn numeric_function_signatures_are_exact() {
    let _: fn(i128, i128) -> Option<i128> = round_ratio_v2;
    let _: fn(SpatialScalarV2) -> bool = SpatialScalarV2::is_in_domain;
    let _: fn(i32) -> Option<SpatialScalarV2> = SpatialScalarV2::checked_from_i32;
    let _: fn(SpatialScalarV2, SpatialScalarV2) -> Option<SpatialScalarV2> =
        SpatialScalarV2::checked_add;
    let _: fn(SpatialScalarV2, SpatialScalarV2) -> Option<SpatialScalarV2> =
        SpatialScalarV2::checked_sub;
    let _: fn(SpatialScalarV2) -> Option<SpatialScalarV2> = SpatialScalarV2::checked_neg;
    let _: fn(SpatialScalarV2, SpatialScalarV2) -> Option<SpatialScalarV2> =
        SpatialScalarV2::checked_mul;
    let _: fn(SpatialScalarV2, SpatialScalarV2) -> Option<SpatialScalarV2> =
        SpatialScalarV2::checked_div;

    let _: fn() -> Affine2V2 = Affine2V2::identity;
    let _: fn(SpatialScalarV2, SpatialScalarV2) -> Affine2V2 = Affine2V2::translation;
    let _: fn(SpatialScalarV2, SpatialScalarV2) -> Affine2V2 = Affine2V2::scale;
    let _: fn() -> Affine2V2 = Affine2V2::quarter_turn_clockwise;
    let _: fn(Affine2V2, Affine2V2) -> Result<Affine2V2, SpatialAffineComponentV2> =
        Affine2V2::checked_compose;
    let _: fn(Affine2V2, SpatialPointV2) -> Result<SpatialPointV2, SpatialAxisV2> =
        Affine2V2::checked_apply_point;
    let _: fn(Affine2V2) -> i128 = Affine2V2::determinant_raw;
    let _: fn(Affine2V2, SpatialPointV2) -> Option<SpatialPointV2> = Affine2V2::inverse_point;
    let _: fn(Affine2V2, SpatialAabbV2) -> Result<SpatialAabbV2, SpatialArithmeticOperationV2> =
        Affine2V2::checked_transform_aabb;

    let _: fn() -> SpatialAabbV2 = SpatialAabbV2::empty;
    let _: fn(
        SpatialScalarV2,
        SpatialScalarV2,
        SpatialScalarV2,
        SpatialScalarV2,
    ) -> Option<SpatialAabbV2> = SpatialAabbV2::from_edges;
    let _: fn(SpatialAabbV2) -> bool = SpatialAabbV2::is_empty;
    let _: fn(SpatialAabbV2) -> SpatialScalarV2 = SpatialAabbV2::min_x;
    let _: fn(SpatialAabbV2) -> SpatialScalarV2 = SpatialAabbV2::min_y;
    let _: fn(SpatialAabbV2) -> SpatialScalarV2 = SpatialAabbV2::max_x;
    let _: fn(SpatialAabbV2) -> SpatialScalarV2 = SpatialAabbV2::max_y;
    let _: fn(SpatialAabbV2, SpatialAabbV2) -> SpatialAabbV2 = SpatialAabbV2::intersection;

    let _: u32 = SpatialScalarV2::FRACTIONAL_BITS;
    let _: i64 = SpatialScalarV2::SCALE;
    let _: i64 = SpatialScalarV2::MIN_RAW;
    let _: i64 = SpatialScalarV2::MAX_RAW;
}

#[test]
fn numeric_aabb_fields_remain_private() {
    let source = all_source();
    let marker = "pub struct SpatialAabbV2";
    let start = source.find(marker).expect("SpatialAabbV2 declaration");
    let declaration = &source[start..];
    let brace = declaration.find('{');
    let tuple = declaration.find('(');
    match (brace, tuple) {
        (Some(brace), None) => assert_braced_fields_private(declaration, brace),
        (Some(brace), Some(tuple)) if brace < tuple => {
            assert_braced_fields_private(declaration, brace);
        }
        (_, Some(tuple)) => {
            let end = declaration[tuple..].find(';').expect("tuple AABB end") + tuple;
            let fields = &declaration[tuple + 1..end];
            assert!(!fields.contains("pub ") && !fields.contains("pub("));
        }
        _ => panic!("unsupported SpatialAabbV2 declaration"),
    }
}

fn assert_braced_fields_private(declaration: &str, brace: usize) {
    let end = declaration[brace..].find('}').expect("AABB end") + brace;
    assert!(
        !declaration[brace + 1..end]
            .lines()
            .any(|line| line.trim_start().starts_with("pub")),
        "SpatialAabbV2 fields must remain private"
    );
}

fn public_methods(source: &str, type_name: &str) -> BTreeSet<String> {
    let mut methods = BTreeSet::new();
    for implementation in implementation_blocks(source, type_name) {
        for line in implementation.lines().map(str::trim) {
            if line.starts_with("pub ") && line.contains("fn ") {
                let suffix = line.split_once("fn ").expect("public method").1;
                let name = suffix.split(['(', '<']).next().expect("method name").trim();
                assert!(methods.insert(name.to_owned()), "duplicate method {name}");
            }
        }
    }
    methods
}

fn public_constants(source: &str, type_name: &str) -> BTreeSet<String> {
    let mut constants = BTreeSet::new();
    for implementation in implementation_blocks(source, type_name) {
        for line in implementation.lines().map(str::trim) {
            let Some(suffix) = line.strip_prefix("pub const ") else {
                continue;
            };
            if suffix.starts_with("fn ") {
                continue;
            }
            let name = suffix
                .split([':', '='])
                .next()
                .expect("constant name")
                .trim();
            assert!(
                constants.insert(name.to_owned()),
                "duplicate constant {name}"
            );
        }
    }
    constants
}

fn assert_const_and_must_use(source: &str, type_name: &str, names: &[&str]) {
    let implementations = implementation_blocks(source, type_name);
    for name in names {
        let marker = format!("pub const fn {name}");
        let mut found = false;
        for implementation in &implementations {
            if let Some(offset) = implementation.find(&marker) {
                assert!(
                    has_must_use(implementation, offset),
                    "{type_name}::{name} must be must_use"
                );
                found = true;
            }
        }
        assert!(found, "missing const method {type_name}::{name}");
    }
}

fn has_must_use(source: &str, item_offset: usize) -> bool {
    for line in source[..item_offset].lines().rev() {
        let line = line.trim();
        if line == "#[must_use]" || line.starts_with("#[must_use = ") {
            return true;
        }
        if line.is_empty() || line.starts_with("///") || line.starts_with("#[") {
            continue;
        }
        break;
    }
    false
}

fn implementation_blocks<'a>(source: &'a str, type_name: &str) -> Vec<&'a str> {
    let marker = format!("impl {type_name} {{");
    let mut remaining = source;
    let mut blocks = Vec::new();
    while let Some(start) = remaining.find(&marker) {
        let implementation = &remaining[start..];
        let end = balanced_block_end(implementation);
        blocks.push(&implementation[..end]);
        remaining = &implementation[end..];
    }
    blocks
}

fn balanced_block_end(source: &str) -> usize {
    let mut depth = 0_usize;
    for (offset, character) in source.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return offset + 1;
                }
            }
            _ => {}
        }
    }
    panic!("unterminated impl")
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
