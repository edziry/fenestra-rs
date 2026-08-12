use std::collections::BTreeSet;

pub(super) fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

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

pub(super) fn assert_struct_fields(source: &str, type_name: &str, expected: &[(&str, &str)]) {
    let marker = format!("pub struct {type_name}");
    let declaration = &source[source.find(&marker).expect("registered struct")..];
    let brace = declaration.find('{').expect("braced struct");
    let end = matching_delimiter(declaration, brace, '}');
    let observed = split_top_level(&declaration[brace + 1..end])
        .into_iter()
        .map(|field| {
            let significant = significant(field);
            assert!(
                !significant.starts_with("pub"),
                "{type_name} field is public: {significant}"
            );
            let (name, field_type) = significant.split_once(':').expect("named field");
            (name.to_owned(), field_type.to_owned())
        })
        .collect::<Vec<_>>();
    let expected = expected
        .iter()
        .map(|(name, field_type)| ((*name).to_owned(), (*field_type).to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(observed, expected, "field inventory for {type_name}");
}

pub(super) fn assert_private_tuple_struct(source: &str, type_name: &str, expected: &str) {
    let marker = format!("pub struct {type_name}");
    let declaration = &source[source.find(&marker).expect("registered struct")..];
    let open = declaration.find('(').expect("tuple struct");
    let close = matching_delimiter(declaration, open, ')');
    let field = significant(&declaration[open + 1..close]);
    assert!(!field.starts_with("pub"), "{type_name} field is public");
    assert_eq!(field, compact(expected), "tuple field for {type_name}");
}

pub(super) fn assert_u32_symbol_surface(source: &str, type_name: &str) {
    let marker = format!("pub struct {type_name}");
    if source.contains(&marker) {
        assert_private_tuple_struct(source, type_name, "u32");
        assert_method_surface(source, type_name, &["new", "get"], &["new", "get"]);
        return;
    }

    assert!(
        macro_invocations(source, "u32_symbol").contains(type_name),
        "missing u32 symbol declaration for {type_name}"
    );
    assert_private_tuple_struct(source, "$name", "u32");
    assert_method_surface(source, "$name", &["new", "get"], &["new", "get"]);
    assert!(
        public_methods(source, type_name).is_empty()
            && public_constants(source, type_name).is_empty(),
        "unexpected explicit surface for macro-generated {type_name}"
    );
}

pub(super) fn assert_struct_private(source: &str, type_name: &str) {
    let marker = format!("pub struct {type_name}");
    let declaration = &source[source.find(&marker).expect("registered struct")..];
    let brace = declaration.find('{');
    let tuple = declaration.find('(');
    let (open, close) = match (brace, tuple) {
        (Some(brace), None) => (brace, '}'),
        (Some(brace), Some(tuple)) if brace < tuple => (brace, '}'),
        (_, Some(tuple)) => (tuple, ')'),
        _ => panic!("unsupported struct {type_name}"),
    };
    let end = matching_delimiter(declaration, open, close);
    for field in split_top_level(&declaration[open + 1..end]) {
        assert!(
            !significant(field).starts_with("pub"),
            "{type_name} has a public field"
        );
    }
}

pub(super) fn assert_enum_body(source: &str, type_name: &str, expected: &str) {
    let marker = format!("pub enum {type_name}");
    let start = source.find(&marker).expect("registered enum");
    let declaration = &source[start..];
    let brace = declaration.find('{').expect("enum body");
    let end = matching_delimiter(declaration, brace, '}');
    let observed = significant(&declaration[brace + 1..end]).replace(",}", "}");
    let expected = compact(expected).replace(",}", "}");
    assert_eq!(observed, expected, "variant inventory for {type_name}");
    let attributes = source[..start]
        .lines()
        .rev()
        .take_while(|line| {
            let line = line.trim();
            line.is_empty() || line.starts_with("///") || line.starts_with("#[")
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !attributes.contains("non_exhaustive"),
        "{type_name} must remain exhaustive"
    );
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
        let marker = format!("fn {name}(");
        let (implementation, offset) = implementations
            .iter()
            .find_map(|implementation| {
                implementation
                    .find(&marker)
                    .map(|offset| (*implementation, offset))
            })
            .unwrap_or_else(|| panic!("missing method {type_name}::{name}"));
        assert!(
            has_must_use(implementation, offset),
            "{type_name}::{name} must be must_use"
        );
        let const_marker = format!("pub const fn {name}");
        assert_eq!(
            implementation.contains(&const_marker),
            const_methods.contains(name),
            "constness of {type_name}::{name}"
        );
    }
}

pub(super) fn assert_payload_surface(source: &str, type_name: &str, methods: &[&str]) {
    assert_eq!(public_methods(source, type_name), names(methods));
    assert!(public_constants(source, type_name).is_empty());
}

pub(super) fn implementation_blocks<'a>(source: &'a str, type_name: &str) -> Vec<&'a str> {
    let mut remaining = source;
    let mut blocks = Vec::new();
    loop {
        let mut line_offset = 0_usize;
        let mut start = None;
        for line in remaining.split_inclusive('\n') {
            let trimmed = line.trim_start();
            let declaration = trimmed.strip_prefix("impl").map(str::trim_start);
            let self_type = declaration.map(|declaration| {
                if declaration.starts_with('<') {
                    let end = matching_delimiter(declaration, 0, '>');
                    declaration[end + 1..].trim_start()
                } else {
                    declaration
                }
            });
            let impl_target =
                self_type.and_then(|self_type| self_type.split(['<', ' ', '{']).next());
            if impl_target == Some(type_name) && !trimmed.contains(" for ") {
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
    let line_start = source[..item_offset]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    if source[line_start..item_offset].contains("#[must_use") {
        return true;
    }
    for line in source[..line_start].lines().rev() {
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

fn split_top_level(body: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0_usize;
    let mut angle = 0_usize;
    let mut square = 0_usize;
    let mut paren = 0_usize;
    let mut brace = 0_usize;
    for (offset, character) in body.char_indices() {
        match character {
            '<' => angle += 1,
            '>' => angle -= 1,
            '[' => square += 1,
            ']' => square -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            ',' if angle == 0 && square == 0 && paren == 0 && brace == 0 => {
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

fn macro_invocations<'a>(source: &'a str, macro_name: &str) -> BTreeSet<&'a str> {
    let marker = format!("{macro_name}!(");
    source
        .split(&marker)
        .skip(1)
        .filter_map(|invocation| {
            let name = invocation
                .trim_start()
                .split([',', '\n'])
                .next()
                .expect("macro argument")
                .trim();
            (!name.starts_with('$')).then_some(name)
        })
        .collect()
}

fn significant(source: &str) -> String {
    compact(
        &source
            .lines()
            .filter(|line| {
                let line = line.trim();
                !line.starts_with("///") && !line.starts_with("#[")
            })
            .collect::<Vec<_>>()
            .join(""),
    )
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}
