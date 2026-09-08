use fenestra_ui_spatial::prototype::{
    SpatialImageDestinationRectV2, SpatialImageKeyV2, SpatialImagePaintAttachmentV2,
    SpatialImageV2, SpatialNodeKeyV2, SpatialScalarV2,
};

use crate::{Color, Size};

pub(super) fn focus_paints(
    owner: SpatialNodeKeyV2,
    size: Size,
    color: Option<Color>,
    offset: (i64, i64),
) -> Vec<SpatialImagePaintAttachmentV2> {
    let (width, height) = (size.width(), size.height());
    if width == 0 || height == 0 {
        return Vec::new();
    }
    let mut paints = Vec::with_capacity(8);
    let bounds = [0, 0, width, height];
    if let Some(color) = color {
        border(&mut paints, owner, offset, bounds, 2, color);
    } else if width < 3 || height < 3 {
        border(
            &mut paints,
            owner,
            offset,
            bounds,
            1,
            Color::rgba8(255, 255, 255, 255),
        );
    } else {
        border(
            &mut paints,
            owner,
            offset,
            bounds,
            1,
            Color::rgba8(0, 0, 0, 255),
        );
        border(
            &mut paints,
            owner,
            offset,
            [1, 1, width - 2, height - 2],
            1,
            Color::rgba8(255, 255, 255, 255),
        );
    }
    paints
}

fn border(
    paints: &mut Vec<SpatialImagePaintAttachmentV2>,
    owner: SpatialNodeKeyV2,
    offset: (i64, i64),
    [x, y, width, height]: [u32; 4],
    thickness: u32,
    color: Color,
) {
    let top = thickness.min(height);
    let bottom = thickness.min(height - top);
    let middle = height - top - bottom;
    let left = thickness.min(width);
    let right = thickness.min(width - left);
    let mut rgba = color.to_rgba8();
    let alpha = u16::from(rgba[3]);
    for channel in &mut rgba[..3] {
        *channel = ((u16::from(*channel) * alpha + 127) / 255) as u8;
    }
    // Top and bottom own the corners; side strips cover only the middle.
    // Partitioning even tiny rectangles keeps translucent pixels from blending twice.
    for [x, y, width, height] in [
        [x, y, width, top],
        [x, y + height - bottom, width, bottom],
        [x, y + top, left, middle],
        [x + width - right, y + top, right, middle],
    ] {
        if width == 0 || height == 0 {
            continue;
        }
        let scalar = |value| SpatialScalarV2::new(i64::from(value) * 65_536);
        paints.push(SpatialImagePaintAttachmentV2::new(
            owner,
            SpatialImageV2::new(SpatialImageKeyV2::new(0), 1, 1, 4, Box::new(rgba)),
            SpatialImageDestinationRectV2::new(
                SpatialScalarV2::new((i64::from(x) + offset.0) * 65_536),
                SpatialScalarV2::new((i64::from(y) + offset.1) * 65_536),
                scalar(width),
                scalar(height),
            ),
            None,
        ));
    }
}

#[cfg(test)]
mod tests {
    use fenestra_ui_spatial::prototype::{
        ReferenceRasterLimitsV2, SpatialLimitKindV2, SpatialLimitsV2, SpatialPointV2,
        SpatialScalarV2,
    };

    use super::*;
    use crate::{Application, Element, Raster, Style, View};

    fn application(size: Size, viewport: Size, background: Color) -> Application {
        Application::new(
            View::new(
                "focus",
                Element::rect("owner").style(
                    Style::new()
                        .width(size.width() as i32)
                        .height(size.height() as i32)
                        .background(background)
                        .input(true),
                ),
            ),
            viewport,
        )
        .unwrap()
    }

    fn decorated(app: &Application, size: Size, color: Option<Color>) -> Raster {
        let committed = app.runtime.committed();
        let spatial = committed.spatial().unwrap();
        let owner = spatial.spatial_key(app.nodes[0].id).unwrap();
        let before = spatial.snapshot();
        let additions = focus_paints(owner, size, color, (0, 0));
        assert!(additions.len() <= 8);
        let limits = SpatialLimitsV2::new(SpatialLimitKindV2::ALL.map(|kind| match kind {
            SpatialLimitKindV2::Images | SpatialLimitKindV2::ImagePixelsTotal => 8,
            SpatialLimitKindV2::PaintItems | SpatialLimitKindV2::PaintItemsPerNode => 9,
            SpatialLimitKindV2::ImageEdge => 1,
            _ => app.spatial_limits.limit(kind),
        }));
        let after = before
            .with_image_paints(additions.into_boxed_slice(), limits)
            .unwrap();
        assert_eq!(after.output().geometry(), before.output().geometry());
        assert_eq!(after.output().hits(), before.output().hits());
        assert_eq!(after.output().semantics(), before.output().semantics());
        for y in 0..app.size.height() {
            for x in 0..app.size.width() {
                let point = SpatialPointV2::new(
                    SpatialScalarV2::new(i64::from(x) * 65_536),
                    SpatialScalarV2::new(i64::from(y) * 65_536),
                );
                assert_eq!(after.hit_test(point), before.hit_test(point));
            }
        }
        let raster = after
            .paint_frame()
            .rasterize_reference(ReferenceRasterLimitsV2::new(1024))
            .unwrap();
        Raster::new(app.size, raster.bytes().to_vec()).unwrap()
    }

    fn pixel(raster: &Raster, x: usize, y: usize) -> [u8; 4] {
        let start = (y * raster.size().width() as usize + x) * 4;
        raster.bytes()[start..start + 4].try_into().unwrap()
    }

    #[test]
    fn default_focus_has_black_and_white_interior_edges_on_light_and_dark_backgrounds() {
        for background in [Color::rgba8(0, 0, 0, 255), Color::rgba8(255, 255, 255, 255)] {
            let size = Size::new(7, 7);
            let app = application(size, Size::new(9, 9), background);
            let raster = decorated(&app, size, None);
            assert_eq!(pixel(&raster, 0, 0), [0, 0, 0, 255]);
            assert_eq!(pixel(&raster, 6, 6), [0, 0, 0, 255]);
            assert_eq!(pixel(&raster, 1, 1), [255; 4]);
            assert_eq!(pixel(&raster, 5, 5), [255; 4]);
            assert_eq!(pixel(&raster, 3, 3), background.to_rgba8());
            assert_eq!(pixel(&raster, 7, 1), [0; 4]);
            assert_eq!(pixel(&raster, 1, 7), [0; 4]);
        }
    }

    #[test]
    fn custom_focus_is_two_pixels_thick_and_transparency_is_composited_once() {
        let size = Size::new(7, 7);
        let app = application(size, size, Color::rgba8(0, 0, 0, 0));
        for (color, expected) in [
            (Color::rgba8(129, 65, 33, 128), [65, 33, 17, 128]),
            (Color::rgba8(255, 127, 1, 1), [1, 0, 0, 1]),
            (Color::rgba8(255, 255, 255, 0), [0; 4]),
        ] {
            let raster = decorated(&app, size, Some(color));
            for (x, y) in [(0, 0), (1, 1), (5, 1), (6, 6), (0, 3), (3, 6)] {
                assert_eq!(pixel(&raster, x, y), expected);
            }
            assert_eq!(pixel(&raster, 3, 3), [0; 4]);
        }
        let opaque = application(size, size, Color::rgba8(20, 40, 60, 255));
        let raster = decorated(&opaque, size, Some(Color::rgba8(129, 65, 33, 128)));
        assert_eq!(pixel(&raster, 0, 0), [75, 53, 47, 255]);
        assert_eq!(pixel(&raster, 1, 1), [75, 53, 47, 255]);
        assert_eq!(pixel(&raster, 3, 3), [20, 40, 60, 255]);
    }

    #[test]
    fn small_focus_areas_do_not_overlap_or_escape_the_owner() {
        for (width, height) in [(1, 1), (1, 5), (5, 1), (2, 5), (5, 2), (3, 3), (4, 4)] {
            let size = Size::new(width, height);
            let app = application(size, Size::new(7, 7), Color::rgba8(0, 0, 0, 0));
            let custom = decorated(&app, size, Some(Color::rgba8(100, 50, 20, 128)));
            for y in 0..height as usize {
                for x in 0..width as usize {
                    assert_eq!(pixel(&custom, x, y), [50, 25, 10, 128]);
                }
            }
            assert_eq!(pixel(&custom, width as usize, 0), [0; 4]);
            assert_eq!(pixel(&custom, 0, height as usize), [0; 4]);
            if width < 3 || height < 3 {
                let default = decorated(&app, size, None);
                assert_eq!(pixel(&default, 0, 0), [255; 4]);
                assert_eq!(
                    pixel(&default, width as usize - 1, height as usize - 1),
                    [255; 4]
                );
            }
        }
    }

    #[test]
    fn viewport_clips_the_focus_without_moving_its_edges() {
        let size = Size::new(7, 7);
        let app = application(size, Size::new(4, 3), Color::rgba8(20, 40, 60, 255));
        let raster = decorated(&app, size, None);
        assert_eq!(pixel(&raster, 0, 2), [0, 0, 0, 255]);
        assert_eq!(pixel(&raster, 1, 2), [255; 4]);
        assert_eq!(pixel(&raster, 2, 2), [20, 40, 60, 255]);
        assert_eq!(pixel(&raster, 3, 2), [20, 40, 60, 255]);
    }

    #[test]
    fn empty_focus_areas_add_no_image_resources() {
        for size in [Size::new(0, 8), Size::new(8, 0), Size::new(0, 0)] {
            for color in [None, Some(Color::rgba8(100, 50, 20, 128))] {
                assert!(focus_paints(SpatialNodeKeyV2::new(0), size, color, (0, 0)).is_empty());
            }
        }
    }

    #[test]
    fn maximum_owner_extent_uses_only_eight_single_pixel_images() {
        let size = Size::new(i32::MAX as u32, i32::MAX as u32);
        let app = application(size, Size::new(3, 3), Color::rgba8(20, 40, 60, 255));
        let raster = decorated(&app, size, None);
        assert_eq!(pixel(&raster, 0, 0), [0, 0, 0, 255]);
        assert_eq!(pixel(&raster, 1, 1), [255; 4]);
        assert_eq!(pixel(&raster, 2, 2), [20, 40, 60, 255]);
    }
}
