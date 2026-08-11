use std::collections::BTreeSet;

pub(super) fn public_methods(source: &str, type_name: &str) -> BTreeSet<String> {
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

pub(super) fn public_constants(source: &str, type_name: &str) -> BTreeSet<String> {
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

pub(super) fn assert_const_and_must_use(source: &str, type_name: &str, names: &[&str]) {
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

pub(super) fn has_must_use(source: &str, item_offset: usize) -> bool {
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

pub(super) fn implementation_blocks<'a>(source: &'a str, type_name: &str) -> Vec<&'a str> {
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

pub(super) fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
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
