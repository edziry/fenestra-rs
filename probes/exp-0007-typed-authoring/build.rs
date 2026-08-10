use std::env;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use fenestra_ui_authoring::prototype::{
    AuthoringDiagnosticV1, DiagnosticLocationV1, FenSourceV1, REFERENCE_AUTHORING_LIMITS_V1,
    canonical_rust_v1, compile_fen_v1,
};
use fenestra_ui_ir::prototype::SourceId;

const FIXTURE_LABEL: &str = "fixtures/layout-board.fen";
const OUTPUT_NAME: &str = "layout_board_fen_v1.rs";
const SOURCE: SourceId = SourceId::new(7);

fn main() -> ExitCode {
    println!("cargo::rerun-if-changed={FIXTURE_LABEL}");
    match generate() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            println!("cargo::error={error}");
            ExitCode::FAILURE
        }
    }
}

fn generate() -> Result<(), BuildFailure> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").ok_or(BuildFailure::Environment)?;
    let output_dir = env::var_os("OUT_DIR").ok_or(BuildFailure::Environment)?;
    let fixture = PathBuf::from(manifest_dir).join(FIXTURE_LABEL);
    let bytes = fs::read(fixture).map_err(|_| BuildFailure::Read)?;
    let compiled = compile_fen_v1(
        FenSourceV1::new(SOURCE, &bytes),
        REFERENCE_AUTHORING_LIMITS_V1,
    )
    .map_err(BuildFailure::Authoring)?;
    let generated = canonical_rust_v1(&compiled, REFERENCE_AUTHORING_LIMITS_V1)
        .map_err(BuildFailure::Authoring)?;
    let output = PathBuf::from(output_dir).join(OUTPUT_NAME);
    fs::write(output, generated.as_str()).map_err(|_| BuildFailure::Write)
}

enum BuildFailure {
    Environment,
    Read,
    Authoring(AuthoringDiagnosticV1),
    Write,
}

impl fmt::Display for BuildFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Environment => formatter.write_str("typed-authoring-build-environment"),
            Self::Read => write!(formatter, "{FIXTURE_LABEL}:0..0:read-failed"),
            Self::Authoring(error) => write_authoring_failure(formatter, error),
            Self::Write => formatter.write_str("typed-authoring-build-write"),
        }
    }
}

fn write_authoring_failure(
    formatter: &mut fmt::Formatter<'_>,
    error: &AuthoringDiagnosticV1,
) -> fmt::Result {
    let physical = match error.location() {
        DiagnosticLocationV1::Physical(origin)
        | DiagnosticLocationV1::Anchored {
            physical: origin, ..
        } => origin,
    };
    match physical.fen_byte_range() {
        Some((start, end)) => write!(formatter, "{FIXTURE_LABEL}:{start}..{end}:{error}"),
        None => formatter.write_str("typed-authoring-build-origin"),
    }
}
