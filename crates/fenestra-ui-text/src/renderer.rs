use fenestra_ui::{Raster, TextEngine, TextError, TextLayout, TextMetrics, TextRequest};
use parley::{
    Alignment, AlignmentOptions, Layout, LayoutContext, LineHeight, PositionedLayoutItem,
    StyleProperty,
};

use crate::{FontError, fonts::Fonts, raster};

/// A persistent private layout/raster context over an explicit ordered font set.
pub struct TextRenderer {
    fonts: Fonts,
    layout_context: LayoutContext<()>,
    raster_context: raster::Context,
}

impl TextRenderer {
    /// Copies and validates 1 through 32 explicit single-face TTF/OTF fonts.
    ///
    /// Each font is limited to 8 MiB and the entire set to 32 MiB. Limits are
    /// checked before copying or parsing. Registration is atomic: any invalid,
    /// collection, color or bitmap font fails the whole constructor. The first
    /// font is preferred, with subsequent fonts as ordered fallback candidates.
    /// Glyph masks are limited to the request pixel bound and 4,194,304 pixels;
    /// scan conversion accepts at most 65,536 outline points with coordinate
    /// magnitude at most 1,048,576 pixels. Font parsing and outline decomposition
    /// still use private candidate allocations before these output checks.
    pub fn new<B: AsRef<[u8]>>(fonts: impl IntoIterator<Item = B>) -> Result<Self, FontError> {
        Ok(Self {
            fonts: Fonts::new(fonts)?,
            layout_context: LayoutContext::new(),
            raster_context: raster::Context::default(),
        })
    }
}

impl TextEngine for TextRenderer {
    fn layout(&mut self, request: TextRequest<'_>) -> Result<TextLayout, TextError> {
        let mut builder =
            self.layout_context
                .ranged_builder(&mut self.fonts.context, request.text(), 1.0, false);
        builder.push_default(StyleProperty::FontFamily(
            self.fonts.families.as_slice().into(),
        ));
        builder.push_default(StyleProperty::FontSize(
            request.style().font_size_value() as f32
        ));
        builder.push_default(StyleProperty::LineHeight(LineHeight::Absolute(
            request.style().line_height_value() as f32,
        )));
        builder.push_default(StyleProperty::OverflowWrap(parley::OverflowWrap::BreakWord));
        let mut layout: Layout<()> = builder.build(request.text());
        layout.break_all_lines(Some(request.size().width() as f32));
        layout.align(Alignment::Start, AlignmentOptions::default());
        let metrics = preflight(&layout, request)?;
        let size = request.size();
        let length = (size.width() as usize)
            .checked_mul(size.height() as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(TextError::InvalidRaster)?;
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(length)
            .map_err(|_| TextError::Engine("text raster allocation failed".into()))?;
        pixels.resize(length, 0);
        if request.style().color_value().to_rgba8()[3] != 0 {
            raster::paint(
                &layout,
                request,
                &self.fonts.raster_keys,
                &mut self.raster_context,
                &mut pixels,
            )?;
        }
        let raster = Raster::new(size, pixels).map_err(|_| TextError::InvalidRaster)?;
        let output = TextLayout::new(raster, metrics)?;
        output.validate_request(request)?;
        Ok(output)
    }
}

fn preflight(layout: &Layout<()>, request: TextRequest<'_>) -> Result<TextMetrics, TextError> {
    if !layout.width().is_finite()
        || layout.width() < 0.0
        || !layout.height().is_finite()
        || layout.height() < 0.0
    {
        return Err(TextError::InvalidMetrics);
    }
    let mut glyphs = 0;
    let mut missing = 0;
    let mut lines = 0;
    for line in layout.lines() {
        lines += 1;
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            for glyph in run.positioned_glyphs() {
                glyphs += 1;
                if glyphs > request.limits().max_glyphs() {
                    return Err(TextError::LimitExceeded {
                        resource: "text glyphs",
                        actual: glyphs,
                        limit: request.limits().max_glyphs(),
                    });
                }
                if !glyph.x.is_finite() || !glyph.y.is_finite() || glyph.id > u32::from(u16::MAX) {
                    return Err(TextError::InvalidMetrics);
                }
                missing += usize::from(glyph.id == 0);
            }
        }
    }
    if missing > 0 {
        return Err(TextError::MissingGlyphs { count: missing });
    }
    Ok(TextMetrics::new(
        layout.width(),
        layout.height(),
        lines,
        glyphs,
        missing,
    ))
}
