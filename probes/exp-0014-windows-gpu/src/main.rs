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
    let cli = match parse_probe_cli_v1(std::env::args_os()) {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("probe-cli-error={error:?}");
            return ExitCode::FAILURE;
        }
    };
    let bytes = match run_interactive_probe_v1() {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("interactive-probe-error={error:?}");
            return ExitCode::FAILURE;
        }
    };
    let verified = match verify_interactive_artifact_v1(&bytes) {
        Ok(verified) => verified,
        Err(error) => {
            eprintln!("artifact-verification-error={error:?}");
            return ExitCode::FAILURE;
        }
    };
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(cli.artifact_path())
    {
        Ok(file) => file,
        Err(_) => {
            eprintln!("artifact-open-error");
            return ExitCode::FAILURE;
        }
    };
    if file.write_all(&bytes).and_then(|()| file.flush()).is_err() {
        eprintln!("artifact-write-error");
        return ExitCode::FAILURE;
    }
    if verified.result() == InteractiveResultV1::Pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
