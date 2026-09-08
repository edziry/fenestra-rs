use fenestra_controls_app::{DemoState, application, checksum, exercise, summary};
use fenestra_ui::Application;

mod arguments;
mod export;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = arguments::Arguments::parse(std::env::args_os().skip(1))?;
    if arguments.help {
        println!("No arguments or --headless: exercise controls with keyboard input.");
        println!("With the native feature, use --native or --native-smoke.");
        println!("--ppm PATH exports the final raster's RGB channels as a PPM image.");
        return Ok(());
    }
    let mut app = application()?;
    let mut state = DemoState::default();
    if let Some(smoke) = arguments.native {
        app = run_native(app, &mut state, smoke)?;
        println!("{}", summary("native", &app, &state)?);
    } else {
        for checkpoint in exercise(&mut app, &mut state)? {
            println!("{checkpoint}");
        }
    }
    let raster = app.raster()?;
    if let Some(path) = arguments.ppm {
        export::write_ppm(&path, &raster)?;
    }
    println!(
        "generation={} nodes={} rgba_bytes={} checksum={:016x}",
        app.generation(),
        app.node_count(),
        raster.bytes().len(),
        checksum(raster.bytes()),
    );
    Ok(())
}

#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
fn run_native(
    app: Application,
    state: &mut DemoState,
    smoke: bool,
) -> Result<Application, Box<dyn std::error::Error>> {
    use fenestra_ui::native::WindowOptions;

    let options = WindowOptions::new("Fenestra control preferences")
        .size(640, 520)
        .smoke(smoke);
    Ok(app.run(options, |app, event| state.event(app, &event))?)
}

#[cfg(not(all(feature = "native", any(target_os = "linux", target_os = "windows"))))]
fn run_native(
    _app: Application,
    _state: &mut DemoState,
    _smoke: bool,
) -> Result<Application, Box<dyn std::error::Error>> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "native windows require --features native on Windows or Linux Wayland",
    )
    .into())
}
