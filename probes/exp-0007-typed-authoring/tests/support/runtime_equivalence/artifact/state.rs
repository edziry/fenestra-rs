use fenestra_ui_testkit::prototype::{NormalizedChildGroupV1, NormalizedStateV1};

use super::RuntimeArtifactEncodeErrorV1;
use super::encode::ArtifactWriterV1;
use super::path::{fragment_path, node_path};
use super::value::property_value;

pub(super) fn encode_state(
    writer: &mut ArtifactWriterV1,
    state: &NormalizedStateV1,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let property_count = state
        .nodes()
        .iter()
        .map(|node| node.properties().len())
        .sum::<usize>();
    let child_count = state
        .nodes()
        .iter()
        .map(|node| node.child_groups().len())
        .sum::<usize>();
    let member_count = state
        .fragments()
        .iter()
        .map(|fragment| fragment.members().len())
        .sum::<usize>();

    writer.push("state|begin")?;
    writer.push(&format!("nodes|count={}", state.nodes().len()))?;
    writer.push(&format!("properties|count={property_count}"))?;
    writer.push(&format!("child-groups|count={child_count}"))?;
    writer.push(&format!(
        "fragments|count={}|members={member_count}",
        state.fragments().len()
    ))?;

    for (order, node) in state.nodes().iter().enumerate() {
        let parent = node.parent().map_or_else(|| "none".to_owned(), node_path);
        writer.push(&format!(
            "node|i={order}|path={}|parent={parent}|template={}|component={}",
            node_path(node.path()),
            node.template().get(),
            node.component().get(),
        ))?;
    }
    for node in state.nodes() {
        for (order, property) in node.properties().iter().enumerate() {
            writer.push(&format!(
                "property|node={}|i={order}|id={}|value={}",
                node_path(node.path()),
                property.property().get(),
                property_value(property.value()),
            ))?;
        }
    }
    for node in state.nodes() {
        for (order, child) in node.child_groups().iter().enumerate() {
            let relationship = match child {
                NormalizedChildGroupV1::Static(path) => {
                    format!("kind=static|target={}", node_path(path))
                }
                NormalizedChildGroupV1::Region(path) => {
                    format!("kind=region|target={}", fragment_path(path))
                }
            };
            writer.push(&format!(
                "child|node={}|i={order}|{relationship}",
                node_path(node.path())
            ))?;
        }
    }
    for (order, fragment) in state.fragments().iter().enumerate() {
        writer.push(&format!(
            "fragment|i={order}|path={}|descriptor={}",
            fragment_path(fragment.path()),
            fragment.descriptor().get(),
        ))?;
    }
    for fragment in state.fragments() {
        for (order, member) in fragment.members().iter().enumerate() {
            writer.push(&format!(
                "member|fragment={}|i={order}|key={}|node={}",
                fragment_path(fragment.path()),
                member.key(),
                node_path(member.node()),
            ))?;
        }
    }
    writer.push("state|end")
}
