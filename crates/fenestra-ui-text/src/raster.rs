use std::collections::HashMap;

use fenestra_ui::{TextError, TextRequest, TextViewportRequest};
use parley::{Layout, PositionedLayoutItem};
use swash::{
    CacheKey, FontRef,
    scale::{ScaleContext, outline::Outline},
    zeno::{Format, Mask, Origin, Scratch, Vector},
};

const MAX_GLYPH_PIXELS: usize = 4_194_304;
const MAX_OUTLINE_POINTS: usize = 65_536;
const MAX_COORDINATE: usize = 1_048_576;

#[derive(Default)]
pub(crate) struct Context {
    scale: ScaleContext,
    outline: Outline,
    scratch: Scratch,
    mask_pixels: Vec<u8>,
}

pub(crate) fn paint(
    layout: &Layout<()>,
    viewport: TextViewportRequest<'_>,
    keys: &HashMap<u64, (u32, CacheKey)>,
    context: &mut Context,
    pixels: &mut [u8],
) -> Result<(), TextError> {
    let Context {
        scale,
        outline,
        scratch,
        mask_pixels,
    } = context;
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let (offset, key) = keys
                .get(&run.font().data.id())
                .copied()
                .ok_or(TextError::FontUnavailable)?;
            let font = FontRef {
                data: run.font().data.as_ref(),
                offset,
                key,
            };
            let mut scaler = scale
                .builder(font)
                .size(run.font_size())
                .hint(true)
                .normalized_coords(run.normalized_coords())
                .build();
            for glyph in glyph_run.positioned_glyphs() {
                if scaler.scale_outline_into(glyph.id as u16, outline) {
                    draw_outline(
                        outline,
                        (
                            f64::from(glyph.x) - f64::from(viewport.offset_x()),
                            f64::from(glyph.y) - f64::from(viewport.offset_y()),
                        ),
                        viewport.request(),
                        scratch,
                        mask_pixels,
                        pixels,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn draw_outline(
    outline: &Outline,
    position: (f64, f64),
    request: TextRequest<'_>,
    scratch: &mut Scratch,
    mask_pixels: &mut Vec<u8>,
    pixels: &mut [u8],
) -> Result<(), TextError> {
    if outline.points().is_empty() {
        return Ok(());
    }
    check(
        "glyph outline points",
        outline.points().len(),
        MAX_OUTLINE_POINTS,
    )?;
    for point in outline.points() {
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err(TextError::InvalidMetrics);
        }
        check(
            "glyph coordinates",
            point.x.abs().max(point.y.abs()).ceil() as usize,
            MAX_COORDINATE,
        )?;
    }
    let (base_x, base_y, offset) = origin(position);
    let mut mask = Mask::with_scratch(outline.path(), scratch);
    mask.format(Format::Alpha)
        .origin(Origin::BottomLeft)
        .offset(offset)
        .render_offset(offset);
    let mut dimensions = (0, 0);
    mask.inspect(|_, width, height| dimensions = (width, height));
    let length = (dimensions.0 as usize).saturating_mul(dimensions.1 as usize);
    check(
        "glyph raster pixels",
        length,
        request.limits().max_pixels().min(MAX_GLYPH_PIXELS),
    )?;
    mask_pixels.clear();
    mask_pixels
        .try_reserve_exact(length)
        .map_err(|_| TextError::Engine("glyph raster allocation failed".into()))?;
    mask_pixels.resize(length, 0);
    let placement = mask.render_into(mask_pixels, None);
    let size = request.size();
    let color = request.style().color_value().to_rgba8();
    let left = base_x.saturating_add(i64::from(placement.left));
    let top = base_y.saturating_sub(i64::from(placement.top));
    let x0 = left.max(0);
    let y0 = top.max(0);
    let x1 = left
        .saturating_add(i64::from(placement.width))
        .min(i64::from(size.width()));
    let y1 = top
        .saturating_add(i64::from(placement.height))
        .min(i64::from(size.height()));
    for y in y0..y1 {
        for x in x0..x1 {
            let source = (y - top) as usize * placement.width as usize + (x - left) as usize;
            let destination = (y as usize * size.width() as usize + x as usize) * 4;
            blend(
                &mut pixels[destination..destination + 4],
                color,
                mask_pixels[source],
            );
        }
    }
    Ok(())
}

fn origin((x, y): (f64, f64)) -> (i64, i64, Vector) {
    let base_x = x.floor();
    let base_y = y.floor();
    // Outlines use upward Y; the owned raster and baseline use downward Y.
    (
        base_x as i64,
        base_y as i64,
        Vector::new((x - base_x) as f32, -(y - base_y) as f32),
    )
}

fn check(resource: &'static str, actual: usize, limit: usize) -> Result<(), TextError> {
    if actual > limit {
        Err(TextError::LimitExceeded {
            resource,
            actual,
            limit,
        })
    } else {
        Ok(())
    }
}

fn blend(destination: &mut [u8], color: [u8; 4], coverage: u8) {
    let alpha = (u32::from(coverage) * u32::from(color[3]) + 127) / 255;
    for channel in 0..4 {
        let source = if channel == 3 {
            alpha
        } else {
            (u32::from(color[channel]) * alpha + 127) / 255
        };
        destination[channel] =
            (source + (u32::from(destination[channel]) * (255 - alpha) + 127) / 255) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::origin;

    #[test]
    fn glyph_origin_preserves_negative_and_vertical_subpixels() {
        for (x, y) in [(-0.25, 3.75), (0.25, -3.75), (-2.0, 3.0), (1.5, 0.0)] {
            let (base_x, base_y, offset) = origin((x, y));
            assert!((0.0..1.0).contains(&offset.x));
            assert!(offset.y > -1.0 && offset.y <= 0.0);
            assert_eq!(base_x as f64 + f64::from(offset.x), x);
            assert_eq!(base_y as f64 - f64::from(offset.y), y);
        }
    }
}
