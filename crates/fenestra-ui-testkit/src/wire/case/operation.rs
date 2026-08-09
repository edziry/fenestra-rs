use fenestra_ui_ir::prototype::PropertyId;

use crate::case::SemanticOperationV1;

use super::RawOperation;
use crate::wire::error::ArtifactDecodeError;
use crate::wire::path::{parse_fragment_path, parse_node_path};
use crate::wire::primitive::{parse_property_value, parse_u32, parse_u64};

pub(super) fn parse_operation(
    raw: &RawOperation<'_>,
    line: u32,
) -> Result<(SemanticOperationV1, usize), ArtifactDecodeError> {
    let result = match raw {
        RawOperation::Set(node, property, value) => {
            let node = parse_node_path(node, line)?;
            let depth = node.depth();
            (
                SemanticOperationV1::SetProperty {
                    node,
                    property: PropertyId::new(parse_u32(property, line)?),
                    value: parse_property_value(value, line)?,
                },
                depth,
            )
        }
        RawOperation::Insert(fragment, key, index) => {
            let fragment = parse_fragment_path(fragment, line)?;
            let depth = fragment.owner().depth();
            (
                SemanticOperationV1::InsertKeyed {
                    fragment,
                    key: parse_u64(key, line)?,
                    final_index: parse_u32(index, line)?,
                },
                depth,
            )
        }
        RawOperation::Move(fragment, key, index) => {
            let fragment = parse_fragment_path(fragment, line)?;
            let depth = fragment.owner().depth();
            (
                SemanticOperationV1::MoveKeyed {
                    fragment,
                    key: parse_u64(key, line)?,
                    final_index: parse_u32(index, line)?,
                },
                depth,
            )
        }
        RawOperation::Update(fragment, key, property, value) => {
            let fragment = parse_fragment_path(fragment, line)?;
            let depth = fragment.owner().depth();
            (
                SemanticOperationV1::UpdateKeyed {
                    fragment,
                    key: parse_u64(key, line)?,
                    property: PropertyId::new(parse_u32(property, line)?),
                    value: parse_property_value(value, line)?,
                },
                depth,
            )
        }
        RawOperation::Remove(fragment, key) => {
            let fragment = parse_fragment_path(fragment, line)?;
            let depth = fragment.owner().depth();
            (
                SemanticOperationV1::RemoveKeyed {
                    fragment,
                    key: parse_u64(key, line)?,
                },
                depth,
            )
        }
    };
    Ok(result)
}
