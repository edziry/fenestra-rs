use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use fenestra_ui_authoring::prototype::{
    FenSourceV2, REFERENCE_AUTHORING_LIMITS_V2, canonical_rust_v2, compile_fen_v2,
};
use fenestra_ui_ir::prototype::SourceId;

const FIXTURE: &str = "../../probes/exp-0007-typed-authoring/fixtures/hybrid-spatial-v2.fen";
const OUTPUT: &str = "layout_inspector_fen_v2.rs";
const SOURCE: SourceId = SourceId::new(15);

fn main() -> ExitCode {
    println!("cargo::rerun-if-changed={FIXTURE}");
    match generate() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => {
            println!("cargo::error=layout-inspector-authoring-generation-failed");
            ExitCode::FAILURE
        }
    }
}

fn generate() -> Result<(), ()> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").ok_or(())?;
    let output_dir = env::var_os("OUT_DIR").ok_or(())?;
    let source = PathBuf::from(manifest_dir).join(FIXTURE);
    let bytes = fs::read(source).map_err(|_| ())?;
    let compiled = compile_fen_v2(
        FenSourceV2::new(SOURCE, &bytes),
        REFERENCE_AUTHORING_LIMITS_V2,
    )
    .map_err(|_| ())?;
    let generated = canonical_rust_v2(&compiled, REFERENCE_AUTHORING_LIMITS_V2).map_err(|_| ())?;
    fs::write(PathBuf::from(output_dir).join(OUTPUT), generated.as_str()).map_err(|_| ())
}
