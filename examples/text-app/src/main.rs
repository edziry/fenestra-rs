use fenestra_text_app::{application, checksum, update_headless};
use fenestra_ui::Application;

mod arguments;
mod export;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = arguments::Arguments::parse(std::env::args_os().skip(1))?;
    if arguments.help {
        println!("Run without arguments for deterministic text editing and raster output.");
        println!("With the native feature, use --native or --native-smoke.");
        println!("--ppm PATH exports the final raster's RGB channels as a PPM image.");
        return Ok(());
    }
    let mut app = application()?;
    if let Some(smoke) = arguments.native {
        app = run_native(app, smoke)?;
    } else {
        update_headless(&mut app)?;
    }
    let metrics = app.text_metrics("content")?;
    let raster = app.raster()?;
    if let Some(path) = arguments.ppm {
        export::write_ppm(&path, &raster)?;
    }
    println!(
        "generation={} nodes={} text_bytes={} lines={} glyphs={} missing={} rgba_bytes={} checksum={:016x}",
        app.generation(),
        app.node_count(),
        app.text("content")?.len(),
        metrics.lines(),
        metrics.glyphs(),
        metrics.missing_glyphs(),
        raster.bytes().len(),
        checksum(raster.bytes()),
    );
    Ok(())
}

#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
fn run_native(app: Application, smoke: bool) -> Result<Application, Box<dyn std::error::Error>> {
    fenestra_text_app::native::run(app, smoke)
}

#[cfg(not(all(feature = "native", any(target_os = "linux", target_os = "windows"))))]
fn run_native(_app: Application, _smoke: bool) -> Result<Application, Box<dyn std::error::Error>> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "native windows require --features native on Windows or Linux Wayland",
    )
    .into())
}
