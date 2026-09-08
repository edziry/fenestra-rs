//! Disposable Parley layout adapter with explicit Swash outline rasterization.

use fenestra_text_screen_common::{FAMILY, FONT, Glyph, Line, Report};
use parley::fontique::{Blob, Collection, CollectionOptions};
use parley::{
    Alignment, AlignmentOptions, FontContext, Layout, LayoutContext, LineHeight,
    PositionedLayoutItem, StyleProperty,
};
use std::sync::Arc;
use swash::{
    FontRef,
    scale::{Render, ScaleContext, Source},
    zeno::{Format, Vector},
};

pub fn render(text: &str, width: u32, height: u32, font_size: f32, line_height: f32) -> Report {
    assert!(font_size.is_finite() && font_size > 0.0);
    assert!(line_height.is_finite() && line_height > 0.0);
    let mut report = Report::new(width, height);
    let mut fonts = FontContext {
        collection: Collection::new(CollectionOptions {
            system_fonts: false,
            shared: false,
        }),
        source_cache: Default::default(),
    };
    fonts
        .collection
        .register_fonts(Blob::new(Arc::new(FONT.to_vec())), None);
    let mut context = LayoutContext::new();
    let mut builder = context.ranged_builder(&mut fonts, text, 1.0, false);
    builder.push_default(StyleProperty::FontFamily(FAMILY.into()));
    builder.push_default(StyleProperty::FontSize(font_size));
    builder.push_default(StyleProperty::OverflowWrap(parley::OverflowWrap::BreakWord));
    builder.push_default(StyleProperty::LineHeight(LineHeight::Absolute(line_height)));
    let mut layout: Layout<()> = builder.build(text);
    layout.break_all_lines(Some(width as f32));
    layout.align(Alignment::Start, AlignmentOptions::default());
    report.measured_width = layout.width();
    report.measured_height = layout.height();
    let mut scaler = ScaleContext::new();
    for (line_index, line) in layout.lines().enumerate() {
        let metrics = line.metrics();
        report.lines.push(Line {
            source: line.text_range(),
            baseline: metrics.baseline,
            top: metrics.block_min_coord,
            height: metrics.line_height,
            width: metrics.advance,
        });
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let sources: Vec<_> = run
                .visual_clusters()
                .flat_map(|cluster| {
                    let source = cluster_source(cluster);
                    cluster.glyphs().map(move |_| source.clone())
                })
                .collect();
            let positioned: Vec<_> = glyph_run.positioned_glyphs().collect();
            assert_eq!(sources.len(), positioned.len(), "single style per run");
            let font = FontRef::from_index(run.font().data.as_ref(), run.font().index as usize)
                .expect("versioned valid font");
            for (glyph, source) in positioned.into_iter().zip(sources) {
                let projected = Glyph {
                    source,
                    id: glyph.id,
                    line: line_index,
                    x: glyph.x,
                    y: glyph.y,
                    advance: glyph.advance,
                    rtl: run.is_rtl(),
                };
                paint(&mut report, &mut scaler, font, font_size, &projected);
                report.glyphs.push(projected);
            }
        }
    }
    report
}

fn paint(
    report: &mut Report,
    context: &mut ScaleContext,
    font: FontRef<'_>,
    size: f32,
    glyph: &Glyph,
) {
    let mut scaler = context.builder(font).size(size).hint(true).build();
    let Some(image) = Render::new(&[Source::Outline])
        .format(Format::Alpha)
        .offset(Vector::new(glyph.x.fract(), 0.0))
        .render(&mut scaler, glyph.id as u16)
    else {
        return;
    };
    let x = glyph.x.floor() as i32 + image.placement.left;
    let y = glyph.y.floor() as i32 - image.placement.top;
    for row in 0..image.placement.height {
        for col in 0..image.placement.width {
            let alpha = image.data[(row * image.placement.width + col) as usize];
            report.paint(x + col as i32, y + row as i32, alpha);
        }
    }
}

fn cluster_source(cluster: parley::Cluster<'_, ()>) -> std::ops::Range<usize> {
    let mut source = cluster.text_range();
    if cluster.is_ligature_start() {
        let mut next = cluster.next_logical();
        while let Some(continuation) = next.filter(|item| item.is_ligature_continuation()) {
            source.end = continuation.text_range().end;
            next = continuation.next_logical();
        }
    }
    source
}
