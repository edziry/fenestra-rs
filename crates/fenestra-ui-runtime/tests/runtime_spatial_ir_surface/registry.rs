use std::collections::BTreeSet;

use super::source::{all_source, read, source_dir};

const EXPORTS: [&str; 76] = [
    "CallbackFinish",
    "CallbackScope",
    "CapacityKind",
    "CommitReceipt",
    "CommittedRuntimeSnapshot",
    "CompletionWatermark",
    "ComputedStyleView",
    "ControlAdmission",
    "ControlSequence",
    "FragmentId",
    "FrameId",
    "FrameWork",
    "HeadlessGeometryView",
    "HeadlessHitRegionView",
    "HeadlessPoint",
    "HeadlessProjectionCapacity",
    "HeadlessProjectionErrorKind",
    "HeadlessProjectionLimitKind",
    "HeadlessProjectionSpec",
    "HeadlessProjectionView",
    "HeadlessRect",
    "HeadlessSceneRectangleView",
    "HeadlessSemanticAction",
    "HeadlessSemanticRole",
    "HeadlessSemanticView",
    "HeadlessSurface",
    "HeadlessSurfaceChangeView",
    "KeyInsertView",
    "KeyMoveView",
    "KeyRemoveView",
    "KeyedMemberIter",
    "LogicalTree",
    "ManifestEntry",
    "ManifestIter",
    "MutationIter",
    "MutationRecordView",
    "NestedCallbackScope",
    "NodeId",
    "PropertyChangeView",
    "QueueCapacity",
    "QueueStats",
    "RendererEpoch",
    "RuntimeCapacity",
    "RuntimeGeneration",
    "RuntimeInitializationError",
    "RuntimeInitializationErrorKind",
    "RuntimePaintFrameV2",
    "RuntimeSpatialBuildViewV2",
    "RuntimeSpatialErrorV2",
    "RuntimeSpatialInputV2",
    "RuntimeSpatialIrErrorKindV2",
    "RuntimeSpatialIrErrorV2",
    "RuntimeSpatialProgramV2",
    "RuntimeSpatialViewV2",
    "ScheduledCommit",
    "SchedulerAction",
    "SchedulerCapacity",
    "SchedulerError",
    "SchedulerErrorKind",
    "SchedulerInput",
    "SchedulerInputResult",
    "SchedulerLane",
    "SchedulerState",
    "SchedulerStats",
    "SchedulerTick",
    "SpatialViewportChangeViewV2",
    "SubmissionId",
    "TransactionError",
    "TransactionErrorKind",
    "TreeError",
    "TreeInvariantError",
    "UiRuntime",
    "UiScheduler",
    "UiTransaction",
    "VisualCancelResult",
    "VisualRequestResult",
];

const STRUCTS: [&str; 53] = [
    "CallbackScope",
    "CommitReceipt",
    "CommittedRuntimeSnapshot",
    "CompletionWatermark",
    "ComputedStyleView",
    "ControlSequence",
    "FragmentId",
    "FrameId",
    "FrameWork",
    "HeadlessGeometryView",
    "HeadlessHitRegionView",
    "HeadlessPoint",
    "HeadlessProjectionCapacity",
    "HeadlessProjectionSpec",
    "HeadlessProjectionView",
    "HeadlessRect",
    "HeadlessSceneRectangleView",
    "HeadlessSemanticView",
    "HeadlessSurface",
    "HeadlessSurfaceChangeView",
    "KeyInsertView",
    "KeyMoveView",
    "KeyRemoveView",
    "KeyedMemberIter",
    "LogicalTree",
    "ManifestIter",
    "MutationIter",
    "NestedCallbackScope",
    "NodeId",
    "PropertyChangeView",
    "QueueCapacity",
    "QueueStats",
    "RendererEpoch",
    "RuntimeCapacity",
    "RuntimeGeneration",
    "RuntimeInitializationError",
    "RuntimePaintFrameV2",
    "RuntimeSpatialBuildViewV2",
    "RuntimeSpatialInputV2",
    "RuntimeSpatialIrErrorV2",
    "RuntimeSpatialViewV2",
    "ScheduledCommit",
    "SchedulerCapacity",
    "SchedulerError",
    "SchedulerStats",
    "SchedulerTick",
    "SpatialViewportChangeViewV2",
    "SubmissionId",
    "TransactionError",
    "TreeInvariantError",
    "UiRuntime",
    "UiScheduler",
    "UiTransaction",
];

#[test]
fn runtime_ir_adapter_has_exact_76_name_prototype_registry() {
    let source = read(&source_dir().join("lib.rs"));
    let all_source = all_source();
    for forbidden in ["include!", "#[macro_export]"] {
        assert!(
            !all_source.contains(forbidden),
            "unexpected API form {forbidden}"
        );
    }
    let marker = "pub mod prototype {";
    assert!(source.contains("#[doc(hidden)]\npub mod prototype {"));
    let offset = source.find(marker).expect("prototype module");
    assert!(!source[..offset].lines().any(is_public_line));
    let prototype = &source[offset + marker.len()..source.len() - 2];
    for forbidden in [" as ", "::*"] {
        assert!(
            !prototype.contains(forbidden),
            "unexpected API form {forbidden}"
        );
    }
    assert!(
        prototype
            .lines()
            .filter(|line| is_public_line(line))
            .all(|line| line.trim_start().starts_with("pub use crate::"))
    );
    let mut observed = BTreeSet::new();
    for item in prototype.split("pub use crate::").skip(1) {
        let names = if let Some(start) = item.find("::{") {
            &item[start + 3..item.find("};").expect("grouped reexport")]
        } else {
            let end = item.find(';').expect("singleton reexport");
            item[..end].rsplit("::").next().expect("singleton name")
        };
        for name in names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            assert!(observed.insert(name), "duplicate reexport {name}");
        }
    }
    assert_eq!(observed, EXPORTS.into_iter().collect());
}

#[test]
fn runtime_ir_adapter_has_exact_53_struct_registry() {
    let source = all_source();
    let lines = source.lines().collect::<Vec<_>>();
    let mut observed = BTreeSet::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if !line.starts_with("pub struct ") {
            index += 1;
            continue;
        }
        let name = line["pub struct ".len()..]
            .split(['<', '(', '{', ';'])
            .next()
            .expect("struct name")
            .trim();
        assert!(observed.insert(name.to_owned()), "duplicate struct {name}");
        if line.ends_with(';') {
            assert!(!line.contains("(pub "), "public tuple field on {name}");
            index += 1;
            continue;
        }
        index += 1;
        while index < lines.len() && lines[index].trim() != "}" {
            assert!(
                !lines[index].trim_start().starts_with("pub "),
                "public field: {}",
                lines[index]
            );
            index += 1;
        }
        index += 1;
    }
    assert_eq!(observed, STRUCTS.into_iter().map(str::to_owned).collect());
}

fn is_public_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("pub ") || line.starts_with("pub(")
}
