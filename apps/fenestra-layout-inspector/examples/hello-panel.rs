#![forbid(unsafe_code)]

//! Runs the minimal authored card panel headlessly or in a native window.

use std::{env, process::ExitCode};

use fenestra_layout_inspector::{InspectorAction, InspectorErrorKind, LayoutInspector};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("hello-panel-error={error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    if arguments.len() > 1 {
        return Err("usage: hello-panel [--native | --native-smoke | --help]".into());
    }
    let mode = arguments.first().map(|value| value.to_str());
    if mode == Some(Some("--help")) {
        println!("usage: hello-panel [--native | --native-smoke | --help]");
        println!("Default: deterministic headless selection, insertion, and resize.");
        println!("Native: click a blue card to select it; press Space to add a card.");
        return Ok(());
    }
    if !matches!(mode, None | Some(Some("--native" | "--native-smoke"))) {
        return Err("usage: hello-panel [--native | --native-smoke | --help]".into());
    }

    let inspector = LayoutInspector::from_programs(include!(concat!(
        env!("OUT_DIR"),
        "/hello_panel_fen_v2.rs"
    )))
    .map_err(|error| format!("{error:?}"))?;
    match mode {
        None => run_headless(inspector).map_err(|error| format!("{error:?}")),
        Some(Some("--native")) => run_native(inspector, false),
        Some(Some("--native-smoke")) => run_native(inspector, true),
        _ => unreachable!("arguments were validated before constructing the panel"),
    }
}

fn run_headless(mut inspector: LayoutInspector) -> Result<(), InspectorErrorKind> {
    let initial = inspector.observe()?;
    inspector.dispatch(InspectorAction::PointerMove { x: 20, y: 20 })?;
    inspector.dispatch(InspectorAction::PointerPress)?;
    inspector.dispatch(InspectorAction::InsertTile { key: 30 })?;
    inspector.dispatch(InspectorAction::Resize {
        width: 224,
        height: 160,
    })?;
    let diagnostics = inspector.diagnostics()?;
    let frame = diagnostics.frame();
    println!(
        "hello-panel|initial-generation={}|final-generation={}|nodes={}|keys={:?}|viewport={}x{}|paints={}|hits={}|selected-tone={:?}",
        initial.generation(),
        frame.generation(),
        frame.node_count(),
        frame.keyed_keys(),
        frame.viewport().width(),
        frame.viewport().height(),
        frame.paint_count(),
        frame.hit_count(),
        diagnostics.selected_tone(),
    );
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn run_native(inspector: LayoutInspector, smoke: bool) -> Result<(), String> {
    use fenestra_layout_inspector::native::{
        run_native_smoke_with_inspector, run_native_with_inspector,
    };

    let result = if smoke {
        run_native_smoke_with_inspector(inspector)
    } else {
        run_native_with_inspector(inspector)
    };
    result.map_err(|error| format!("{error:?}"))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn run_native(_inspector: LayoutInspector, _smoke: bool) -> Result<(), String> {
    Err("native presentation is available only on the Windows and Linux adapters".into())
}
