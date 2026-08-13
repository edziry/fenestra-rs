#![forbid(unsafe_code)]

//! Standalone verifier for one WU-0014 interaction artifact.

use std::process::ExitCode;

use fenestra_ui_exp_0014_windows_gpu::{
    InteractiveResultV1, parse_probe_cli_v1, verify_interactive_artifact_v1,
};

fn main() -> ExitCode {
    let Ok(cli) = parse_probe_cli_v1(std::env::args_os()) else {
        eprintln!("usage: fenestra-wu-0014-verify <artifact-path>");
        return ExitCode::FAILURE;
    };
    let Ok(bytes) = std::fs::read(cli.artifact_path()) else {
        eprintln!("artifact read failed");
        return ExitCode::FAILURE;
    };
    let Ok(verified) = verify_interactive_artifact_v1(&bytes) else {
        eprintln!("artifact verification failed");
        return ExitCode::FAILURE;
    };
    if verified.result() != InteractiveResultV1::Pass {
        eprintln!("artifact result is not pass");
        return ExitCode::FAILURE;
    }
    let Some(generation) = verified.last_generation() else {
        eprintln!("artifact verification failed");
        return ExitCode::FAILURE;
    };
    println!(
        "pass|records={}|bytes={}|generation={generation}",
        verified.record_count(),
        verified.byte_count(),
    );
    ExitCode::SUCCESS
}
