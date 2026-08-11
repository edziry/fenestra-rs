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

pub(super) fn assert_const_and_must_use(
    source: &str,
    type_name: &str,
    const_names: &[&str],
    all_names: &[&str],
) {
    let implementations = implementation_blocks(source, type_name);
    for name in all_names {
        let marker = format!("fn {name}(");
        let mut found = false;
        for implementation in &implementations {
            if let Some(offset) = implementation.find(&marker) {
                let line_start = implementation[..offset]
                    .rfind('\n')
                    .map_or(0, |newline| newline + 1);
                assert!(
                    has_must_use(implementation, line_start),
                    "{type_name}::{name}"
                );
                found = true;
            }
        }
        assert!(found, "missing method {type_name}::{name}");
    }
    for name in const_names {
        let marker = format!("pub const fn {name}");
        assert!(
            implementations
                .iter()
                .any(|implementation| implementation.contains(&marker)),
            "missing const method {type_name}::{name}"
        );
    }
}

pub(super) fn assert_struct_fields_private(source: &str, type_name: &str) {
    let marker = format!("pub struct {type_name}");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing {type_name}"));
    let declaration = &source[start..];
    let brace = declaration.find('{');
    let tuple = declaration.find('(');
    match (brace, tuple) {
        (Some(brace), None) => assert_braced_fields_private(declaration, brace),
        (Some(brace), Some(tuple)) if brace < tuple => {
            assert_braced_fields_private(declaration, brace);
        }
        (_, Some(tuple)) => {
            let end = declaration[tuple..].find(';').expect("tuple struct end") + tuple;
            let fields = &declaration[tuple + 1..end];
            assert!(!fields.contains("pub ") && !fields.contains("pub("));
        }
        _ => panic!("unsupported struct {type_name}"),
    }
}

pub(super) fn struct_field_types(source: &str, type_name: &str) -> Vec<String> {
    let marker = format!("pub struct {type_name}");
    let declaration = source
        .get(source.find(&marker).expect("registered struct")..)
        .expect("struct declaration");
    let brace = declaration.find('{');
    let tuple = declaration.find('(');
    let (open, close, named) = match (brace, tuple) {
        (Some(brace), None) => (brace, '}', true),
        (Some(brace), Some(tuple)) if brace < tuple => (brace, '}', true),
        (_, Some(tuple)) => (tuple, ')', false),
        _ => panic!("unsupported struct {type_name}"),
    };
    let end = matching_delimiter(declaration, open, close);
    split_fields(&declaration[open + 1..end])
        .into_iter()
        .map(|field| field_type(field, named))
        .collect()
}

pub(super) fn implementation_blocks<'a>(source: &'a str, type_name: &str) -> Vec<&'a str> {
    let mut remaining = source;
    let mut blocks = Vec::new();
    loop {
        let mut line_offset = 0_usize;
        let mut start = None;
        for line in remaining.split_inclusive('\n') {
            let trimmed = line.trim_start();
            if trimmed.starts_with("impl")
                && trimmed.contains(type_name)
                && !trimmed.contains(" for ")
            {
                start = Some(line_offset + line.len() - trimmed.len());
                break;
            }
            line_offset += line.len();
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

pub(super) fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
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

fn assert_braced_fields_private(declaration: &str, brace: usize) {
    let end = matching_delimiter(declaration, brace, '}');
    assert!(
        !declaration[brace + 1..end]
            .lines()
            .any(|line| line.trim_start().starts_with("pub"))
    );
}

fn matching_delimiter(source: &str, open: usize, close: char) -> usize {
    let opening = source.as_bytes()[open] as char;
    let mut depth = 0_usize;
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
    panic!("unterminated declaration")
}

fn split_fields(body: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0_usize;
    let mut angle = 0_usize;
    let mut square = 0_usize;
    let mut paren = 0_usize;
    for (offset, character) in body.char_indices() {
        match character {
            '<' => angle += 1,
            '>' => angle -= 1,
            '[' => square += 1,
            ']' => square -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            ',' if angle == 0 && square == 0 && paren == 0 => {
                if !body[start..offset].trim().is_empty() {
                    fields.push(body[start..offset].trim());
                }
                start = offset + 1;
            }
            _ => {}
        }
    }
    if !body[start..].trim().is_empty() {
        fields.push(body[start..].trim());
    }
    fields
}

fn field_type(field: &str, named: bool) -> String {
    let significant = field
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("///") && !line.starts_with("#["))
        .collect::<String>();
    let field_type = if named {
        significant.split_once(':').expect("named field type").1
    } else {
        significant
            .strip_prefix("pub(crate)")
            .or_else(|| significant.strip_prefix("pub(super)"))
            .or_else(|| significant.strip_prefix("pub"))
            .unwrap_or(&significant)
    };
    field_type
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
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
