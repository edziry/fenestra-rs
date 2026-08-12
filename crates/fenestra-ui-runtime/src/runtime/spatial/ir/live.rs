use std::collections::HashSet;

use fenestra_ui_ir::prototype::{
    ChildFactory, ChildFactoryIter, RegionFactory, TemplateFactory, ValidatedSpatialProgramV2,
};

use super::super::error::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};
use super::super::view::RuntimeSpatialBuildViewV2;
use super::model::{ExpandedSpatialNode, LiveProgram, ProgramIndex};
use crate::logical_tree::NodeId;
use crate::runtime::fragment::FragmentId;
use crate::runtime::view::KeyedMemberIter;

struct LogicalInstance {
    node: NodeId,
    template: fenestra_ui_ir::prototype::TemplateNodeId,
    context: Vec<u64>,
}

struct PendingLogical<'a> {
    node: NodeId,
    template: TemplateFactory<'a>,
    context: Vec<u64>,
}

struct RegionState<'a, 'view> {
    region: RegionFactory<'a>,
    fragment: FragmentId,
    members: KeyedMemberIter<'view>,
}

struct OwnerFrame<'a, 'view> {
    owner: PendingLogical<'a>,
    slots: ChildFactoryIter<'a>,
    region: Option<RegionState<'a, 'view>>,
    actual_index: usize,
}

struct SpatialFrame {
    declaration: usize,
    logical_index: usize,
    parent_ordinal: u128,
}

pub(super) fn build_live<'a>(
    program: &'a ValidatedSpatialProgramV2,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<LiveProgram<'a>, RuntimeSpatialIrErrorV2> {
    let index = ProgramIndex::new(program)?;
    let logical = enumerate_logical(program, view)?;
    let expanded = expand_spatial(&index, &logical)?;
    Ok(LiveProgram::new(index, expanded))
}

fn enumerate_logical<'program, 'view>(
    program: &'program ValidatedSpatialProgramV2,
    view: RuntimeSpatialBuildViewV2<'view>,
) -> Result<Vec<LogicalInstance>, RuntimeSpatialIrErrorV2> {
    let root = view.root();
    if view.parent(root).is_some() {
        return Err(invariant(program));
    }
    let root_factory = program.style().construction().root_factory();
    let root = PendingLogical {
        node: root,
        template: root_factory,
        context: Vec::new(),
    };
    let mut visited = HashSet::with_capacity(view.node_count());
    enter(program, view, &root, &mut visited)?;
    let mut logical = vec![instance(&root)];
    let mut stack = vec![open_owner(root)];

    while let Some(mut frame) = stack.pop() {
        if let Some(child) = next_child(program, view, &mut frame)? {
            stack.push(frame);
            enter(program, view, &child, &mut visited)?;
            logical.push(instance(&child));
            stack.push(open_owner(child));
        } else {
            let Some(actual) = view.children(frame.owner.node) else {
                return Err(invariant(program));
            };
            if frame.actual_index != actual.len() {
                return Err(invariant(program));
            }
        }
    }

    if logical.len() != view.node_count() || visited.len() != view.node_count() {
        return Err(invariant(program));
    }
    Ok(logical)
}

fn enter(
    program: &ValidatedSpatialProgramV2,
    view: RuntimeSpatialBuildViewV2<'_>,
    pending: &PendingLogical<'_>,
    visited: &mut HashSet<NodeId>,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    if !visited.insert(pending.node)
        || view.template(pending.node) != Some(pending.template.id())
        || view.component(pending.node) != Some(pending.template.component().id())
    {
        return Err(invariant(program));
    }
    Ok(())
}

fn open_owner<'program, 'view>(owner: PendingLogical<'program>) -> OwnerFrame<'program, 'view> {
    OwnerFrame {
        slots: owner.template.children(),
        owner,
        region: None,
        actual_index: 0,
    }
}

fn next_child<'program, 'view>(
    program: &ValidatedSpatialProgramV2,
    view: RuntimeSpatialBuildViewV2<'view>,
    frame: &mut OwnerFrame<'program, 'view>,
) -> Result<Option<PendingLogical<'program>>, RuntimeSpatialIrErrorV2> {
    loop {
        if let Some(region) = &mut frame.region {
            if let Some((key, node)) = region.members.next() {
                let fragment = region.fragment;
                let template = region.region.repeat_body();
                let child = child_at(program, view, frame, node, template)?;
                if view.keyed_member(fragment, key) != Some(node) {
                    return Err(invariant(program));
                }
                let mut context = frame.owner.context.clone();
                context.push(key);
                return Ok(Some(PendingLogical { context, ..child }));
            }
            frame.region = None;
        }

        let Some(slot) = frame.slots.next() else {
            return Ok(None);
        };
        match slot {
            ChildFactory::Static { template, .. } => {
                let Some(actual) = view.children(frame.owner.node) else {
                    return Err(invariant(program));
                };
                let Some(&node) = actual.get(frame.actual_index) else {
                    return Err(invariant(program));
                };
                let mut child = child_at(program, view, frame, node, template)?;
                child.context.clone_from(&frame.owner.context);
                return Ok(Some(child));
            }
            ChildFactory::Region { region, .. } => {
                let fragment = view
                    .fragment(frame.owner.node, region.id())
                    .ok_or_else(|| invariant(program))?;
                let members = view
                    .keyed_members(fragment)
                    .ok_or_else(|| invariant(program))?;
                frame.region = Some(RegionState {
                    region,
                    fragment,
                    members,
                });
            }
        }
    }
}

fn child_at<'program, 'view>(
    program: &ValidatedSpatialProgramV2,
    view: RuntimeSpatialBuildViewV2<'view>,
    frame: &mut OwnerFrame<'program, 'view>,
    node: NodeId,
    template: TemplateFactory<'program>,
) -> Result<PendingLogical<'program>, RuntimeSpatialIrErrorV2> {
    let Some(actual) = view.children(frame.owner.node) else {
        return Err(invariant(program));
    };
    if actual.get(frame.actual_index).copied() != Some(node)
        || view.parent(node) != Some(frame.owner.node)
        || view.template(node) != Some(template.id())
    {
        return Err(invariant(program));
    }
    frame.actual_index = checked_increment(frame.actual_index, program)?;
    Ok(PendingLogical {
        node,
        template,
        context: Vec::new(),
    })
}

fn instance(pending: &PendingLogical<'_>) -> LogicalInstance {
    LogicalInstance {
        node: pending.node,
        template: pending.template.id(),
        context: pending.context.clone(),
    }
}

fn expand_spatial<'a>(
    index: &ProgramIndex<'a>,
    logical: &[LogicalInstance],
) -> Result<Vec<ExpandedSpatialNode<'a>>, RuntimeSpatialIrErrorV2> {
    let program = index.program();
    let mut stack = Vec::new();
    push_matching_children(index, logical, None, &[], 0, &mut stack);
    let mut expanded = Vec::new();

    while let Some(frame) = stack.pop() {
        let ordinal = u128::try_from(expanded.len())
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| arithmetic(program))?;
        let logical_instance = &logical[frame.logical_index];
        expanded.push(ExpandedSpatialNode::new(
            logical_instance.node,
            frame.declaration,
            index.declaration(frame.declaration),
            logical_instance.context.clone(),
            ordinal,
            frame.parent_ordinal,
        ));
        push_matching_children(
            index,
            logical,
            Some(frame.declaration),
            &logical_instance.context,
            ordinal,
            &mut stack,
        );
    }
    Ok(expanded)
}

fn push_matching_children(
    index: &ProgramIndex<'_>,
    logical: &[LogicalInstance],
    parent: Option<usize>,
    parent_context: &[u64],
    parent_ordinal: u128,
    stack: &mut Vec<SpatialFrame>,
) {
    let mut matches = Vec::new();
    for declaration in index.children(parent) {
        let signature = index.signature(declaration);
        for (logical_index, instance) in logical.iter().enumerate() {
            if instance.template == *index.declaration(declaration).template().value()
                && instance.context.len() == signature.len()
                && instance.context.starts_with(parent_context)
            {
                matches.push(SpatialFrame {
                    declaration,
                    logical_index,
                    parent_ordinal,
                });
            }
        }
    }
    stack.extend(matches.into_iter().rev());
}

fn checked_increment(
    value: usize,
    program: &ValidatedSpatialProgramV2,
) -> Result<usize, RuntimeSpatialIrErrorV2> {
    value.checked_add(1).ok_or_else(|| arithmetic(program))
}

fn invariant(program: &ValidatedSpatialProgramV2) -> RuntimeSpatialIrErrorV2 {
    RuntimeSpatialIrErrorV2::new(
        RuntimeSpatialIrErrorKindV2::InvariantViolation,
        program.program().span(),
    )
}

fn arithmetic(program: &ValidatedSpatialProgramV2) -> RuntimeSpatialIrErrorV2 {
    RuntimeSpatialIrErrorV2::new(
        RuntimeSpatialIrErrorKindV2::ArithmeticExhausted,
        program.program().span(),
    )
}
