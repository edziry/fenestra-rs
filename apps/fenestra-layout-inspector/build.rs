use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use fenestra_ui_authoring::prototype::{
    FenSourceV2, REFERENCE_AUTHORING_LIMITS_V2, canonical_rust_v2, compile_fen_v2,
};
use fenestra_ui_ir::prototype::SourceId;
use fenestra_ui_macros::ui;

const FIXTURE: &str = "../../probes/exp-0007-typed-authoring/fixtures/hybrid-spatial-v2.fen";
const UI_FIXTURE: &str = "../../probes/exp-0007-typed-authoring/fixtures/hybrid-spatial-v2.ui";
const OUTPUT: &str = "layout_inspector_fen_v2.rs";
const EXAMPLE: &str = "../../examples/hello-panel.fen";
const UI_EXAMPLE: &str = "../../examples/hello-panel.ui";
const EXAMPLE_OUTPUT: &str = "hello_panel_fen_v2.rs";
const SOURCE: SourceId = SourceId::new(15);

type RawPrograms = (
    fenestra_ui_ir::prototype::SchemaManifest,
    fenestra_ui_ir::prototype::ConstructionProgram,
    fenestra_ui_ir::prototype::StyleProgram,
    fenestra_ui_ir::prototype::SpatialProgramV2,
);

fn main() -> ExitCode {
    println!("cargo::rerun-if-changed={FIXTURE}");
    println!("cargo::rerun-if-changed={UI_FIXTURE}");
    println!("cargo::rerun-if-changed={EXAMPLE}");
    println!("cargo::rerun-if-changed={UI_EXAMPLE}");
    match generate() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => {
            println!("cargo::error=layout-inspector-authoring-generation-failed");
            ExitCode::FAILURE
        }
    }
}

fn generate() -> Result<(), ()> {
    generate_program(FIXTURE, OUTPUT, &ui_programs())?;
    generate_program(EXAMPLE, EXAMPLE_OUTPUT, &example_ui_programs())
}

fn generate_program(input: &str, output: &str, ui: &RawPrograms) -> Result<(), ()> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").ok_or(())?;
    let output_dir = env::var_os("OUT_DIR").ok_or(())?;
    let source = PathBuf::from(manifest_dir).join(input);
    let bytes = fs::read(source).map_err(|_| ())?;
    let compiled = compile_fen_v2(
        FenSourceV2::new(SOURCE, &bytes),
        REFERENCE_AUTHORING_LIMITS_V2,
    )
    .map_err(|_| ())?;
    let generated = canonical_rust_v2(&compiled, REFERENCE_AUTHORING_LIMITS_V2).map_err(|_| ())?;
    if !same_programs(&compiled, ui) {
        return Err(());
    }
    fs::write(PathBuf::from(output_dir).join(output), generated.as_str()).map_err(|_| ())
}

fn ui_programs() -> RawPrograms {
    include!("../../probes/exp-0007-typed-authoring/fixtures/hybrid-spatial-v2.ui")
}

fn example_ui_programs() -> RawPrograms {
    include!("../../examples/hello-panel.ui")
}

fn same_programs(
    fen: &fenestra_ui_authoring::prototype::CompiledAuthoringV2,
    ui: &RawPrograms,
) -> bool {
    fen.schema() == &ui.0
        && fen.construction() == &ui.1
        && fen.style() == &ui.2
        && fen.spatial() == &ui.3
}
