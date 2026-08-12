use std::env;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use fenestra_ui_authoring::prototype::{
    AuthoringDiagnosticV1, AuthoringDiagnosticV2, DiagnosticLocationV1, DiagnosticLocationV2,
    FenSourceV1, FenSourceV2, REFERENCE_AUTHORING_LIMITS_V1, REFERENCE_AUTHORING_LIMITS_V2,
    canonical_rust_v1, canonical_rust_v2, compile_fen_v1, compile_fen_v2,
};
use fenestra_ui_ir::prototype::SourceId;

const FIXTURE_LABEL: &str = "fixtures/layout-board.fen";
const OUTPUT_NAME: &str = "layout_board_fen_v1.rs";
const SOURCE: SourceId = SourceId::new(7);
const SPATIAL_FIXTURE_LABEL: &str = "fixtures/hybrid-spatial-v2.fen";
const SPATIAL_OUTPUT_NAME: &str = "hybrid_spatial_fen_v2.rs";
const SPATIAL_SOURCE: SourceId = SourceId::new(13);

fn main() -> ExitCode {
    println!("cargo::rerun-if-changed={FIXTURE_LABEL}");
    println!("cargo::rerun-if-changed={SPATIAL_FIXTURE_LABEL}");
    match generate() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            println!("cargo::error={error}");
            ExitCode::FAILURE
        }
    }
}

fn generate() -> Result<(), BuildFailure> {
    generate_v1()?;
    generate_v2()
}

fn generate_v1() -> Result<(), BuildFailure> {
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

fn generate_v2() -> Result<(), BuildFailure> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").ok_or(BuildFailure::Environment)?;
    let output_dir = env::var_os("OUT_DIR").ok_or(BuildFailure::Environment)?;
    let fixture = PathBuf::from(manifest_dir).join(SPATIAL_FIXTURE_LABEL);
    let bytes = fs::read(fixture).map_err(|_| BuildFailure::SpatialRead)?;
    let compiled = compile_fen_v2(
        FenSourceV2::new(SPATIAL_SOURCE, &bytes),
        REFERENCE_AUTHORING_LIMITS_V2,
    )
    .map_err(BuildFailure::SpatialAuthoring)?;
    let generated = canonical_rust_v2(&compiled, REFERENCE_AUTHORING_LIMITS_V2)
        .map_err(BuildFailure::SpatialAuthoring)?;
    let output = PathBuf::from(output_dir).join(SPATIAL_OUTPUT_NAME);
    fs::write(output, generated.as_str()).map_err(|_| BuildFailure::SpatialWrite)
}

enum BuildFailure {
    Environment,
    Read,
    Authoring(AuthoringDiagnosticV1),
    Write,
    SpatialRead,
    SpatialAuthoring(AuthoringDiagnosticV2),
    SpatialWrite,
}

impl fmt::Display for BuildFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Environment => formatter.write_str("typed-authoring-build-environment"),
            Self::Read => write!(formatter, "{FIXTURE_LABEL}:0..0:read-failed"),
            Self::Authoring(error) => write_authoring_failure(formatter, error),
            Self::Write => formatter.write_str("typed-authoring-build-write"),
            Self::SpatialRead => write!(formatter, "{SPATIAL_FIXTURE_LABEL}:0..0:read-failed"),
            Self::SpatialAuthoring(error) => write_spatial_authoring_failure(formatter, error),
            Self::SpatialWrite => formatter.write_str("typed-authoring-spatial-build-write"),
        }
    }
}

fn write_spatial_authoring_failure(
    formatter: &mut fmt::Formatter<'_>,
    error: &AuthoringDiagnosticV2,
) -> fmt::Result {
    let physical = match error.location() {
        DiagnosticLocationV2::Physical(origin)
        | DiagnosticLocationV2::Anchored {
            physical: origin, ..
        } => origin,
    };
    match physical.fen_byte_range() {
        Some((start, end)) => write!(formatter, "{SPATIAL_FIXTURE_LABEL}:{start}..{end}:{error}"),
        None => formatter.write_str("typed-authoring-spatial-build-origin"),
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
