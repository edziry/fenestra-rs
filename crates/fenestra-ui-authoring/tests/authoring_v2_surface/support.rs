use std::collections::BTreeSet;

use super::source::significant;

pub(super) fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

pub(super) fn public_methods(source: &str, type_name: &str) -> BTreeSet<String> {
    let mut methods = BTreeSet::new();
    for implementation in implementation_blocks(source, type_name) {
        for line in implementation.lines().map(str::trim) {
            if line.starts_with("pub fn ") || line.starts_with("pub const fn ") {
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

pub(super) fn enum_body(source: &str, type_name: &str) -> String {
    let marker = format!("pub enum {type_name}");
    let declaration = &source[source.find(&marker).expect("registered enum")..];
    let open = declaration.find('{').expect("enum body");
    let close = matching_delimiter(declaration, open, '}');
    significant(&declaration[open + 1..close]).replace(",}", "}")
}

pub(super) fn struct_fields(source: &str, type_name: &str) -> Vec<(String, String)> {
    let marker = format!("pub struct {type_name}");
    let declaration = &source[source.find(&marker).expect("registered struct")..];
    let open = declaration.find('{').expect("braced struct");
    let close = matching_delimiter(declaration, open, '}');
    split_top_level(&declaration[open + 1..close])
        .into_iter()
        .map(|field| {
            let field = significant(field);
            assert!(!field.starts_with("pub"), "public field on {type_name}");
            let (name, field_type) = field.split_once(':').expect("named field");
            (name.to_owned(), field_type.to_owned())
        })
        .collect()
}

pub(super) fn assert_method_surface(
    source: &str,
    type_name: &str,
    expected: &[&str],
    const_methods: &[&str],
) {
    assert_eq!(public_methods(source, type_name), names(expected));
    assert!(public_constants(source, type_name).is_empty());
    let implementations = implementation_blocks(source, type_name);
    for name in expected {
        let marker = format!("fn {name}");
        let (implementation, offset) = implementations
            .iter()
            .find_map(|implementation| {
                implementation
                    .match_indices(&marker)
                    .find(|(offset, _)| {
                        implementation[offset + marker.len()..]
                            .chars()
                            .next()
                            .is_some_and(|character| matches!(character, '(' | '<'))
                    })
                    .map(|(offset, _)| (*implementation, offset))
            })
            .unwrap_or_else(|| panic!("missing method {type_name}::{name}"));
        assert!(has_must_use(implementation, offset), "{type_name}::{name}");
        assert_eq!(
            implementation.contains(&format!("pub const fn {name}")),
            const_methods.contains(name),
            "constness of {type_name}::{name}"
        );
    }
}

pub(super) fn implementation_blocks<'a>(source: &'a str, type_name: &str) -> Vec<&'a str> {
    let mut remaining = source;
    let mut blocks = Vec::new();
    loop {
        let mut line_offset = 0usize;
        let mut start = None;
        for line in remaining.split_inclusive('\n') {
            let trimmed = line.trim_start();
            let declaration = trimmed.strip_prefix("impl").map(str::trim_start);
            let self_type = declaration.map(|value| {
                if value.starts_with('<') {
                    let end = matching_delimiter(value, 0, '>');
                    value[end + 1..].trim_start()
                } else {
                    value
                }
            });
            let target = self_type.and_then(|value| value.split(['<', ' ', '{']).next());
            if target == Some(type_name) && !trimmed.contains(" for ") {
                start = Some(line_offset + line.len() - trimmed.len());
                break;
            }
            line_offset += line.len();
        }
        let Some(start) = start else { break };
        let implementation = &remaining[start..];
        let end = balanced_block_end(implementation);
        blocks.push(&implementation[..end]);
        remaining = &implementation[end..];
    }
    blocks
}

pub(super) fn trait_impl<'a>(source: &'a str, trait_name: &str, type_name: &str) -> &'a str {
    let marker = format!("impl {trait_name} for {type_name}");
    let implementation = &source[source.find(&marker).expect("trait implementation")..];
    &implementation[..balanced_block_end(implementation)]
}

pub(super) fn item_attributes(source: &str, marker: &str) -> BTreeSet<String> {
    let offset = source.find(marker).expect("registered item");
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

fn balanced_block_end(source: &str) -> usize {
    let open = source.find('{').expect("implementation body");
    matching_delimiter(source, open, '}') + 1
}

fn matching_delimiter(source: &str, open: usize, close: char) -> usize {
    let opening = source[open..].chars().next().expect("opening delimiter");
    let mut depth = 0;
    for (offset, character) in source[open..].char_indices() {
        if character == opening {
            depth += 1;
        } else if character == close {
            depth -= 1;
            if depth == 0 {
                return open + offset;
            }
        }
    }
    panic!("unterminated delimiter")
}

fn split_top_level(source: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (offset, character) in source.char_indices() {
        match character {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                let field = source[start..offset].trim();
                if !field.is_empty() {
                    fields.push(field);
                }
                start = offset + 1;
            }
            _ => {}
        }
    }
    let tail = source[start..].trim();
    if !tail.is_empty() {
        fields.push(tail);
    }
    fields
}

fn has_must_use(source: &str, item_offset: usize) -> bool {
    let item_start = source[..item_offset]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    source[..item_start]
        .trim_end()
        .lines()
        .rev()
        .find(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with("///")
        })
        .is_some_and(|line| line.trim().starts_with("#[must_use"))
}
