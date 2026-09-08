use std::cell::Cell;

use super::RasterCache;
use crate::{Application, Color, Element, Error, InputEvent, Limits, Raster, Size, Style, View};

fn pixel(red: u8) -> Raster {
    Raster::new(Size::new(1, 1), vec![red, 0, 0, 255]).unwrap()
}

#[test]
fn repeated_generation_renders_once_and_returns_exact_owned_pixels() {
    let cache = RasterCache::default();
    let calls = Cell::new(0);
    let mut retained = Vec::new();
    for _ in 0..4 {
        retained.push(
            cache
                .get_or_render(0, || {
                    calls.set(calls.get() + 1);
                    Ok(pixel(80))
                })
                .unwrap(),
        );
    }
    assert_eq!(calls.get(), 1);
    assert!(
        retained
            .iter()
            .all(|raster| raster.bytes() == [80, 0, 0, 255])
    );
}

#[test]
fn revisiting_an_old_generation_renders_again_instead_of_retaining_history() {
    let cache = RasterCache::default();
    let calls = Cell::new(0);
    for generation in [0, 1, 0] {
        let raster = cache
            .get_or_render(generation, || {
                calls.set(calls.get() + 1);
                Ok(pixel(calls.get()))
            })
            .unwrap();
        assert_eq!(raster.bytes(), &[calls.get(), 0, 0, 255]);
    }
    assert_eq!(calls.get(), 3);
}

#[test]
fn failed_new_generation_never_returns_stale_pixels_and_can_be_retried() {
    let cache = RasterCache::default();
    let old = cache.get_or_render(0, || Ok(pixel(10))).unwrap();
    let calls = Cell::new(0);
    for _ in 0..2 {
        assert_eq!(
            cache.get_or_render(1, || {
                calls.set(calls.get() + 1);
                Err(Error::InvalidRaster)
            }),
            Err(Error::InvalidRaster)
        );
    }
    let fresh = cache
        .get_or_render(1, || {
            calls.set(calls.get() + 1);
            Ok(pixel(20))
        })
        .unwrap();
    assert_eq!(fresh.bytes(), &[20, 0, 0, 255]);
    assert_eq!(old.bytes(), &[10, 0, 0, 255]);
    assert_eq!(
        cache.get_or_render(1, || Err(Error::CapacityOverflow)),
        Ok(fresh)
    );
    assert_eq!(calls.get(), 3);
}

#[test]
fn explicit_clear_requires_a_fresh_render_even_for_the_same_key() {
    let mut cache = RasterCache::default();
    let old = cache.get_or_render(4, || Ok(pixel(10))).unwrap();
    cache.clear();
    let fresh = cache.get_or_render(4, || Ok(pixel(20))).unwrap();
    assert_eq!(old.bytes(), &[10, 0, 0, 255]);
    assert_eq!(fresh.bytes(), &[20, 0, 0, 255]);
}

fn application(control: bool) -> Application {
    let root = if control {
        Element::button("root", "Action")
    } else {
        Element::rect("root")
    };
    Application::with_limits(
        View::new(
            "cached",
            root.style(
                Style::new()
                    .width(4)
                    .height(4)
                    .background(Color::rgba8(30, 0, 0, 255)),
            ),
        ),
        Size::new(4, 4),
        Limits::new(4, 4, 16),
    )
    .unwrap()
}

#[test]
fn application_renders_lazily_once_after_accepted_publication() {
    let mut app = application(false);
    assert_eq!(app.rasterizations.get(), 0);
    app.set_background("root", Color::rgba8(60, 0, 0, 255))
        .unwrap();
    assert_eq!(app.rasterizations.get(), 0);
    let generation = app.generation();
    for _ in 0..4 {
        assert_eq!(app.raster().unwrap().bytes(), [60, 0, 0, 255].repeat(16));
    }
    assert_eq!(app.rasterizations.get(), 1);
    assert_eq!(app.generation(), generation);
}

#[test]
fn noops_rejections_and_pointer_bookkeeping_keep_the_accepted_render() {
    let mut app = application(false);
    let before = app.raster().unwrap();
    let generation = app.generation();
    app.set_style("root", app.style("root").unwrap()).unwrap();
    app.resize(Size::new(4, 4)).unwrap();
    assert!(app.set_size("root", -1, 4).is_err());
    assert!(app.resize(Size::new(5, 4)).is_err());
    app.dispatch_input(InputEvent::PointerMoved { x: 1, y: 1 })
        .unwrap();
    app.dispatch_input(InputEvent::PointerMoved { x: 2, y: 2 })
        .unwrap();
    assert_eq!(app.generation(), generation);
    assert_eq!(app.raster().unwrap(), before);
    assert_eq!(app.rasterizations.get(), 1);
}

#[test]
fn focus_attachment_changes_invalidate_a_frame_with_unchanged_core_colors() {
    let mut app = application(true);
    let before = app.raster().unwrap();
    assert_eq!(&before.bytes()[..4], &[30, 0, 0, 255]);
    app.focus(Some("root")).unwrap();
    assert_eq!(app.rasterizations.get(), 1);
    let focused = app.raster().unwrap();
    assert_eq!(&focused.bytes()[..4], &[0, 0, 0, 255]);
    assert_eq!(app.raster().unwrap(), focused);
    assert_eq!(app.rasterizations.get(), 2);
    assert_eq!(&before.bytes()[..4], &[30, 0, 0, 255]);
}
