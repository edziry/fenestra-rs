use super::source::all_source;

#[test]
fn geometry_records_store_only_the_registered_field_types() {
    let source = all_source();
    let expected = [
        ("SpatialPathKeyV2", vec!["u32"]),
        ("SpatialShapeKeyV2", vec!["u32"]),
        ("SpatialClipKeyV2", vec!["u32"]),
        ("SpatialPathV2", vec!["SpatialPathKeyV2", "u32", "u32"]),
        (
            "SpatialShapeV2",
            vec![
                "SpatialShapeKeyV2",
                "SpatialNodeKeyV2",
                "SpatialShapeGeometryV2",
            ],
        ),
        (
            "SpatialClipV2",
            vec![
                "SpatialClipKeyV2",
                "SpatialNodeKeyV2",
                "Option<SpatialClipKeyV2>",
                "SpatialShapeKeyV2",
                "SpatialFillRuleV2",
            ],
        ),
        (
            "SpatialGeometryInputV2",
            vec![
                "&'a[SpatialPointV2]",
                "&'a[SpatialPathVerbV2]",
                "&'a[SpatialPathV2]",
                "&'a[SpatialShapeV2]",
                "&'a[SpatialClipV2]",
            ],
        ),
    ];

    for (name, fields) in expected {
        assert_eq!(struct_field_types(&source, name), fields, "{name} storage");
    }
}

fn struct_field_types(source: &str, type_name: &str) -> Vec<String> {
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
    panic!("unterminated struct")
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
