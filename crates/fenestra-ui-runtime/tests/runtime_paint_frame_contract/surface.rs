use std::collections::BTreeSet;

use super::source::{all_source, read, source_dir};

#[test]
fn runtime_paint_frame_and_current_frame_request_exports_are_additive() {
    let source = all_source();
    let lib = read(&source_dir().join("lib.rs"));
    let exports = prototype_exports(&lib);
    let structs = public_structs(&source);

    assert_eq!(exports.len(), 76);
    assert!(exports.contains("RuntimePaintFrameV2"));
    assert!(exports.contains("VisualRequestResult"));
    assert_eq!(structs.len(), 53);
    assert!(structs.contains("RuntimePaintFrameV2"));
    assert_eq!(
        named_fields(&source, "RuntimePaintFrameV2"),
        BTreeSet::from([
            ("generation".to_owned(), "RuntimeGeneration".to_owned()),
            ("spatial".to_owned(), "SpatialPaintFrameV2<'a>".to_owned(),),
        ])
    );
}

#[test]
fn runtime_paint_frame_and_owner_method_sets_are_exact() {
    let source = all_source();
    assert_eq!(
        public_methods(&source, "RuntimePaintFrameV2"),
        names(&["generation", "spatial"])
    );
    assert_eq!(
        public_methods(&source, "FrameWork"),
        names(&[
            "id",
            "generation",
            "snapshot",
            "invalidation",
            "earliest_tick",
            "latest_tick",
            "accounted_bytes",
            "paint_frame",
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
    assert!(public_constants(&source, "RuntimePaintFrameV2").is_empty());
}

#[test]
fn runtime_paint_frame_methods_have_exact_attributes() {
    let source = all_source();
    for name in ["generation", "spatial"] {
        assert_method(&source, "RuntimePaintFrameV2", name, true, true);
    }
    assert_method(&source, "FrameWork", "paint_frame", false, true);
}

#[test]
fn runtime_paint_frame_has_no_public_constructor_or_logical_projection() {
    let source = all_source();
    let joined = implementation_blocks(&source, "RuntimePaintFrameV2").join("\n");
    for forbidden in [
        "pub fn new",
        "pub const fn new",
        "pub fn snapshot",
        "pub fn logical",
        "pub fn node",
        "pub fn mapping",
        "pub fn headless",
        "pub fn source",
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

fn named_fields(source: &str, type_name: &str) -> BTreeSet<(String, String)> {
    let marker = format!("pub struct {type_name}");
    let declaration = &source[source.find(&marker).expect("registered struct")..];
    let open = declaration.find('{').expect("named struct body");
    let close = matching_delimiter(declaration, open);
    declaration[open + 1..close]
        .split(',')
        .filter_map(|field| {
            let significant = field
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with("///"))
                .collect::<String>();
            if significant.is_empty() {
                return None;
            }
            assert!(
                !significant.starts_with("pub"),
                "public field {significant}"
            );
            let (name, field_type) = significant.split_once(':').expect("named field");
            Some((name.trim().to_owned(), field_type.trim().to_owned()))
        })
        .collect()
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

fn assert_method(source: &str, type_name: &str, name: &str, is_const: bool, is_must_use: bool) {
    let blocks = implementation_blocks(source, type_name);
    let marker = format!("fn {name}(");
    let (block, offset) = blocks
        .iter()
        .find_map(|block| block.find(&marker).map(|offset| (*block, offset)))
        .unwrap_or_else(|| panic!("missing method {type_name}::{name}"));
    let line_start = block[..offset].rfind('\n').map_or(0, |line| line + 1);
    let line_end = block[offset..]
        .find('\n')
        .map_or(block.len(), |line| offset + line);
    assert_eq!(
        block[line_start..line_end].contains("pub const fn"),
        is_const
    );
    assert_eq!(has_must_use(block, line_start), is_must_use);
}

fn implementation_blocks<'a>(source: &'a str, type_name: &str) -> Vec<&'a str> {
    let mut remaining = source;
    let mut blocks = Vec::new();
    loop {
        let mut offset = 0_usize;
        let mut start = None;
        for line in remaining.split_inclusive('\n') {
            let trimmed = line.trim_start();
            if trimmed.starts_with("impl")
                && trimmed.contains(type_name)
                && !trimmed.contains(" for ")
            {
                start = Some(offset + line.len() - trimmed.len());
                break;
            }
            offset += line.len();
        }
        let Some(start) = start else {
            break;
        };
        let implementation = &remaining[start..];
        let end = balanced_block_end(implementation);
        blocks.push(&implementation[..end]);
        remaining = &implementation[end..];
    }
    blocks
}

fn matching_delimiter(source: &str, open: usize) -> usize {
    let mut depth = 0_usize;
    for (offset, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + offset;
                }
            }
            _ => {}
        }
    }
    panic!("unterminated declaration")
}

fn balanced_block_end(source: &str) -> usize {
    matching_delimiter(source, source.find('{').expect("implementation body")) + 1
}

fn has_must_use(source: &str, item_offset: usize) -> bool {
    for line in source[..item_offset].lines().rev() {
        let line = line.trim();
        if line.starts_with("#[must_use") {
            return true;
        }
        if line.is_empty() || line.starts_with("///") || line.starts_with("#[") {
            continue;
        }
        break;
    }
    false
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
