use fenestra_text_screen_cosmic::{editor, render};
use fenestra_ui::{Bounds, Error, Raster};

use super::{FONT_SIZE, LINE_HEIGHT, TextPad};

pub(super) fn frame(pad: &TextPad) -> Result<Raster, Error> {
    let base = pad.app.raster()?;
    let mut bytes = base.bytes().to_vec();
    let size = base.size();
    let mut canvas = Canvas {
        bytes: &mut bytes,
        width: size.width(),
        height: size.height(),
    };
    canvas.label(
        pad.app.bounds("title")?,
        "A small place to write",
        26.0,
        34.0,
        [232, 242, 248],
    );
    canvas.label(pad.app.bounds("hint")?,
        "Click to place the caret. Shift + Left/Right selects. Ctrl + A selects all.\nHome/End moves through the whole text. Enter adds a line.",
        14.0, 20.0, [161, 180, 199]);
    let bounds = pad.app.bounds("editor")?;
    let text = pad.display_text();
    let geometry = editor::geometry(text, bounds.width(), FONT_SIZE, LINE_HEIGHT);
    for rect in &geometry.highlights {
        canvas.rect(bounds, *rect, [59, 98, 135], 255);
    }
    canvas.label(bounds, text.text(), FONT_SIZE, LINE_HEIGHT, [235, 241, 246]);
    if let Some(preedit) = &pad.preedit {
        let mut marked = text.clone();
        if marked.set_selection(preedit.marked).is_ok() {
            for mut rect in
                editor::geometry(&marked, bounds.width(), FONT_SIZE, LINE_HEIGHT).highlights
            {
                rect.y += rect.height - 2.0;
                rect.height = 2.0;
                canvas.rect(bounds, rect, [184, 155, 244], 255);
            }
        }
    }
    if pad.focused() && pad.preedit.as_ref().is_none_or(|value| value.caret_visible) {
        canvas.rect(bounds, geometry.caret, [96, 216, 184], 255);
    }
    canvas.label(
        pad.app.bounds("status")?,
        &pad.status_text(),
        14.0,
        20.0,
        if pad.message.is_some() {
            [255, 168, 143]
        } else {
            [139, 171, 182]
        },
    );
    Raster::new(size, bytes)
}

struct Canvas<'a> {
    bytes: &'a mut [u8],
    width: u32,
    height: u32,
}

impl Canvas<'_> {
    fn label(
        &mut self,
        bounds: Bounds,
        text: &str,
        font_size: f32,
        line_height: f32,
        rgb: [u8; 3],
    ) {
        if bounds.width() == 0 || bounds.height() == 0 {
            return;
        }
        let report = render(
            text,
            bounds.width(),
            bounds.height(),
            font_size,
            line_height,
        );
        for y in 0..report.height {
            for x in 0..report.width {
                let alpha = report.rgba[(y as usize * report.width as usize + x as usize) * 4 + 3];
                if alpha != 0 {
                    self.pixel(
                        bounds.x() + i64::from(x),
                        bounds.y() + i64::from(y),
                        rgb,
                        alpha,
                    );
                }
            }
        }
    }

    fn rect(&mut self, bounds: Bounds, rect: editor::TextRect, rgb: [u8; 3], alpha: u8) {
        if ![rect.x, rect.y, rect.width, rect.height]
            .iter()
            .all(|value| value.is_finite())
        {
            return;
        }
        let x0 = rect.x.floor().clamp(0.0, bounds.width() as f32) as i64;
        let x1 = (rect.x + rect.width)
            .ceil()
            .clamp(0.0, bounds.width() as f32) as i64;
        let y0 = rect.y.floor().clamp(0.0, bounds.height() as f32) as i64;
        let y1 = (rect.y + rect.height)
            .ceil()
            .clamp(0.0, bounds.height() as f32) as i64;
        for y in y0..y1 {
            for x in x0..x1 {
                self.pixel(bounds.x() + x, bounds.y() + y, rgb, alpha);
            }
        }
    }

    fn pixel(&mut self, x: i64, y: i64, rgb: [u8; 3], alpha: u8) {
        if x < 0 || y < 0 || x >= i64::from(self.width) || y >= i64::from(self.height) {
            return;
        }
        let index = (y as usize * self.width as usize + x as usize) * 4;
        let alpha = u32::from(alpha);
        for (channel, source) in rgb.into_iter().enumerate() {
            let old = u32::from(self.bytes[index + channel]);
            self.bytes[index + channel] =
                ((u32::from(source) * alpha + old * (255 - alpha) + 127) / 255) as u8;
        }
        self.bytes[index + 3] =
            (alpha + (u32::from(self.bytes[index + 3]) * (255 - alpha) + 127) / 255) as u8;
    }
}
