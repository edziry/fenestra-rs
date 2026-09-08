use fenestra_text_app::{application, checksum, update_headless};
use fenestra_ui::Application;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.as_slice() == ["--help"] {
        println!("Run without arguments for deterministic text editing and raster output.");
        println!("With the native feature, use --native or --native-smoke.");
        return Ok(());
    }
    let mut app = application()?;
    match arguments.as_slice() {
        [] => update_headless(&mut app)?,
        [option] if option == "--native" => app = run_native(app, false)?,
        [option] if option == "--native-smoke" => app = run_native(app, true)?,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "expected no arguments, --native, --native-smoke, or --help",
            )
            .into());
        }
    }
    let metrics = app.text_metrics("content")?;
    let raster = app.raster()?;
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
