use fenestra_ui_testkit::prototype::NormalizedHeadlessProjectionV1;

use super::RuntimeArtifactEncodeErrorV1;
use super::encode::ArtifactWriterV1;
use super::path::node_path;
use super::value::{color, input, rect, semantic_action, semantic_role, surface};

pub(super) fn encode_projection(
    writer: &mut ArtifactWriterV1,
    projection: &NormalizedHeadlessProjectionV1,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    writer.push("projection|begin")?;
    writer.push(&format!("surface|{}", surface(projection.surface())))?;

    writer.push(&format!(
        "computed|count={}",
        projection.computed_styles().len()
    ))?;
    for (order, record) in projection.computed_styles().iter().enumerate() {
        writer.push(&format!(
            "computed-record|i={order}|path={}|width={}|height={}|color={}|visible={}|input={}",
            node_path(record.path()),
            record.width(),
            record.height(),
            color(record.color()),
            record.visible(),
            input(record.input()),
        ))?;
    }

    writer.push(&format!("geometry|count={}", projection.geometries().len()))?;
    for (order, record) in projection.geometries().iter().enumerate() {
        writer.push(&format!(
            "geometry-record|i={order}|path={}|bounds={}|clip={}",
            node_path(record.path()),
            rect(record.bounds()),
            rect(record.clip()),
        ))?;
    }

    writer.push(&format!("semantics|count={}", projection.semantics().len()))?;
    for (order, record) in projection.semantics().iter().enumerate() {
        writer.push(&format!(
            "semantic-record|i={order}|path={}|role={}|label={}|action={}",
            node_path(record.path()),
            semantic_role(record.role()),
            record.label(),
            semantic_action(record.action()),
        ))?;
    }

    writer.push(&format!("hits|count={}", projection.hit_regions().len()))?;
    for (order, record) in projection.hit_regions().iter().enumerate() {
        writer.push(&format!(
            "hit-record|i={order}|path={}|clip={}",
            node_path(record.path()),
            rect(record.clip()),
        ))?;
    }

    writer.push(&format!(
        "scene|count={}",
        projection.scene_rectangles().len()
    ))?;
    for (order, record) in projection.scene_rectangles().iter().enumerate() {
        writer.push(&format!(
            "scene-record|i={order}|path={}|rectangle={}|color={}",
            node_path(record.path()),
            rect(record.rectangle()),
            color(record.color()),
        ))?;
    }
    writer.push("projection|end")
}
