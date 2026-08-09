use std::collections::HashMap;

use crate::construction::{ChildSlotKind, ConstructionProgram};
use crate::error::{IrValidationError, IrValidationErrorKind, ValidationLimitKind};
use crate::ids::{StructuralRegionId, TemplateNodeId};
use crate::limits::ValidationLimits;
use crate::source::SourceSpan;

use super::{failure, limit_failure};

#[derive(Clone, Copy)]
struct Edge {
    target: usize,
    span: SourceSpan,
    multiplicity: usize,
}

pub(super) fn validate_ownership_and_limits(
    program: &ConstructionProgram,
    nodes: &HashMap<TemplateNodeId, usize>,
    regions: &HashMap<StructuralRegionId, usize>,
    limits: ValidationLimits,
) -> Result<usize, IrValidationError> {
    let graph = build_graph(program, nodes, regions)?;
    let root = find_root(program, &graph.incoming)?;
    reject_cycles(program, &graph.edges)?;
    validate_depth(program, &graph.edges, root, limits.template_depth())?;
    validate_expansion(program, &graph.edges, root, limits.initial_instances())?;
    Ok(root)
}

struct Graph {
    edges: Vec<Vec<Edge>>,
    incoming: Vec<Option<SourceSpan>>,
}

fn build_graph(
    program: &ConstructionProgram,
    nodes: &HashMap<TemplateNodeId, usize>,
    regions: &HashMap<StructuralRegionId, usize>,
) -> Result<Graph, IrValidationError> {
    let mut edges = vec![Vec::new(); program.nodes.len()];
    let mut incoming = vec![None; program.nodes.len()];

    for node in &program.nodes {
        for slot in &node.children {
            let ChildSlotKind::Static(child) = slot.kind else {
                continue;
            };
            register_owner(&mut incoming, nodes[&child], slot.span)?;
        }
    }
    for region in &program.regions {
        register_owner(&mut incoming, nodes[&region.repeat_body], region.span)?;
    }

    for (owner_index, node) in program.nodes.iter().enumerate() {
        for slot in &node.children {
            let edge = match slot.kind {
                ChildSlotKind::Static(child) => Edge {
                    target: nodes[&child],
                    span: slot.span,
                    multiplicity: 1,
                },
                ChildSlotKind::Region(region) => {
                    let declared = &program.regions[regions[&region]];
                    Edge {
                        target: nodes[&declared.repeat_body],
                        span: declared.span,
                        multiplicity: declared.initial_keys.len(),
                    }
                }
            };
            edges[owner_index].push(edge);
        }
    }

    Ok(Graph { edges, incoming })
}

fn register_owner(
    incoming: &mut [Option<SourceSpan>],
    target: usize,
    span: SourceSpan,
) -> Result<(), IrValidationError> {
    if incoming[target].replace(span).is_some() {
        return Err(failure(IrValidationErrorKind::DuplicateNodeOwner, span));
    }
    Ok(())
}

fn find_root(
    program: &ConstructionProgram,
    incoming: &[Option<SourceSpan>],
) -> Result<usize, IrValidationError> {
    let mut roots = incoming
        .iter()
        .enumerate()
        .filter_map(|(index, owner)| owner.is_none().then_some(index));
    let Some(root) = roots.next() else {
        return Err(failure(
            IrValidationErrorKind::InvalidRootCount,
            program.span,
        ));
    };
    if roots.next().is_some() {
        return Err(failure(
            IrValidationErrorKind::InvalidRootCount,
            program.span,
        ));
    }
    Ok(root)
}

fn reject_cycles(
    program: &ConstructionProgram,
    edges: &[Vec<Edge>],
) -> Result<(), IrValidationError> {
    let mut colors = vec![0_u8; program.nodes.len()];
    for start in 0..program.nodes.len() {
        if colors[start] != 0 {
            continue;
        }
        colors[start] = 1;
        let mut stack = vec![(start, 0_usize)];
        while let Some((node, next_edge)) = stack.last_mut() {
            let Some(edge) = edges[*node].get(*next_edge).copied() else {
                colors[*node] = 2;
                stack.pop();
                continue;
            };
            *next_edge += 1;
            match colors[edge.target] {
                0 => {
                    colors[edge.target] = 1;
                    stack.push((edge.target, 0));
                }
                1 => {
                    return Err(failure(IrValidationErrorKind::OwnershipCycle, edge.span));
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn validate_depth(
    program: &ConstructionProgram,
    edges: &[Vec<Edge>],
    root: usize,
    maximum: usize,
) -> Result<(), IrValidationError> {
    if maximum < 1 {
        return Err(limit_failure(
            ValidationLimitKind::TemplateDepth,
            program.span,
        ));
    }

    let mut stack = vec![(root, 0_usize, 1_usize)];
    while let Some((node, next_edge, depth)) = stack.last_mut() {
        let Some(edge) = edges[*node].get(*next_edge).copied() else {
            stack.pop();
            continue;
        };
        *next_edge += 1;
        let child_depth = depth
            .checked_add(1)
            .ok_or_else(|| limit_failure(ValidationLimitKind::TemplateDepth, edge.span))?;
        if child_depth > maximum {
            return Err(limit_failure(ValidationLimitKind::TemplateDepth, edge.span));
        }
        stack.push((edge.target, 0, child_depth));
    }
    Ok(())
}

fn validate_expansion(
    program: &ConstructionProgram,
    edges: &[Vec<Edge>],
    root: usize,
    maximum: usize,
) -> Result<(), IrValidationError> {
    let mut total = 1_usize;
    if total > maximum {
        return Err(limit_failure(
            ValidationLimitKind::InitialInstances,
            program.span,
        ));
    }

    let mut stack = vec![(root, 0_usize, 1_usize)];
    while let Some((node, next_edge, multiplicity)) = stack.last_mut() {
        let Some(edge) = edges[*node].get(*next_edge).copied() else {
            stack.pop();
            continue;
        };
        *next_edge += 1;
        let child_multiplicity = multiplicity
            .checked_mul(edge.multiplicity)
            .ok_or_else(|| limit_failure(ValidationLimitKind::InitialInstances, edge.span))?;
        if child_multiplicity == 0 {
            continue;
        }
        total = total
            .checked_add(child_multiplicity)
            .ok_or_else(|| limit_failure(ValidationLimitKind::InitialInstances, edge.span))?;
        if total > maximum {
            return Err(limit_failure(
                ValidationLimitKind::InitialInstances,
                edge.span,
            ));
        }
        stack.push((edge.target, 0, child_multiplicity));
    }
    Ok(())
}
