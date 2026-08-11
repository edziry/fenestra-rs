use super::source::all_source;

const EXPECTED_LOCATION_BODY: &str = "
Input,
Viewport { extent: SpatialExtentV2 },
Node { index: u32 },
NodeField { index: u32, field: SpatialNodeFieldV2 },
Island { index: u32 },
Dependency { ordinal: u32 },
Path { index: u32, field: SpatialPathFieldV2 },
PathVerb { path: u32, verb: u32, field: SpatialPathVerbFieldV2 },
Shape { index: u32, field: SpatialShapeFieldV2 },
PolygonPoint { shape: u32, point: u32, field: SpatialPolygonPointFieldV2 },
Brush { index: u32, field: SpatialBrushFieldV2 },
GradientStop { brush: u32, stop: u32, field: SpatialGradientStopFieldV2 },
Image { index: u32, field: SpatialImageFieldV2 },
ImagePixel { image: u32, pixel: u128, channel: SpatialColorChannelV2 },
Clip { index: u32, field: SpatialClipFieldV2 },
Paint { index: u32, field: SpatialPaintFieldV2 },
Hit { index: u32, field: SpatialHitFieldV2 },
Semantic { index: u32, field: SpatialSemanticFieldV2 },
Output { table: SpatialOutputTableV2 },
OutputRecord { table: SpatialOutputTableV2, index: u32, field: SpatialOutputFieldV2 },
";

#[test]
fn trusted_location_declaration_has_exact_variants_payloads_and_order() {
    let source = all_source();
    let body = enum_body(&source, "SpatialErrorLocationV2");

    assert_eq!(normalized_declaration(body), normalized_expected());
}

fn enum_body<'a>(source: &'a str, type_name: &str) -> &'a str {
    let marker = format!("pub enum {type_name}");
    assert_eq!(
        source.matches(&marker).count(),
        1,
        "location enum must have one canonical declaration"
    );
    let declaration = &source[source.find(&marker).expect("location enum")..];
    let open = declaration.find('{').expect("location body");
    let close = matching_brace(declaration, open);
    &declaration[open + 1..close]
}

fn matching_brace(source: &str, open: usize) -> usize {
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
    panic!("unterminated location enum")
}

fn normalized_declaration(body: &str) -> String {
    body.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("///") && !line.starts_with("#["))
        .collect::<String>()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn normalized_expected() -> String {
    EXPECTED_LOCATION_BODY
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}
