use std::fmt::Write as _;

use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet};
use fenestra_ui_runtime::prototype::{HeadlessRect, HeadlessSurface};
use fenestra_ui_testkit::prototype::{NodePathV1, PathSegmentV1};

use super::{
    RuntimeArtifactV1, RuntimeGeometryV1, RuntimeHitV1, RuntimeProjectionCountsV1,
    RuntimeProjectionV1, RuntimeReceiptV1, RuntimeSceneV1, RuntimeStepV1,
};

pub(crate) fn runtime_lines_v1(model: &RuntimeArtifactV1) -> Vec<String> {
    let mut lines = Vec::with_capacity(109);
    let geometry = payload_count(model, |projection| projection.geometries.len());
    let hit = payload_count(model, |projection| projection.hits.len());
    let scene = payload_count(model, |projection| projection.scenes.len());
    lines.push(format!(
        "runtime|steps={}|geometry={geometry}|hit={hit}|scene={scene}|count-order=computed,geometry,semantics,hit,scene",
        model.milestones.len()
    ));

    for (index, milestone) in model.milestones.iter().enumerate() {
        lines.push(format!(
            "runtime-generation|step={}|index={index}|{}|{}|{}|{}|{}",
            step_label(milestone.step),
            receipt("reference", &milestone.reference_receipt),
            receipt("candidate", &milestone.candidate_receipt),
            projection_summary("oracle", &milestone.oracle_projection),
            projection_summary("reference", &milestone.reference_projection),
            projection_summary("candidate", &milestone.candidate_projection),
        ));
    }
    append_geometry(model, &mut lines);
    append_hits(model, &mut lines);
    append_scenes(model, &mut lines);
    lines.push("runtime-result|classification=pass".to_owned());
    lines
}

fn append_geometry(model: &RuntimeArtifactV1, lines: &mut Vec<String>) {
    for (milestone_index, milestone) in model.milestones.iter().enumerate() {
        for (record_index, oracle) in milestone.oracle_projection.geometries.iter().enumerate() {
            let reference = &milestone.reference_projection.geometries[record_index];
            let candidate = &milestone.candidate_projection.geometries[record_index];
            lines.push(format!(
                "runtime-geometry|step={}|index={milestone_index}|record={record_index}|oracle={}|reference={}|candidate={}",
                step_label(milestone.step),
                geometry(oracle),
                geometry(reference),
                geometry(candidate),
            ));
        }
    }
}

fn append_hits(model: &RuntimeArtifactV1, lines: &mut Vec<String>) {
    for (milestone_index, milestone) in model.milestones.iter().enumerate() {
        for (record_index, oracle) in milestone.oracle_projection.hits.iter().enumerate() {
            let reference = &milestone.reference_projection.hits[record_index];
            let candidate = &milestone.candidate_projection.hits[record_index];
            lines.push(format!(
                "runtime-hit|step={}|index={milestone_index}|record={record_index}|oracle={}|reference={}|candidate={}",
                step_label(milestone.step),
                hit(oracle),
                hit(reference),
                hit(candidate),
            ));
        }
    }
}

fn append_scenes(model: &RuntimeArtifactV1, lines: &mut Vec<String>) {
    for (milestone_index, milestone) in model.milestones.iter().enumerate() {
        for (record_index, oracle) in milestone.oracle_projection.scenes.iter().enumerate() {
            let reference = &milestone.reference_projection.scenes[record_index];
            let candidate = &milestone.candidate_projection.scenes[record_index];
            lines.push(format!(
                "runtime-scene|step={}|index={milestone_index}|record={record_index}|oracle={}|reference={}|candidate={}",
                step_label(milestone.step),
                scene(oracle),
                scene(reference),
                scene(candidate),
            ));
        }
    }
}

fn payload_count(
    model: &RuntimeArtifactV1,
    count: impl Fn(&RuntimeProjectionV1) -> usize,
) -> usize {
    model
        .milestones
        .iter()
        .map(|milestone| count(&milestone.oracle_projection))
        .sum()
}

fn receipt(lane: &str, value: &RuntimeReceiptV1) -> String {
    format!(
        "{lane}=receipt:{};projection:{};invalidation:{};mutations:{}",
        generation(value.receipt_generation),
        value.projection_generation,
        invalidation(value.invalidation),
        value.mutation_count,
    )
}

fn projection_summary(lane: &str, projection: &RuntimeProjectionV1) -> String {
    format!(
        "{lane}=surface:{};counts:{}",
        surface(projection.surface),
        counts(projection.counts),
    )
}

fn geometry(record: &RuntimeGeometryV1) -> String {
    format!(
        "path:{};bounds:{};clip:{}",
        path(&record.path),
        rect(record.bounds),
        rect(record.clip),
    )
}

fn hit(record: &RuntimeHitV1) -> String {
    format!("path:{};clip:{}", path(&record.path), rect(record.clip))
}

fn scene(record: &RuntimeSceneV1) -> String {
    format!(
        "path:{};rect:{};rgba:{},{},{},{}",
        path(&record.path),
        rect(record.rectangle),
        record.color[0],
        record.color[1],
        record.color[2],
        record.color[3],
    )
}

fn generation(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn surface(value: HeadlessSurface) -> String {
    format!("{},{}", value.width(), value.height())
}

fn counts(value: RuntimeProjectionCountsV1) -> String {
    format!(
        "{},{},{},{},{}",
        value.computed_styles,
        value.geometry,
        value.semantics,
        value.hit_regions,
        value.scene_rectangles,
    )
}

fn rect(value: HeadlessRect) -> String {
    format!(
        "{},{},{},{}",
        value.x(),
        value.y(),
        value.width(),
        value.height()
    )
}

fn path(value: &NodePathV1) -> String {
    let mut path = "root".to_owned();
    for segment in value.segments() {
        match segment {
            PathSegmentV1::Static { authored_slot } => {
                write!(path, "/s:{authored_slot}").expect("writing to String cannot fail");
            }
            PathSegmentV1::Member { region_slot, key } => {
                write!(path, "/m:{region_slot}:{key}").expect("writing to String cannot fail");
            }
        }
    }
    path
}

fn invalidation(value: InvalidationSet) -> String {
    let mut words = String::new();
    for class in value.iter() {
        if !words.is_empty() {
            words.push(',');
        }
        words.push_str(invalidation_label(class));
    }
    if words.is_empty() {
        words.push_str("none");
    }
    words
}

const fn step_label(step: RuntimeStepV1) -> &'static str {
    match step {
        RuntimeStepV1::Initial => "initial",
        RuntimeStepV1::Color => "color",
        RuntimeStepV1::Insert => "insert",
        RuntimeStepV1::Move => "move",
        RuntimeStepV1::Update => "update",
        RuntimeStepV1::Remove => "remove",
        RuntimeStepV1::Resize => "resize",
    }
}

const fn invalidation_label(class: InvalidationClass) -> &'static str {
    match class {
        InvalidationClass::Structure => "structure",
        InvalidationClass::StyleMatch => "style-match",
        InvalidationClass::Intrinsic => "intrinsic",
        InvalidationClass::Layout => "layout",
        InvalidationClass::Semantics => "semantics",
        InvalidationClass::HitTest => "hit-test",
        InvalidationClass::Paint => "paint",
        InvalidationClass::Composition => "composition",
        InvalidationClass::Surface => "surface",
    }
}
