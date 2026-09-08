use std::io::{self, Write};
use std::path::PathBuf;

use fenestra_ui::native::{WindowContent, WindowOptions};
use fenestra_ui::{Raster, TextBuffer};

#[path = "text-pad/mod.rs"]
mod text_pad;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut native = None;
    let mut ppm = None;
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--native" | "--native-smoke" if native.is_none() => {
                native = Some(argument == "--native-smoke");
            }
            "--ppm" if ppm.is_none() => {
                ppm = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| invalid("--ppm requires a path"))?,
                ));
            }
            "--help" => {
                println!("No arguments: deterministic headless editing exercise.");
                println!("--native: interactive text pad. --native-smoke: present one frame.");
                println!("--ppm PATH: explicitly save the final raster as a PPM image.");
                return Ok(());
            }
            _ => {
                return Err(
                    invalid("expected --native, --native-smoke, --ppm PATH, or --help").into(),
                );
            }
        }
    }
    let text = TextBuffer::new(text_pad::SAMPLE, text_pad::BYTE_LIMIT)?;
    let mut pad = text_pad::TextPad::new(text, text_pad::INITIAL_SIZE)?;
    if let Some(smoke) = native {
        fenestra_ui::native::run(
            &mut pad,
            WindowOptions::new("Fenestra text pad")
                .size(
                    text_pad::INITIAL_SIZE.width(),
                    text_pad::INITIAL_SIZE.height(),
                )
                .ime_allowed(true)
                .smoke(smoke),
        )?;
    } else {
        pad.exercise()?;
    }
    let raster = pad.frame()?;
    if let Some(path) = ppm {
        write_ppm(path, &raster)?;
    }
    println!("{}", pad.summary(&raster));
    Ok(())
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn write_ppm(path: PathBuf, raster: &Raster) -> io::Result<()> {
    let mut file = io::BufWriter::new(std::fs::File::create(path)?);
    write!(
        file,
        "P6\n{} {}\n255\n",
        raster.size().width(),
        raster.size().height()
    )?;
    for pixel in raster.bytes().chunks_exact(4) {
        file.write_all(&pixel[..3])?;
    }
    file.flush()
}
