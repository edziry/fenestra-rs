#![forbid(unsafe_code)]

//! Standalone verifier for one WU-0015 native evidence artifact.

use std::env;
use std::process::ExitCode;

use fenestra_layout_inspector::evidence::{EvidenceResult, verify_artifact};

fn main() -> ExitCode {
    let mut arguments = env::args_os();
    arguments.next();
    let Some(path) = arguments.next() else {
        eprintln!("usage: fenestra-layout-inspector-verify <artifact-path>");
        return ExitCode::FAILURE;
    };
    if arguments.next().is_some() {
        eprintln!("usage: fenestra-layout-inspector-verify <artifact-path>");
        return ExitCode::FAILURE;
    }
    let Ok(bytes) = std::fs::read(path) else {
        eprintln!("artifact read failed");
        return ExitCode::FAILURE;
    };
    let Ok(verified) = verify_artifact(&bytes) else {
        eprintln!("artifact verification failed");
        return ExitCode::FAILURE;
    };
    if verified.result() != EvidenceResult::Pass {
        eprintln!("artifact result is not pass");
        return ExitCode::FAILURE;
    }
    println!(
        "pass|records={}|bytes={}|generation={}",
        verified.record_count(),
        verified.byte_count(),
        verified
            .final_generation()
            .map_or_else(|| "none".to_owned(), |generation| generation.to_string())
    );
    ExitCode::SUCCESS
}
