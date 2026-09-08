use fenestra_ui::{Color, Size, TextEngine, TextError, TextLimits, TextRequest, TextStyle};
use fenestra_ui_text::{FontError, TextRenderer};

const FONT: &[u8] = include_bytes!("../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");

fn request(text: &str, size: Size) -> TextRequest<'_> {
    TextRequest::new(
        text,
        TextStyle::new().font_size(20).line_height(28),
        size,
        TextLimits::default(),
    )
    .unwrap()
}

#[test]
fn registration_requires_valid_single_face_outline_fonts() {
    assert_eq!(
        TextRenderer::new([] as [&[u8]; 0]).err(),
        Some(FontError::EmptySet)
    );
    for bytes in [b"".as_slice(), b"invalid font", &FONT[..64]] {
        assert_eq!(
            TextRenderer::new([bytes]).err(),
            Some(FontError::InvalidFont { index: 0 })
        );
    }
    assert_eq!(
        TextRenderer::new([b"ttcf\0\x01\0\0\xff\xff\xff\xff".as_slice()]).err(),
        Some(FontError::CollectionUnsupported { index: 0 })
    );
    assert_eq!(
        TextRenderer::new([FONT, b"invalid font"]).err(),
        Some(FontError::InvalidFont { index: 1 })
    );
}

#[test]
fn font_budgets_are_checked_before_font_copy_or_parse() {
    assert!(matches!(
        TextRenderer::new([FONT; 33]),
        Err(FontError::LimitExceeded {
            resource: "font count",
            actual: 33,
            limit: 32
        })
    ));
    let oversize = vec![0; 8 * 1024 * 1024 + 1];
    assert!(matches!(
        TextRenderer::new([oversize.as_slice()]),
        Err(FontError::LimitExceeded {
            resource: "font bytes",
            ..
        })
    ));
    let large = vec![0; 8 * 1024 * 1024];
    assert!(matches!(
        TextRenderer::new([large.as_slice(); 5]),
        Err(FontError::LimitExceeded {
            resource: "total font bytes",
            ..
        })
    ));
}

#[test]
fn font_table_bounds_and_unsupported_raster_formats_fail_before_candidate_work() {
    let mut invalid_offset = FONT.to_vec();
    invalid_offset[20..24].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(
        TextRenderer::new([invalid_offset]).err(),
        Some(FontError::InvalidFont { index: 0 })
    );
    let mut color = FONT.to_vec();
    let record = table_record(b"FFTM");
    color[record..record + 4].copy_from_slice(b"COLR");
    assert_eq!(
        TextRenderer::new([color]).err(),
        Some(FontError::ColorOrBitmapUnsupported { index: 0 })
    );
    let mut no_outlines = FONT.to_vec();
    let record = table_record(b"glyf");
    no_outlines[record..record + 4].copy_from_slice(b"xxxx");
    assert_eq!(
        TextRenderer::new([no_outlines]).err(),
        Some(FontError::OutlinesUnavailable { index: 0 })
    );
}

#[test]
fn zero_em_fonts_are_rejected_and_expanded_glyph_masks_have_a_separate_budget() {
    let head = read_u32(FONT, table_record(b"head") + 8) as usize;
    let mut invalid_em = FONT.to_vec();
    invalid_em[head + 18..head + 20].copy_from_slice(&0_u16.to_be_bytes());
    assert_eq!(
        TextRenderer::new([invalid_em]).err(),
        Some(FontError::InvalidFont { index: 0 })
    );

    let mut expanded = FONT.to_vec();
    expanded[head + 18..head + 20].copy_from_slice(&16_u16.to_be_bytes());
    let mut renderer = TextRenderer::new([expanded]).unwrap();
    let request = TextRequest::new(
        "A",
        TextStyle::new().font_size(20),
        Size::new(1, 1),
        TextLimits::new(32, 256, 32),
    )
    .unwrap();
    assert!(matches!(
        renderer.layout(request),
        Err(TextError::LimitExceeded {
            resource: "glyph raster pixels",
            ..
        })
    ));
}

#[test]
fn persistent_contexts_keep_multilingual_measurement_and_pixels_repeatable() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "Latin \u{03b1}\u{03b2}\u{03b3} \u{05e9}\u{05dc}\u{05d5}\u{05dd} \u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064a}\u{0629} e\u{301}";
    let first = renderer.layout(request(text, Size::new(300, 140))).unwrap();
    renderer
        .layout(request("other", Size::new(80, 28)))
        .unwrap();
    let second = renderer.layout(request(text, Size::new(300, 140))).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.metrics().missing_glyphs(), 0);
    assert!(first.metrics().glyphs() > 0);
    assert!(first.raster().bytes().iter().any(|byte| *byte != 0));
}

#[test]
fn complete_measurement_and_glyph_budget_include_clipped_lines() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "first\nsecond\nthird\nfourth";
    let short = renderer.layout(request(text, Size::new(130, 28))).unwrap();
    let tall = renderer.layout(request(text, Size::new(130, 140))).unwrap();
    assert_eq!(short.metrics(), tall.metrics());
    assert_eq!(short.metrics().height(), 112.0);
    assert_eq!(
        short.raster().bytes(),
        &tall.raster().bytes()[..130 * 28 * 4]
    );
    let limited = TextRequest::new(
        text,
        TextStyle::new(),
        Size::new(130, 1),
        TextLimits::new(100, 130, 2),
    )
    .unwrap();
    assert!(matches!(
        renderer.layout(limited),
        Err(TextError::LimitExceeded {
            resource: "text glyphs",
            ..
        })
    ));
}

#[test]
fn raster_uses_requested_premultiplied_color_and_transparency() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    for color in [Color::rgba8(210, 80, 20, 128), Color::rgba8(255, 90, 70, 0)] {
        let request = TextRequest::new(
            "ink",
            TextStyle::new().color(color),
            Size::new(120, 40),
            TextLimits::default(),
        )
        .unwrap();
        let layout = renderer.layout(request).unwrap();
        for pixel in layout.raster().bytes().chunks_exact(4) {
            assert!(pixel[..3].iter().all(|channel| *channel <= pixel[3]));
            assert!(pixel[3] <= color.to_rgba8()[3]);
        }
        if color.to_rgba8()[3] == 0 {
            assert!(layout.raster().bytes().iter().all(|byte| *byte == 0));
        } else {
            assert!(
                layout
                    .raster()
                    .bytes()
                    .chunks_exact(4)
                    .any(|pixel| pixel[0] > pixel[1] && pixel[1] > pixel[2])
            );
        }
    }
}

#[test]
fn missing_coverage_is_a_typed_error_and_does_not_poison_later_layout() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    assert_eq!(
        renderer
            .layout(request("\u{4e2d}\u{1f680}", Size::new(130, 56)))
            .err(),
        Some(TextError::MissingGlyphs { count: 2 })
    );
    assert!(
        renderer
            .layout(request("valid", Size::new(130, 56)))
            .is_ok()
    );
}

#[test]
fn font_order_supplies_an_explicit_fallback_without_system_fonts() {
    let no_coverage = reduced_ascii_coverage();
    let mut missing = TextRenderer::new([no_coverage.as_slice()]).unwrap();
    assert!(matches!(
        missing.layout(request("A", Size::new(60, 40))),
        Err(TextError::MissingGlyphs { .. })
    ));
    let mut fallback = TextRenderer::new([no_coverage.as_slice(), FONT]).unwrap();
    let mut only_valid = TextRenderer::new([FONT]).unwrap();
    assert_eq!(
        fallback.layout(request("A", Size::new(60, 40))).unwrap(),
        only_valid.layout(request("A", Size::new(60, 40))).unwrap()
    );
}

fn table_record(tag: &[u8; 4]) -> usize {
    let tables = u16::from_be_bytes(FONT[4..6].try_into().unwrap()) as usize;
    (0..tables)
        .map(|i| 12 + 16 * i)
        .find(|i| &FONT[*i..*i + 4] == tag)
        .unwrap()
}

fn reduced_ascii_coverage() -> Vec<u8> {
    // Derive a valid reduced-coverage cmap in memory from the versioned fixture.
    // Keep its encoding records; removing them would make registration invalid.
    let mut bytes = FONT.to_vec();
    let record = table_record(b"cmap");
    let cmap = read_u32(FONT, record + 8) as usize;
    let count = read_u16(FONT, cmap + 2) as usize;
    for encoding in 0..count {
        let offset = cmap + read_u32(FONT, cmap + 8 + encoding * 8) as usize;
        match read_u16(FONT, offset) {
            4 => {
                let segments = read_u16(FONT, offset + 6) as usize / 2;
                for segment in 0..segments {
                    let end = offset + 14 + segment * 2;
                    let start = offset + 16 + segments * 2 + segment * 2;
                    if read_u16(FONT, start) <= 64 && read_u16(FONT, end) >= 65 {
                        bytes[end..end + 2].copy_from_slice(&64_u16.to_be_bytes());
                    }
                }
            }
            12 => {
                for group in 0..read_u32(FONT, offset + 12) as usize {
                    let start = offset + 16 + group * 12;
                    if read_u32(FONT, start) <= 64 && read_u32(FONT, start + 4) >= 65 {
                        bytes[start + 4..start + 8].copy_from_slice(&64_u32.to_be_bytes());
                    }
                }
            }
            _ => {}
        }
    }
    bytes
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
