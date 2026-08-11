use crate::*;

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
    let normalized = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("///") && !line.starts_with("#["))
        .collect::<String>()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    normalized.replace(",}", "}")
}

fn normalized_expected() -> String {
    let normalized = EXPECTED_LOCATION_BODY
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    normalized.replace(",}", "}")
}

#[test]
fn every_location_variant_is_constructible_and_exhaustively_matchable() {
    let wide_pixel = u128::from(u32::MAX) + 1;
    let locations = [
        SpatialErrorLocationV2::Input,
        SpatialErrorLocationV2::Viewport {
            extent: SpatialExtentV2::Height,
        },
        SpatialErrorLocationV2::Node { index: 2 },
        SpatialErrorLocationV2::NodeField {
            index: 3,
            field: SpatialNodeFieldV2::TargetKey,
        },
        SpatialErrorLocationV2::Island { index: 4 },
        SpatialErrorLocationV2::Dependency { ordinal: 5 },
        SpatialErrorLocationV2::Path {
            index: 6,
            field: SpatialPathFieldV2::VerbLength,
        },
        SpatialErrorLocationV2::PathVerb {
            path: 7,
            verb: 8,
            field: SpatialPathVerbFieldV2::Control2Y,
        },
        SpatialErrorLocationV2::Shape {
            index: 9,
            field: SpatialShapeFieldV2::CircleRadius,
        },
        SpatialErrorLocationV2::PolygonPoint {
            shape: 10,
            point: 11,
            field: SpatialPolygonPointFieldV2::Y,
        },
        SpatialErrorLocationV2::Brush {
            index: 12,
            field: SpatialBrushFieldV2::GradientEndY,
        },
        SpatialErrorLocationV2::GradientStop {
            brush: 13,
            stop: 14,
            field: SpatialGradientStopFieldV2::A,
        },
        SpatialErrorLocationV2::Image {
            index: 15,
            field: SpatialImageFieldV2::Pixel,
        },
        SpatialErrorLocationV2::ImagePixel {
            image: 16,
            pixel: wide_pixel,
            channel: SpatialColorChannelV2::A,
        },
        SpatialErrorLocationV2::Clip {
            index: 17,
            field: SpatialClipFieldV2::FillRule,
        },
        SpatialErrorLocationV2::Paint {
            index: 18,
            field: SpatialPaintFieldV2::Opacity,
        },
        SpatialErrorLocationV2::Hit {
            index: 19,
            field: SpatialHitFieldV2::InputPolicy,
        },
        SpatialErrorLocationV2::Semantic {
            index: 20,
            field: SpatialSemanticFieldV2::Clip,
        },
        SpatialErrorLocationV2::Output {
            table: SpatialOutputTableV2::Hit,
        },
        SpatialErrorLocationV2::OutputRecord {
            table: SpatialOutputTableV2::Semantic,
            index: 21,
            field: SpatialOutputFieldV2::ItemOrdinal,
        },
    ];

    assert_eq!(
        locations.map(location_ordinal),
        expected_location_ordinals()
    );
}

fn expected_location_ordinals() -> [u8; 20] {
    [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    ]
}

fn location_ordinal(location: SpatialErrorLocationV2) -> u8 {
    match location {
        SpatialErrorLocationV2::Input => 0,
        SpatialErrorLocationV2::Viewport { extent } => {
            assert_eq!(extent, SpatialExtentV2::Height);
            1
        }
        SpatialErrorLocationV2::Node { index } => {
            assert_eq!(index, 2);
            2
        }
        SpatialErrorLocationV2::NodeField { index, field } => {
            assert_eq!((index, field), (3, SpatialNodeFieldV2::TargetKey));
            3
        }
        SpatialErrorLocationV2::Island { index } => {
            assert_eq!(index, 4);
            4
        }
        SpatialErrorLocationV2::Dependency { ordinal } => {
            assert_eq!(ordinal, 5);
            5
        }
        SpatialErrorLocationV2::Path { index, field } => {
            assert_eq!((index, field), (6, SpatialPathFieldV2::VerbLength));
            6
        }
        SpatialErrorLocationV2::PathVerb { path, verb, field } => {
            assert_eq!(
                (path, verb, field),
                (7, 8, SpatialPathVerbFieldV2::Control2Y)
            );
            7
        }
        SpatialErrorLocationV2::Shape { index, field } => {
            assert_eq!((index, field), (9, SpatialShapeFieldV2::CircleRadius));
            8
        }
        SpatialErrorLocationV2::PolygonPoint {
            shape,
            point,
            field,
        } => {
            assert_eq!(
                (shape, point, field),
                (10, 11, SpatialPolygonPointFieldV2::Y)
            );
            9
        }
        SpatialErrorLocationV2::Brush { index, field } => {
            assert_eq!((index, field), (12, SpatialBrushFieldV2::GradientEndY));
            10
        }
        SpatialErrorLocationV2::GradientStop { brush, stop, field } => {
            assert_eq!(
                (brush, stop, field),
                (13, 14, SpatialGradientStopFieldV2::A)
            );
            11
        }
        SpatialErrorLocationV2::Image { index, field } => {
            assert_eq!((index, field), (15, SpatialImageFieldV2::Pixel));
            12
        }
        SpatialErrorLocationV2::ImagePixel {
            image,
            pixel,
            channel,
        } => {
            assert_eq!(image, 16);
            assert_eq!(pixel, u128::from(u32::MAX) + 1);
            assert!(pixel > u128::from(u32::MAX));
            assert_eq!(channel, SpatialColorChannelV2::A);
            13
        }
        SpatialErrorLocationV2::Clip { index, field } => {
            assert_eq!((index, field), (17, SpatialClipFieldV2::FillRule));
            14
        }
        SpatialErrorLocationV2::Paint { index, field } => {
            assert_eq!((index, field), (18, SpatialPaintFieldV2::Opacity));
            15
        }
        SpatialErrorLocationV2::Hit { index, field } => {
            assert_eq!((index, field), (19, SpatialHitFieldV2::InputPolicy));
            16
        }
        SpatialErrorLocationV2::Semantic { index, field } => {
            assert_eq!((index, field), (20, SpatialSemanticFieldV2::Clip));
            17
        }
        SpatialErrorLocationV2::Output { table } => {
            assert_eq!(table, SpatialOutputTableV2::Hit);
            18
        }
        SpatialErrorLocationV2::OutputRecord {
            table,
            index,
            field,
        } => {
            assert_eq!(
                (table, index, field),
                (
                    SpatialOutputTableV2::Semantic,
                    21,
                    SpatialOutputFieldV2::ItemOrdinal,
                )
            );
            19
        }
    }
}
