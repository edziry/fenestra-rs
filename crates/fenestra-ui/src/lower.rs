use std::collections::HashSet;

use fenestra_ui_ir::prototype::{
    IrValidationError, PropertyId, SUPPORTED_SPATIAL_FORMAT, SchemaNamespace, SchemaRevision,
    SourceSpan, SpatialProgramV2, SpatialValidationLimitsV2, ValidatedSpatialProgramV2,
    validate_spatial,
};
use fenestra_ui_runtime::prototype::RuntimeCapacity;
use fenestra_ui_spatial::prototype::SpatialLimitsV2;

use crate::model::ElementKind;
use crate::{Element, Error, Limits, View};

mod construction;
mod spatial;

pub(crate) const WIDTH: PropertyId = PropertyId::new(0);
pub(crate) const HEIGHT: PropertyId = PropertyId::new(1);
pub(crate) const PADDING: PropertyId = PropertyId::new(2);
pub(crate) const GAP: PropertyId = PropertyId::new(3);
pub(crate) const BACKGROUND: PropertyId = PropertyId::new(4);
pub(crate) const INPUT: PropertyId = PropertyId::new(5);
const NAMESPACE: SchemaNamespace = SchemaNamespace::new(1);
const REVISION: SchemaRevision = SchemaRevision::new(1);
const SPAN: SourceSpan = SourceSpan::Synthetic;

pub(crate) struct Lowered {
    pub(crate) program: ValidatedSpatialProgramV2,
    pub(crate) names: Vec<String>,
    pub(crate) spatial_limits: SpatialLimitsV2,
    pub(crate) capacity: RuntimeCapacity,
}

struct FlatElement<'a> {
    element: &'a Element,
    parent: Option<usize>,
    children: Vec<u32>,
}

struct FlatView<'a> {
    nodes: Vec<FlatElement<'a>>,
    depth: usize,
    children_per_node: usize,
}

pub(crate) fn lower(view: &View, limits: Limits) -> Result<Lowered, Error> {
    validate_name(&view.name)?;
    let flat = flatten(&view.root, limits)?;
    let n = flat.nodes.len();
    let spatial_nodes = n.checked_add(1).ok_or(Error::CapacityOverflow)?;
    let property_slots = n.checked_mul(6).ok_or(Error::CapacityOverflow)?;
    let spatial_depth = flat.depth.checked_add(1).ok_or(Error::CapacityOverflow)?;
    u32::try_from(spatial_nodes).map_err(|_| Error::CapacityOverflow)?;

    let style = construction::build(&flat, property_slots)?;
    let nodes = flat
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| spatial::node(index as u32, node))
        .collect();
    let program = SpatialProgramV2::new(
        SUPPORTED_SPATIAL_FORMAT,
        NAMESPACE,
        REVISION,
        spatial::viewport(),
        nodes,
        Vec::new(),
        SPAN,
    );
    let validation_limits = SpatialValidationLimitsV2::new([n, n, n, 0, n, n, 0, 0, 0, 0, 0, 0, 0]);
    let program = validate_spatial(&style, program, validation_limits).map_err(invalid_program)?;
    // The viewport adds one spatial node. The connected layout tree adds one
    // island and one dependency vertex, with no free-placement edges.
    let spatial_limits = SpatialLimitsV2::new([
        spatial_nodes,
        n,
        n,
        0,
        n,
        n,
        0,
        0,
        0,
        0,
        0,
        0,
        spatial_depth,
        flat.children_per_node.max(1),
        1,
        spatial_nodes,
        spatial_nodes,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        1,
        1,
        0,
        0,
        1,
        0,
    ]);
    Ok(Lowered {
        program,
        names: flat
            .nodes
            .iter()
            .map(|node| node.element.name.clone())
            .collect(),
        spatial_limits,
        capacity: RuntimeCapacity::new(16, 0, n, 0, property_slots, 2),
    })
}

fn flatten(root: &Element, limits: Limits) -> Result<FlatView<'_>, Error> {
    let mut result = FlatView {
        nodes: Vec::new(),
        depth: 0,
        children_per_node: 0,
    };
    let mut names = HashSet::new();
    let mut pending = vec![(root, None::<usize>, 1usize)];
    while let Some((element, parent, depth)) = pending.pop() {
        let count = result
            .nodes
            .len()
            .checked_add(1)
            .ok_or(Error::CapacityOverflow)?;
        check_limit("nodes", count, limits.max_nodes)?;
        check_limit("depth", depth, limits.max_depth)?;
        let id = u32::try_from(result.nodes.len()).map_err(|_| Error::CapacityOverflow)?;
        validate_name(&element.name)?;
        if !names.insert(element.name.as_str()) {
            return Err(Error::DuplicateName {
                name: element.name.clone(),
            });
        }
        element.style.validate(&element.name, element.kind)?;
        if element.kind == ElementKind::Rect && !element.children.is_empty() {
            return Err(Error::InvalidElement {
                node: element.name.clone(),
                reason: "rectangles cannot contain children",
            });
        }
        if let Some(parent) = parent {
            result.nodes[parent].children.push(id);
        }
        let index = result.nodes.len();
        result.depth = result.depth.max(depth);
        result.children_per_node = result.children_per_node.max(element.children.len());
        result.nodes.push(FlatElement {
            element,
            parent,
            children: Vec::new(),
        });
        if !element.children.is_empty() {
            let child_depth = depth.checked_add(1).ok_or(Error::CapacityOverflow)?;
            pending.extend(
                element
                    .children
                    .iter()
                    .rev()
                    .map(|child| (child, Some(index), child_depth)),
            );
        }
    }
    Ok(result)
}

fn validate_name(name: &str) -> Result<(), Error> {
    let mut bytes = name.bytes();
    let starts_correctly = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    if !starts_correctly || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        return Err(Error::InvalidName { name: name.into() });
    }
    Ok(())
}

fn check_limit(resource: &'static str, actual: usize, limit: usize) -> Result<(), Error> {
    if actual > limit {
        Err(Error::LimitExceeded {
            resource,
            limit,
            actual,
        })
    } else {
        Ok(())
    }
}

fn invalid_program(error: IrValidationError) -> Error {
    Error::InvalidProgram(format!("{:?}", error.kind()))
}

#[cfg(test)]
mod tests;
