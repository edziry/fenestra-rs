use std::collections::BTreeSet;

use super::source::all_source;

#[test]
fn runtime_spatial_values_have_exact_public_methods() {
    let source = all_source();
    assert_eq!(
        public_methods(&source, "RuntimeSpatialBuildViewV2"),
        names(&[
            "children",
            "component",
            "fragment",
            "keyed_member",
            "keyed_members",
            "node_count",
            "parent",
            "property",
            "root",
            "template",
        ])
    );
    assert_eq!(
        public_methods(&source, "RuntimeSpatialInputV2"),
        names(&["new"])
    );
    assert_eq!(
        public_methods(&source, "RuntimeSpatialViewV2"),
        names(&["logical_node", "snapshot", "spatial_key"])
    );
    assert_eq!(
        public_methods(&source, "SpatialViewportChangeViewV2"),
        names(&["new_viewport", "old_viewport"])
    );
    assert!(public_methods(&source, "RuntimeSpatialErrorV2").is_empty());

    for type_name in [
        "RuntimeSpatialBuildViewV2",
        "RuntimeSpatialInputV2",
        "RuntimeSpatialViewV2",
        "RuntimeSpatialErrorV2",
        "SpatialViewportChangeViewV2",
    ] {
        assert!(public_constants(&source, type_name).is_empty());
    }
}

#[test]
fn runtime_spatial_cut_preserves_exact_owner_method_sets() {
    let source = all_source();
    assert_eq!(
        public_methods(&source, "UiRuntime"),
        names(&[
            "begin_transaction",
            "commit",
            "committed",
            "new",
            "new_headless",
            "new_headless_with_layout_engine",
            "new_spatial",
            "new_spatial_with_layout_engine",
        ])
    );
    assert_eq!(
        public_methods(&source, "UiTransaction"),
        names(&[
            "insert_keyed",
            "move_keyed",
            "remove_keyed",
            "resize_headless",
            "resize_spatial",
            "set_property",
            "update_keyed",
        ])
    );
    assert_eq!(
        public_methods(&source, "CommittedRuntimeSnapshot"),
        names(&[
            "children",
            "component",
            "fragment",
            "fragment_count",
            "generation",
            "headless_projection",
            "keyed_member",
            "keyed_members",
            "node_count",
            "parent",
            "property",
            "property_slot_count",
            "root",
            "shares_state_with",
            "spatial",
            "template",
        ])
    );
}

#[test]
fn runtime_spatial_methods_have_exact_attributes() {
    let source = all_source();
    assert_methods_are_must_use(
        &source,
        "RuntimeSpatialBuildViewV2",
        &[
            "root",
            "node_count",
            "template",
            "component",
            "property",
            "parent",
            "children",
            "fragment",
            "keyed_members",
            "keyed_member",
        ],
    );
    assert_methods_are_must_use(&source, "RuntimeSpatialInputV2", &["new"]);
    assert_methods_are_must_use(
        &source,
        "RuntimeSpatialViewV2",
        &["snapshot", "logical_node", "spatial_key"],
    );
    assert_methods_are_must_use(
        &source,
        "SpatialViewportChangeViewV2",
        &["old_viewport", "new_viewport"],
    );
    assert_const_methods(
        &source,
        "SpatialViewportChangeViewV2",
        &["old_viewport", "new_viewport"],
    );
    for (type_name, methods) in [
        (
            "RuntimeSpatialBuildViewV2",
            &[
                "root",
                "node_count",
                "template",
                "component",
                "property",
                "parent",
                "children",
                "fragment",
                "keyed_members",
                "keyed_member",
            ][..],
        ),
        ("RuntimeSpatialInputV2", &["new"][..]),
        (
            "RuntimeSpatialViewV2",
            &["snapshot", "logical_node", "spatial_key"][..],
        ),
    ] {
        assert_nonconst_methods(&source, type_name, methods);
    }
}

#[test]
fn runtime_spatial_program_has_one_exact_method_and_bound_set() {
    let source = significant(&all_source());
    assert!(source.contains(concat!(
        "pubtraitRuntimeSpatialProgramV2:",
        "Send+Sync+Unpin+UnwindSafe+RefUnwindSafe+'static{",
        "#[must_use]fnbuild(&self,runtime:RuntimeSpatialBuildViewV2<'_>,",
        "viewport:SpatialViewportV2,)->RuntimeSpatialInputV2;}"
    )));
    let traits = public_names(&source, "pubtrait");
    assert_eq!(traits, names(&["RuntimeSpatialProgramV2"]));
}

#[test]
fn runtime_spatial_closed_enums_have_exact_variants_and_order() {
    let source = significant(&all_source());
    assert!(source.contains(concat!(
        "pubenumRuntimeSpatialErrorV2{",
        "ViewportMismatch,MappingLengthMismatch,",
        "MissingLogicalNode{key:SpatialNodeKeyV2,},",
        "DuplicateLogicalNode{key:SpatialNodeKeyV2,},",
        "Resolve(SpatialResolveErrorV2),}"
    )));
    assert!(source.contains(concat!(
        "pubenumRuntimeInitializationErrorKind{",
        "CapacityExceeded(CapacityKind),Headless(HeadlessProjectionErrorKind),",
        "Spatial(RuntimeSpatialErrorV2),InvariantViolation,}"
    )));
    assert!(source.contains(concat!(
        "pubenumTransactionErrorKind{",
        "CapacityExceeded(CapacityKind),Headless(HeadlessProjectionErrorKind),",
        "Spatial(RuntimeSpatialErrorV2),HeadlessUnavailable,SpatialUnavailable,",
        "StaleBase,MissingNode,MissingFragment,MissingKey,DuplicateKey,",
        "UnknownProperty,PropertyTypeMismatch,IndexOutOfBounds,",
        "GenerationExhausted,InvariantViolation,}"
    )));
    assert!(source.contains(concat!(
        "pubenumMutationRecordView<'a>{",
        "PropertyChanged(PropertyChangeView<'a>),KeyInserted(KeyInsertView<'a>),",
        "KeyMoved(KeyMoveView<'a>),KeyRemoved(KeyRemoveView<'a>),",
        "HeadlessSurfaceChanged(HeadlessSurfaceChangeView<'a>),",
        "SpatialViewportChanged(SpatialViewportChangeViewV2<'a>),}"
    )));
}

#[test]
fn runtime_spatial_owner_methods_have_exact_attributes() {
    let source = all_source();
    assert_eq!(method_attributes(&source, "new_spatial"), BTreeSet::new());
    assert_eq!(
        method_attributes(&source, "new_spatial_with_layout_engine"),
        names(&["#[doc(hidden)]"])
    );
    assert_eq!(
        method_attributes(&source, "resize_spatial"),
        BTreeSet::new()
    );
    assert_eq!(
        method_attributes(&source, "spatial"),
        names(&["#[must_use]"])
    );
    for name in [
        "new_spatial",
        "new_spatial_with_layout_engine",
        "resize_spatial",
        "spatial",
    ] {
        assert!(!source.contains(&format!("pub const fn {name}(")));
    }
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
            if !suffix.starts_with("fn ") {
                constants.insert(
                    suffix
                        .split([':', '='])
                        .next()
                        .expect("constant name")
                        .trim()
                        .to_owned(),
                );
            }
        }
    }
    constants
}

fn assert_methods_are_must_use(source: &str, type_name: &str, names: &[&str]) {
    let implementations = implementation_blocks(source, type_name);
    for name in names {
        let marker = format!("fn {name}(");
        let (implementation, offset) = implementations
            .iter()
            .find_map(|implementation| {
                implementation
                    .find(&marker)
                    .map(|offset| (*implementation, offset))
            })
            .unwrap_or_else(|| panic!("missing method {type_name}::{name}"));
        let line_start = implementation[..offset]
            .rfind('\n')
            .map_or(0, |newline| newline + 1);
        assert!(
            has_must_use(implementation, line_start),
            "{type_name}::{name}"
        );
    }
}

fn assert_const_methods(source: &str, type_name: &str, names: &[&str]) {
    let implementations = implementation_blocks(source, type_name);
    for name in names {
        let marker = format!("pub const fn {name}");
        assert!(implementations.iter().any(|block| block.contains(&marker)));
    }
}

fn assert_nonconst_methods(source: &str, type_name: &str, names: &[&str]) {
    let implementations = implementation_blocks(source, type_name);
    for name in names {
        let marker = format!("pub const fn {name}");
        assert!(!implementations.iter().any(|block| block.contains(&marker)));
    }
}

fn implementation_blocks<'a>(source: &'a str, type_name: &str) -> Vec<&'a str> {
    let mut remaining = source;
    let mut blocks = Vec::new();
    loop {
        let start = remaining
            .split_inclusive('\n')
            .scan(0, |offset, line| {
                let start = *offset;
                *offset += line.len();
                Some((start, line.trim_start()))
            })
            .find_map(|(offset, line)| {
                (line.starts_with("impl") && line.contains(type_name) && !line.contains(" for "))
                    .then_some(
                        offset + remaining[offset..].len() - remaining[offset..].trim_start().len(),
                    )
            });
        let Some(start) = start else { break };
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

fn has_must_use(source: &str, item_offset: usize) -> bool {
    source[..item_offset]
        .lines()
        .rev()
        .find(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with("///")
        })
        .is_some_and(|line| line.trim().starts_with("#[must_use"))
}

fn significant(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with("///") && !line.starts_with("#[doc =")
        })
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn public_names(source: &str, marker: &str) -> BTreeSet<String> {
    source
        .split(marker)
        .skip(1)
        .filter_map(|suffix| suffix.split(['<', ':', '{']).next())
        .map(str::to_owned)
        .collect()
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn method_attributes(source: &str, name: &str) -> BTreeSet<String> {
    let marker = format!("pub fn {name}(");
    let offset = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing public method {name}"));
    let mut attributes = BTreeSet::new();
    for line in source[..offset].lines().rev() {
        let line = line.trim();
        if line.starts_with("#[") {
            attributes.insert(line.to_owned());
        } else if !line.is_empty() && !line.starts_with("///") {
            break;
        }
    }
    attributes
}
