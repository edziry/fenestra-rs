use std::fmt::Write;

use crate::case::{GeneratedCaseV1, OperationV1, SemanticOperationV1};

use super::{
    CASE_BYTES_LIMIT, OPERATIONS_LIMIT, OPERATIONS_PER_TRANSACTION_LIMIT, PATH_DEPTH_LIMIT,
    TRANSACTIONS_LIMIT,
};
use crate::wire::error::{ArtifactEncodeError, ArtifactLimitKind};
use crate::wire::path::{write_fragment_path, write_node_path};
use crate::wire::primitive::write_property_value;
use crate::wire::scan::{LINE_BYTES_LIMIT, LINES_LIMIT};

/// Encodes only canonical `tx` and `op` records, including each record LF.
pub fn encode_case_v1(case: &GeneratedCaseV1) -> Result<Vec<u8>, ArtifactEncodeError> {
    let lines = render_case(case);
    let byte_count = validate_limits(case, &lines)?;
    let mut encoded = Vec::with_capacity(byte_count);
    for line in lines {
        encoded.extend_from_slice(line.as_bytes());
        encoded.push(b'\n');
    }
    Ok(encoded)
}

fn render_case(case: &GeneratedCaseV1) -> Vec<String> {
    let mut lines = Vec::new();
    for transaction in case.transactions() {
        lines.push(format!(
            "tx|{}|{}",
            transaction.id().get(),
            transaction.operations().len()
        ));
        lines.extend(transaction.operations().iter().map(render_operation));
    }
    lines
}

fn render_operation(operation: &OperationV1) -> String {
    let mut line = format!("op|{}|", operation.id().get());
    match operation.operation() {
        SemanticOperationV1::SetProperty {
            node,
            property,
            value,
        } => {
            line.push_str("set|");
            write_node_path(&mut line, node);
            let _ = write!(line, "|{}|", property.get());
            write_property_value(&mut line, value);
        }
        SemanticOperationV1::InsertKeyed {
            fragment,
            key,
            final_index,
        } => render_keyed_index(&mut line, "insert", fragment, *key, *final_index),
        SemanticOperationV1::MoveKeyed {
            fragment,
            key,
            final_index,
        } => render_keyed_index(&mut line, "move", fragment, *key, *final_index),
        SemanticOperationV1::UpdateKeyed {
            fragment,
            key,
            property,
            value,
        } => {
            line.push_str("update|");
            write_fragment_path(&mut line, fragment);
            let _ = write!(line, "|{key}|{}|", property.get());
            write_property_value(&mut line, value);
        }
        SemanticOperationV1::RemoveKeyed { fragment, key } => {
            line.push_str("remove|");
            write_fragment_path(&mut line, fragment);
            let _ = write!(line, "|{key}");
        }
    }
    line
}

fn render_keyed_index(
    line: &mut String,
    kind: &str,
    fragment: &crate::semantic::FragmentPathV1,
    key: u64,
    index: u32,
) {
    line.push_str(kind);
    line.push('|');
    write_fragment_path(line, fragment);
    let _ = write!(line, "|{key}|{index}");
}

fn validate_limits(case: &GeneratedCaseV1, lines: &[String]) -> Result<usize, ArtifactEncodeError> {
    if lines.iter().any(|line| line.len() > LINE_BYTES_LIMIT) {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::LineBytes));
    }
    if lines.len() > LINES_LIMIT {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::Lines));
    }
    let byte_count = lines.iter().try_fold(0_usize, |total, line| {
        line.len()
            .checked_add(1)
            .and_then(|line_bytes| total.checked_add(line_bytes))
    });
    let Some(byte_count) = byte_count else {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::CaseBytes));
    };
    if byte_count > CASE_BYTES_LIMIT {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::CaseBytes));
    }
    if case.transactions().len() > TRANSACTIONS_LIMIT {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::Transactions));
    }
    if case
        .transactions()
        .iter()
        .any(|transaction| transaction.operations().len() > OPERATIONS_PER_TRANSACTION_LIMIT)
    {
        return Err(ArtifactEncodeError::limit(
            ArtifactLimitKind::OperationsPerTransaction,
        ));
    }
    let operations = case
        .transactions()
        .iter()
        .try_fold(0_usize, |total, transaction| {
            total.checked_add(transaction.operations().len())
        });
    if operations.is_none_or(|count| count > OPERATIONS_LIMIT) {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::Operations));
    }
    if case.transactions().iter().any(|transaction| {
        transaction
            .operations()
            .iter()
            .any(|operation| operation_path_depth(operation.operation()) > PATH_DEPTH_LIMIT)
    }) {
        return Err(ArtifactEncodeError::limit(ArtifactLimitKind::PathDepth));
    }
    Ok(byte_count)
}

fn operation_path_depth(operation: &SemanticOperationV1) -> usize {
    match operation {
        SemanticOperationV1::SetProperty { node, .. } => node.depth(),
        SemanticOperationV1::InsertKeyed { fragment, .. }
        | SemanticOperationV1::MoveKeyed { fragment, .. }
        | SemanticOperationV1::UpdateKeyed { fragment, .. }
        | SemanticOperationV1::RemoveKeyed { fragment, .. } => fragment.owner().depth(),
    }
}
