//! Public text geometry, viewport pixels and native caret-context preview.

use std::error::Error;
use std::path::PathBuf;

use fenestra_text_app::checksum;
use fenestra_ui::Size;

#[path = "geometry_gallery/canvas.rs"]
mod canvas;
#[path = "../src/export.rs"]
mod export;
#[path = "geometry_gallery/scene.rs"]
mod scene;

type GalleryError = Box<dyn Error + Send + Sync>;

fn main() -> Result<(), GalleryError> {
    let mut args = std::env::args_os().skip(1);
    let mut native_smoke = false;
    let mut output = None;
    while let Some(argument) = args.next() {
        match argument.to_str() {
            Some("--native-smoke") => native_smoke = true,
            Some("--ppm") => output = Some(PathBuf::from(args.next().ok_or("missing PPM path")?)),
            _ => return Err("usage: geometry-gallery [--native-smoke] [--ppm PATH]".into()),
        }
    }
    let mut gallery = scene::render(Size::new(900, 540))?;
    if native_smoke {
        gallery = present(gallery)?;
    }
    if let Some(path) = output {
        export::write_ppm(&path, &gallery.raster)?;
    }
    for line in &gallery.report {
        println!("{line}");
    }
    let (x, y, width, height) = gallery.ime_caret;
    println!("ime_area={x},{y},{width},{height}");
    println!(
        "viewport={}x{} rgba_bytes={} checksum={:016x} native_presented={native_smoke}",
        gallery.raster.size().width(),
        gallery.raster.size().height(),
        gallery.raster.bytes().len(),
        checksum(gallery.raster.bytes()),
    );
    Ok(())
}

#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
#[path = "geometry_gallery/native.rs"]
mod native;

#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
fn present(gallery: scene::Gallery) -> Result<scene::Gallery, GalleryError> {
    native::present(gallery)
}

#[cfg(not(all(feature = "native", any(target_os = "linux", target_os = "windows"))))]
fn present(_gallery: scene::Gallery) -> Result<scene::Gallery, GalleryError> {
    Err("native smoke requires --features native on Windows or Linux Wayland".into())
}
