#![forbid(unsafe_code)]

#[path = "layout_artifact/runtime.rs"]
mod runtime;
#[path = "runtime_candidate/script.rs"]
mod script;
#[path = "runtime_candidate/support.rs"]
mod support;

use fenestra_ui_ir::prototype::InvalidationSet;
use fenestra_ui_runtime::prototype::HeadlessSurface;

use runtime::{
    RuntimeArtifactFaultV1, RuntimeArtifactSliceV1, RuntimeArtifactV1, RuntimeExecutionLaneV1,
    RuntimeGeometryFieldV1, RuntimeHitFieldV1, RuntimeProjectionFieldV1, RuntimeProjectionLaneV1,
    RuntimeReceiptFieldV1, RuntimeSceneFieldV1, RuntimeStepV1, build_runtime_artifact_v1,
    inject_runtime_artifact_fault_v1, runtime_lines_v1,
};
use script::{run_candidate_lane_v1, run_reference_lane_v1};
use support::{
    DIMENSION_INVALIDATION, PAINT_INVALIDATION, REGION_INVALIDATION, RESIZE_INVALIDATION,
};

const NODE_COUNTS: [usize; 7] = [5, 5, 6, 6, 6, 5, 5];
const HIT_COUNTS: [usize; 7] = [3, 3, 4, 4, 4, 3, 3];
const EXPECTED_FAULTS: usize = 1_010;

#[test]
fn runtime_artifact_model_is_complete_and_faults_are_slice_atomic() {
    let reference = run_reference_lane_v1();
    let candidate = run_candidate_lane_v1();
    let model = build_runtime_artifact_v1(reference.transcript(), candidate.transcript());

    assert_eq!(
        model.milestones.len(),
        RuntimeStepV1::ALL.len(),
        "runtime artifact model scaffold must materialize all seven milestones"
    );
    assert_registered_milestones(&model);
    let canonical_lines = runtime_lines_v1(&model);
    assert_runtime_lines(&canonical_lines);
    assert_eq!(canonical_lines, runtime_lines_v1(&model));

    let faults = registered_faults(&model);
    assert_eq!(faults.len(), EXPECTED_FAULTS);
    for fault in faults.iter().copied() {
        let faulted = inject_runtime_artifact_fault_v1(&model, fault);
        let changed_slices = faults
            .iter()
            .copied()
            .filter(|slice| slice_changed(&model, &faulted, *slice))
            .collect::<Vec<_>>();
        assert_eq!(changed_slices, vec![fault]);

        let faulted_lines = runtime_lines_v1(&faulted);
        let changed_rows = canonical_lines
            .iter()
            .zip(&faulted_lines)
            .enumerate()
            .filter_map(|(index, (left, right))| (left != right).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(faulted_lines.len(), canonical_lines.len());
        assert_eq!(changed_rows.len(), 1, "{fault:?}");
        assert!(faulted_lines[changed_rows[0]].starts_with(fault_row_tag(fault)));
    }
}

fn assert_runtime_lines(lines: &[String]) {
    assert_eq!(lines.len(), 109);
    assert_eq!(
        lines.first().map(String::as_str),
        Some(
            "runtime|steps=7|geometry=38|hit=24|scene=38|count-order=computed,geometry,semantics,hit,scene"
        )
    );
    assert_eq!(
        lines.last().map(String::as_str),
        Some("runtime-result|classification=pass")
    );
    assert_eq!(
        lines[1],
        "runtime-generation|step=initial|index=0|reference=receipt:-;projection:0;invalidation:none;mutations:0|candidate=receipt:-;projection:0;invalidation:none;mutations:0|oracle=surface:120,90;counts:5,5,1,3,5|reference=surface:120,90;counts:5,5,1,3,5|candidate=surface:120,90;counts:5,5,1,3,5"
    );
    assert_eq!(
        lines[8],
        "runtime-geometry|step=initial|index=0|record=0|oracle=path:root;bounds:0,0,100,80;clip:0,0,100,80|reference=path:root;bounds:0,0,100,80;clip:0,0,100,80|candidate=path:root;bounds:0,0,100,80;clip:0,0,100,80"
    );
    assert_eq!(
        lines[46],
        "runtime-hit|step=initial|index=0|record=0|oracle=path:root/s:0/s:0;clip:0,0,30,10|reference=path:root/s:0/s:0;clip:0,0,30,10|candidate=path:root/s:0/s:0;clip:0,0,30,10"
    );
    assert_eq!(
        lines[70],
        "runtime-scene|step=initial|index=0|record=0|oracle=path:root;rect:0,0,100,80;rgba:1,1,1,255|reference=path:root;rect:0,0,100,80;rgba:1,1,1,255|candidate=path:root;rect:0,0,100,80;rgba:1,1,1,255"
    );
    for (tag, expected) in [
        ("runtime-generation|", 7),
        ("runtime-geometry|", 38),
        ("runtime-hit|", 24),
        ("runtime-scene|", 38),
    ] {
        assert_eq!(
            lines.iter().filter(|line| line.starts_with(tag)).count(),
            expected
        );
    }
    assert!(lines.iter().all(|line| line.len() <= 512));
    assert!(
        lines
            .iter()
            .all(|line| line.bytes().all(|byte| byte.is_ascii_graphic()))
    );
    let source = lines.join("\n");
    for forbidden in [
        "NodeId",
        "FragmentId",
        "DefaultKey",
        "Debug",
        "/home/",
        "target/",
    ] {
        assert!(!source.contains(forbidden));
    }
}

const fn fault_row_tag(fault: RuntimeArtifactFaultV1) -> &'static str {
    match fault {
        RuntimeArtifactFaultV1::Receipt { .. } | RuntimeArtifactFaultV1::Projection { .. } => {
            "runtime-generation|"
        }
        RuntimeArtifactFaultV1::Geometry { .. } => "runtime-geometry|",
        RuntimeArtifactFaultV1::Hit { .. } => "runtime-hit|",
        RuntimeArtifactFaultV1::Scene { .. } => "runtime-scene|",
    }
}

fn assert_registered_milestones(model: &RuntimeArtifactV1) {
    let invalidations = [
        InvalidationSet::NONE,
        PAINT_INVALIDATION,
        REGION_INVALIDATION,
        REGION_INVALIDATION,
        DIMENSION_INVALIDATION,
        REGION_INVALIDATION,
        RESIZE_INVALIDATION,
    ];
    let surfaces = [
        HeadlessSurface::new(120, 90),
        HeadlessSurface::new(120, 90),
        HeadlessSurface::new(120, 90),
        HeadlessSurface::new(120, 90),
        HeadlessSurface::new(120, 90),
        HeadlessSurface::new(120, 90),
        HeadlessSurface::new(90, 70),
    ];

    for (index, milestone) in model.milestones.iter().enumerate() {
        assert_eq!(milestone.step, RuntimeStepV1::ALL[index]);
        for lane in RuntimeExecutionLaneV1::ALL {
            let receipt = milestone.receipt(lane);
            assert_eq!(
                receipt.receipt_generation,
                (index > 0).then_some(index as u64)
            );
            assert_eq!(receipt.projection_generation, index as u64);
            assert_eq!(receipt.invalidation, invalidations[index]);
            assert_eq!(receipt.mutation_count, usize::from(index > 0));
        }
        for lane in RuntimeProjectionLaneV1::ALL {
            let projection = milestone.projection(lane);
            assert_eq!(projection.surface, surfaces[index]);
            assert_eq!(projection.counts.computed_styles, NODE_COUNTS[index]);
            assert_eq!(projection.counts.geometry, NODE_COUNTS[index]);
            assert_eq!(projection.counts.semantics, 1);
            assert_eq!(projection.counts.hit_regions, HIT_COUNTS[index]);
            assert_eq!(projection.counts.scene_rectangles, NODE_COUNTS[index]);
            assert_eq!(projection.geometries.len(), NODE_COUNTS[index]);
            assert_eq!(projection.hits.len(), HIT_COUNTS[index]);
            assert_eq!(projection.scenes.len(), NODE_COUNTS[index]);
        }
        assert_eq!(milestone.oracle_projection, milestone.reference_projection);
        assert_eq!(milestone.oracle_projection, milestone.candidate_projection);
    }

    assert_eq!(
        sum_counts(model, |projection| projection.geometries.len()),
        38
    );
    assert_eq!(sum_counts(model, |projection| projection.hits.len()), 24);
    assert_eq!(sum_counts(model, |projection| projection.scenes.len()), 38);
    assert_eq!(
        model
            .milestones
            .iter()
            .map(|milestone| milestone.oracle_projection.counts.semantics)
            .sum::<usize>(),
        7
    );
}

fn sum_counts(
    model: &RuntimeArtifactV1,
    count: impl Fn(&runtime::RuntimeProjectionV1) -> usize,
) -> usize {
    model
        .milestones
        .iter()
        .map(|milestone| count(&milestone.oracle_projection))
        .sum()
}

fn registered_faults(model: &RuntimeArtifactV1) -> Vec<RuntimeArtifactFaultV1> {
    let mut faults = Vec::new();
    for (milestone_index, milestone) in model.milestones.iter().enumerate() {
        for lane in RuntimeExecutionLaneV1::ALL {
            for field in RuntimeReceiptFieldV1::ALL {
                faults.push(RuntimeArtifactFaultV1::Receipt {
                    milestone: milestone_index,
                    lane,
                    field,
                });
            }
        }
        for lane in RuntimeProjectionLaneV1::ALL {
            let projection = milestone.projection(lane);
            for field in RuntimeProjectionFieldV1::ALL {
                faults.push(RuntimeArtifactFaultV1::Projection {
                    milestone: milestone_index,
                    lane,
                    field,
                });
            }
            for record in 0..projection.geometries.len() {
                for field in RuntimeGeometryFieldV1::ALL {
                    faults.push(RuntimeArtifactFaultV1::Geometry {
                        milestone: milestone_index,
                        lane,
                        record,
                        field,
                    });
                }
            }
            for record in 0..projection.hits.len() {
                for field in RuntimeHitFieldV1::ALL {
                    faults.push(RuntimeArtifactFaultV1::Hit {
                        milestone: milestone_index,
                        lane,
                        record,
                        field,
                    });
                }
            }
            for record in 0..projection.scenes.len() {
                for field in RuntimeSceneFieldV1::ALL {
                    faults.push(RuntimeArtifactFaultV1::Scene {
                        milestone: milestone_index,
                        lane,
                        record,
                        field,
                    });
                }
            }
        }
    }
    faults
}

fn slice_changed(
    baseline: &RuntimeArtifactV1,
    faulted: &RuntimeArtifactV1,
    slice: RuntimeArtifactSliceV1,
) -> bool {
    match slice {
        RuntimeArtifactFaultV1::Receipt {
            milestone,
            lane,
            field,
        } => {
            let left = baseline.milestones[milestone].receipt(lane);
            let right = faulted.milestones[milestone].receipt(lane);
            match field {
                RuntimeReceiptFieldV1::ReceiptGeneration => {
                    left.receipt_generation != right.receipt_generation
                }
                RuntimeReceiptFieldV1::ProjectionGeneration => {
                    left.projection_generation != right.projection_generation
                }
                RuntimeReceiptFieldV1::Invalidation => left.invalidation != right.invalidation,
                RuntimeReceiptFieldV1::MutationCount => left.mutation_count != right.mutation_count,
            }
        }
        RuntimeArtifactFaultV1::Projection {
            milestone,
            lane,
            field,
        } => projection_slice_changed(baseline, faulted, milestone, lane, field),
        RuntimeArtifactFaultV1::Geometry {
            milestone,
            lane,
            record,
            field,
        } => {
            let left = &baseline.milestones[milestone].projection(lane).geometries[record];
            let right = &faulted.milestones[milestone].projection(lane).geometries[record];
            match field {
                RuntimeGeometryFieldV1::Path => left.path != right.path,
                RuntimeGeometryFieldV1::Bounds => left.bounds != right.bounds,
                RuntimeGeometryFieldV1::Clip => left.clip != right.clip,
            }
        }
        RuntimeArtifactFaultV1::Hit {
            milestone,
            lane,
            record,
            field,
        } => {
            let left = &baseline.milestones[milestone].projection(lane).hits[record];
            let right = &faulted.milestones[milestone].projection(lane).hits[record];
            match field {
                RuntimeHitFieldV1::Path => left.path != right.path,
                RuntimeHitFieldV1::Clip => left.clip != right.clip,
            }
        }
        RuntimeArtifactFaultV1::Scene {
            milestone,
            lane,
            record,
            field,
        } => {
            let left = &baseline.milestones[milestone].projection(lane).scenes[record];
            let right = &faulted.milestones[milestone].projection(lane).scenes[record];
            match field {
                RuntimeSceneFieldV1::Path => left.path != right.path,
                RuntimeSceneFieldV1::Rectangle => left.rectangle != right.rectangle,
                RuntimeSceneFieldV1::Color => left.color != right.color,
            }
        }
    }
}

fn projection_slice_changed(
    baseline: &RuntimeArtifactV1,
    faulted: &RuntimeArtifactV1,
    milestone: usize,
    lane: RuntimeProjectionLaneV1,
    field: RuntimeProjectionFieldV1,
) -> bool {
    let left = baseline.milestones[milestone].projection(lane);
    let right = faulted.milestones[milestone].projection(lane);
    match field {
        RuntimeProjectionFieldV1::Surface => left.surface != right.surface,
        RuntimeProjectionFieldV1::ComputedStyleCount => {
            left.counts.computed_styles != right.counts.computed_styles
        }
        RuntimeProjectionFieldV1::GeometryCount => left.counts.geometry != right.counts.geometry,
        RuntimeProjectionFieldV1::SemanticsCount => left.counts.semantics != right.counts.semantics,
        RuntimeProjectionFieldV1::HitRegionCount => {
            left.counts.hit_regions != right.counts.hit_regions
        }
        RuntimeProjectionFieldV1::SceneRectangleCount => {
            left.counts.scene_rectangles != right.counts.scene_rectangles
        }
    }
}
