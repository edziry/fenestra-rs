#![forbid(unsafe_code)]

//! Release entry point for the WU-0014 interactive GPU probe.

use std::fs::OpenOptions;
use std::io::Write;
use std::process::ExitCode;

use fenestra_ui_exp_0014_windows_gpu::{
    InteractiveResultV1, parse_probe_cli_v1, run_interactive_probe_v1,
    verify_interactive_artifact_v1,
};

fn main() -> ExitCode {
    let Ok(cli) = parse_probe_cli_v1(std::env::args_os()) else {
        return ExitCode::FAILURE;
    };
    let Ok(bytes) = run_interactive_probe_v1() else {
        return ExitCode::FAILURE;
    };
    let Ok(verified) = verify_interactive_artifact_v1(&bytes) else {
        return ExitCode::FAILURE;
    };
    let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(cli.artifact_path())
    else {
        return ExitCode::FAILURE;
    };
    if file.write_all(&bytes).and_then(|()| file.flush()).is_err() {
        return ExitCode::FAILURE;
    }
    if verified.result() == InteractiveResultV1::Pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
