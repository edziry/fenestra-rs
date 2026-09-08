use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

#[derive(Default)]
pub(super) struct Arguments {
    pub(super) native: Option<bool>,
    pub(super) headless: bool,
    pub(super) ppm: Option<PathBuf>,
    pub(super) help: bool,
}

impl Arguments {
    pub(super) fn parse(input: impl Iterator<Item = OsString>) -> io::Result<Self> {
        let mut result = Self::default();
        let mut args = input;
        while let Some(argument) = args.next() {
            match argument.to_str() {
                Some("--native" | "--native-smoke")
                    if result.native.is_none() && !result.headless =>
                {
                    result.native = Some(argument == "--native-smoke");
                }
                Some("--headless") if result.native.is_none() && !result.headless => {
                    result.headless = true;
                }
                Some("--ppm") if result.ppm.is_none() => {
                    result.ppm = Some(PathBuf::from(args.next().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "--ppm requires a path")
                    })?));
                }
                Some("--help") => {
                    result.help = true;
                    return Ok(result);
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "expected --headless, --native, --native-smoke, --ppm PATH, or --help; options cannot repeat",
                    ));
                }
            }
        }
        Ok(result)
    }
}
