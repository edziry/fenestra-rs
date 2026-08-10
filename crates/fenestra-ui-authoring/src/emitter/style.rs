use proc_macro2::TokenStream;

use crate::resolved::{ResolvedStyleAssignmentV1, ResolvedStyleV1};

use super::builder::{array_into, ir_call, ir_path, u32_literal};
use super::value::{property_value, schema_namespace, schema_revision, span};

pub(super) fn style(style: &ResolvedStyleV1, namespace: u64, revision: u32) -> TokenStream {
    let assignments = style.assignments.iter().map(assignment).collect::<Vec<_>>();
    let trailing = assignments.len() > 1;
    ir_call(
        &["StyleProgram", "new"],
        vec![
            ir_path(&["SUPPORTED_STYLE_FORMAT"]),
            schema_namespace(namespace),
            schema_revision(revision),
            array_into(assignments, trailing),
            span(style.anchor),
        ],
        true,
    )
}

fn assignment(assignment: &ResolvedStyleAssignmentV1) -> TokenStream {
    ir_call(
        &["StyleAssignment", "new"],
        vec![
            ir_call(
                &["TemplateNodeId", "new"],
                vec![u32_literal(assignment.target)],
                false,
            ),
            ir_call(
                &["PropertyId", "new"],
                vec![u32_literal(assignment.property)],
                false,
            ),
            property_value(&assignment.value),
            span(assignment.anchor),
        ],
        true,
    )
}
