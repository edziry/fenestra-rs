#![forbid(unsafe_code)]

//! Interactive native entry point for the layout inspector.

use std::process::ExitCode;
use std::{env, fs, path::PathBuf};

use fenestra_layout_inspector::native::{
    NativeInspectorError, run_native, run_native_artifact, run_native_smoke,
};

fn main() -> ExitCode {
    let artifact = artifact_path();
    let result = if artifact.is_some() {
        run_native_artifact().and_then(|bytes| {
            fs::write(artifact.expect("artifact path was checked"), bytes)
                .map_err(|_| NativeInspectorError::Presenter)
        })
    } else if env::args_os().any(|argument| argument == "--smoke") {
        run_native_smoke()
    } else {
        run_native()
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(NativeInspectorError::Application(error)) => {
            eprintln!("fenestra-layout-inspector-native-error={error:?}");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("fenestra-layout-inspector-native-error={error:?}");
            ExitCode::FAILURE
        }
    }
}

fn artifact_path() -> Option<PathBuf> {
    let mut arguments = env::args_os();
    arguments.next();
    arguments.find_map(|argument| {
        argument
            .to_str()
            .and_then(|argument| argument.strip_prefix("--artifact="))
            .map(PathBuf::from)
    })
}
