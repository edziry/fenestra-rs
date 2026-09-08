use std::env;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use super::{Diagnostic, Limits, compile_fen};

/// Failure to read, compile, or write a view from a Cargo build script.
#[derive(Debug)]
pub struct BuildError {
    kind: Failure,
}

#[derive(Debug)]
enum Failure {
    Environment(&'static str),
    OutputName,
    Read(PathBuf, io::Error),
    Write(PathBuf, io::Error),
    Compile {
        path: PathBuf,
        diagnostic: Diagnostic,
        line: usize,
        column: usize,
    },
}

/// Compiles a `.fen` view into a Rust expression in Cargo's `OUT_DIR`.
///
/// Relative input paths resolve from `CARGO_MANIFEST_DIR`. The output name must
/// be one file name. This function prints a `cargo::rerun-if-changed` directive.
/// Include the generated expression from `OUT_DIR` in the target application.
///
/// # Errors
/// Reports missing Cargo environment, invalid output names, I/O failures, or
/// authoring diagnostics with a file path, one-based line, and byte column.
pub fn build_file(input: impl AsRef<Path>, output_name: &str) -> Result<(), BuildError> {
    if !single_file_name(output_name) {
        return Err(BuildError {
            kind: Failure::OutputName,
        });
    }
    let manifest = cargo_directory("CARGO_MANIFEST_DIR")?;
    let output = cargo_directory("OUT_DIR")?.join(output_name);
    let input = manifest.join(input);
    println!("cargo::rerun-if-changed={}", input.display());
    let mut bytes = Vec::new();
    let maximum = Limits::default().source_bytes as u64;
    File::open(&input)
        .and_then(|file| file.take(maximum + 1).read_to_end(&mut bytes))
        .map_err(|error| BuildError {
            kind: Failure::Read(input.clone(), error),
        })?;
    let compiled = compile_fen(&bytes).map_err(|diagnostic| {
        let start = diagnostic.byte_range().map_or(0, |(start, _)| start);
        let prefix = &bytes[..start.min(bytes.len())];
        let line = prefix.iter().filter(|&&byte| byte == b'\n').count() + 1;
        let column = prefix
            .iter()
            .rposition(|&byte| byte == b'\n')
            .map_or(prefix.len() + 1, |newline| prefix.len() - newline);
        BuildError {
            kind: Failure::Compile {
                path: input,
                diagnostic,
                line,
                column,
            },
        }
    })?;
    fs::write(&output, compiled.rust_source()).map_err(|error| BuildError {
        kind: Failure::Write(output, error),
    })
}

fn single_file_name(name: &str) -> bool {
    let mut components = Path::new(name).components();
    matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none()
        && !name.contains(['\n', '\r', '/', '\\'])
}

fn cargo_directory(name: &'static str) -> Result<PathBuf, BuildError> {
    env::var_os(name).map(PathBuf::from).ok_or(BuildError {
        kind: Failure::Environment(name),
    })
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Failure::Environment(name) => write!(formatter, "missing Cargo environment: {name}"),
            Failure::OutputName => formatter.write_str("output_name must be one file name"),
            Failure::Read(path, error) => {
                write!(formatter, "{}: cannot read source: {error}", path.display())
            }
            Failure::Write(path, error) => write!(
                formatter,
                "{}: cannot write generated Rust: {error}",
                path.display()
            ),
            Failure::Compile {
                path,
                diagnostic,
                line,
                column,
            } => {
                write!(
                    formatter,
                    "{}:{line}:{column}: error: {diagnostic}",
                    path.display()
                )
            }
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            Failure::Read(_, error) | Failure::Write(_, error) => Some(error),
            Failure::Compile { diagnostic, .. } => Some(diagnostic),
            Failure::Environment(_) | Failure::OutputName => None,
        }
    }
}
