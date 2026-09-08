use fenestra_ui::{Application, Color, Error, Size, View};

const SELECTED: Color = Color::rgba8(240, 176, 64, 255);

fn panel() -> View {
    include!(concat!(env!("OUT_DIR"), "/panel.rs"))
}

fn update_headless(mut app: Application) -> Result<Application, Error> {
    app.set_background("primary", SELECTED)?;
    app.set_size("primary", 112, 64)?;
    app.resize(Size::new(360, 220))?;
    Ok(app)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.as_slice() == ["--help"] {
        println!("Run without arguments for a deterministic headless example.");
        println!("With the native feature, use --native or --native-smoke.");
        return Ok(());
    }
    let app = Application::new(panel(), Size::new(320, 192))?;
    let app = match arguments.as_slice() {
        [] => update_headless(app)?,
        [option] if option == "--native" => run_native(app, false)?,
        [option] if option == "--native-smoke" => run_native(app, true)?,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "expected no arguments, --native, --native-smoke, or --help",
            )
            .into());
        }
    };
    let raster = app.raster()?;
    println!(
        "generation={} nodes={} viewport={}x{} rgba_bytes={} hit(100,16)={:?}",
        app.generation(),
        app.node_count(),
        app.size().width(),
        app.size().height(),
        raster.bytes().len(),
        app.hit_test(100, 16)
    );
    Ok(())
}

#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
fn run_native(app: Application, smoke: bool) -> Result<Application, Box<dyn std::error::Error>> {
    use fenestra_ui::{Event, native::WindowOptions};

    let mut primary_selected = false;
    let mut secondary_selected = false;
    let options = WindowOptions::new("Fenestra typed application")
        .size(320, 192)
        .smoke(smoke);
    Ok(app.run(options, move |app, event| {
        if let Event::Click { target: Some(name) } = event {
            let (selected, original) = if name == "primary" {
                (&mut primary_selected, Color::rgba8(48, 128, 192, 255))
            } else {
                (&mut secondary_selected, Color::rgba8(88, 168, 112, 255))
            };
            *selected = !*selected;
            app.set_background(&name, if *selected { SELECTED } else { original })?;
        }
        Ok(())
    })?)
}

#[cfg(not(all(feature = "native", any(target_os = "linux", target_os = "windows"))))]
fn run_native(_app: Application, _smoke: bool) -> Result<Application, Box<dyn std::error::Error>> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "native windows require --features native on Windows or Linux Wayland",
    )
    .into())
}

#[cfg(test)]
mod tests;
