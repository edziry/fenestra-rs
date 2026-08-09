use std::fmt::Write;

use fenestra_ui_ir::prototype::InputPolicy;
use fenestra_ui_runtime::prototype::{HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole};

use super::super::model::HeadlessArtifactV1;
use super::{LineSinkV1, push_line};
use crate::headless::artifact::error::HeadlessArtifactEncodeErrorV1;
use crate::wire::write_node_path;

pub(super) fn write_projection(
    output: &mut impl LineSinkV1,
    artifact: &HeadlessArtifactV1,
) -> Result<(), HeadlessArtifactEncodeErrorV1> {
    let projection = &artifact.projection;
    let counts = projection.counts;
    push_line(
        output,
        &format!(
            "projection-begin|{}|{}|{}|{}|{}|{}|{}|{}",
            artifact.final_generation,
            projection.surface.width(),
            projection.surface.height(),
            counts.computed_styles(),
            counts.geometries(),
            counts.semantics(),
            counts.hit_regions(),
            counts.scene_rectangles(),
        ),
    )?;
    push_line(output, "computed-begin")?;
    for record in &projection.computed {
        let mut line = "computed|".to_owned();
        write_node_path(&mut line, &record.path);
        let _ = write!(
            line,
            "|{}|{}|rgba8:{}|{}|{}",
            record.width,
            record.height,
            rgba(record.color),
            bool_word(record.visible),
            input(record.input),
        );
        push_line(output, &line)?;
    }
    push_line(output, "computed-end")?;
    push_line(output, "geometry-begin")?;
    for record in &projection.geometry {
        let mut line = "geometry|".to_owned();
        write_node_path(&mut line, &record.path);
        write_rect(&mut line, record.bounds);
        write_rect(&mut line, record.clip);
        push_line(output, &line)?;
    }
    push_line(output, "geometry-end")?;
    push_line(output, "semantic-begin")?;
    for record in &projection.semantics {
        let mut line = "semantic|".to_owned();
        write_node_path(&mut line, &record.path);
        let _ = write!(
            line,
            "|{}|{}|{}",
            role(record.role),
            record.label,
            action(record.action)
        );
        push_line(output, &line)?;
    }
    push_line(output, "semantic-end")?;
    push_line(output, "hit-begin")?;
    for record in &projection.hits {
        let mut line = "hit|".to_owned();
        write_node_path(&mut line, &record.path);
        write_rect(&mut line, record.rectangle);
        push_line(output, &line)?;
    }
    push_line(output, "hit-end")?;
    push_line(output, "scene-begin")?;
    for record in &projection.scene {
        let mut line = "scene|".to_owned();
        write_node_path(&mut line, &record.path);
        write_rect(&mut line, record.rectangle);
        let _ = write!(line, "|rgba8:{}", rgba(record.color));
        push_line(output, &line)?;
    }
    push_line(output, "scene-end")?;
    push_line(output, "projection-end")
}

fn write_rect(line: &mut String, rectangle: HeadlessRect) {
    let _ = write!(
        line,
        "|{}|{}|{}|{}",
        rectangle.x(),
        rectangle.y(),
        rectangle.width(),
        rectangle.height()
    );
}

fn rgba(value: [u8; 4]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        value[0], value[1], value[2], value[3]
    )
}

const fn bool_word(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

const fn input(value: InputPolicy) -> &'static str {
    match value {
        InputPolicy::Accept => "accept",
        InputPolicy::Ignore => "ignore",
    }
}

const fn role(value: HeadlessSemanticRole) -> &'static str {
    match value {
        HeadlessSemanticRole::Control => "control",
    }
}

const fn action(value: HeadlessSemanticAction) -> &'static str {
    match value {
        HeadlessSemanticAction::Activate => "activate",
    }
}
