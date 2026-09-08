#![forbid(unsafe_code)]

//! Deterministic command-line smoke run for the layout inspector.

use std::process::ExitCode;

use fenestra_layout_inspector::{InspectorAction, LayoutInspector};

fn main() -> ExitCode {
    let result = (|| {
        let mut inspector = LayoutInspector::new()?;
        let initial = inspector.observe()?;
        inspector.dispatch(InspectorAction::PointerMove { x: 4, y: 3 })?;
        inspector.dispatch(InspectorAction::PointerPress)?;
        inspector.dispatch(InspectorAction::InsertTile { key: 30 })?;
        inspector.dispatch(InspectorAction::Resize {
            width: 224,
            height: 160,
        })?;
        let final_frame = inspector.observe()?;
        let diagnostics = inspector.diagnostics()?;
        println!(
            "fenestra-layout-inspector|initial-generation={}|final-generation={}|nodes={}|keys={:?}|viewport={}x{}|selected={}|fen-bytes={:?}|ui-bytes={:?}|generated-bytes={:?}|frontends-equivalent={}|selected-tone={:?}",
            initial.generation(),
            final_frame.generation(),
            final_frame.node_count(),
            final_frame.keyed_keys(),
            final_frame.viewport().width(),
            final_frame.viewport().height(),
            final_frame.has_selection(),
            diagnostics.fen_source_bytes(),
            diagnostics.ui_source_bytes(),
            diagnostics.generated_rust_bytes(),
            diagnostics.frontends_equivalent(),
            diagnostics.selected_tone(),
        );
        Ok::<(), fenestra_layout_inspector::InspectorErrorKind>(())
    })();

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("fenestra-layout-inspector-error={error:?}");
            ExitCode::FAILURE
        }
    }
}
