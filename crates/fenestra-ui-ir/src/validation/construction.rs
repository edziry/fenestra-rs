use std::collections::{HashMap, HashSet};

use crate::construction::{ChildSlotKind, ConstructionProgram};
use crate::error::{IrValidationError, IrValidationErrorKind, ValidationLimitKind};
use crate::ids::{SUPPORTED_CONSTRUCTION_FORMAT, StructuralRegionId, TemplateNodeId};
use crate::invalidation::InvalidationClass;
use crate::limits::ValidationLimits;
use crate::validated::{ValidatedConstruction, ValidatedSchema};

use super::graph::validate_ownership_and_limits;
use super::{add_count, failure, limit_failure};

/// Validates and links a construction program to one exact schema domain.
pub fn validate_construction(
    schema: &ValidatedSchema,
    program: ConstructionProgram,
    limits: ValidationLimits,
) -> Result<ValidatedConstruction, IrValidationError> {
    if !program.span.is_valid() {
        return Err(failure(
            IrValidationErrorKind::InvalidSourceSpan,
            program.span,
        ));
    }
    if program.format != SUPPORTED_CONSTRUCTION_FORMAT {
        return Err(failure(
            IrValidationErrorKind::UnsupportedConstructionFormat,
            program.span,
        ));
    }
    let manifest = &schema.data.manifest;
    if program.schema_namespace != manifest.namespace
        || program.schema_revision != manifest.revision
    {
        return Err(failure(
            IrValidationErrorKind::SchemaIdentityMismatch,
            program.span,
        ));
    }

    preflight_counts(&program, limits)?;
    let node_indexes = first_node_indexes(&program);
    let region_indexes = first_region_indexes(&program);
    validate_records(schema, &program, &node_indexes, &region_indexes)?;
    validate_region_placement(&program, &region_indexes)?;
    let root = validate_ownership_and_limits(&program, &node_indexes, &region_indexes, limits)?;

    Ok(ValidatedConstruction::new(
        schema.clone(),
        program,
        node_indexes,
        region_indexes,
        root,
    ))
}

fn preflight_counts(
    program: &ConstructionProgram,
    limits: ValidationLimits,
) -> Result<(), IrValidationError> {
    if program.nodes.len() > limits.templates() {
        return Err(limit_failure(
            ValidationLimitKind::Templates,
            program.nodes[limits.templates()].span,
        ));
    }
    if program.regions.len() > limits.regions() {
        return Err(limit_failure(
            ValidationLimitKind::Regions,
            program.regions[limits.regions()].span,
        ));
    }

    let mut slots = 0;
    for node in &program.nodes {
        for slot in &node.children {
            add_count(
                &mut slots,
                limits.child_slots(),
                ValidationLimitKind::ChildSlots,
                slot.span(),
            )?;
        }
    }

    let mut properties = 0;
    for node in &program.nodes {
        for property in &node.initial_properties {
            add_count(
                &mut properties,
                limits.initial_properties(),
                ValidationLimitKind::InitialProperties,
                property.span,
            )?;
        }
    }

    let mut keys = 0;
    for region in &program.regions {
        for key in &region.initial_keys {
            add_count(
                &mut keys,
                limits.initial_keys(),
                ValidationLimitKind::InitialKeys,
                key.span,
            )?;
        }
    }
    Ok(())
}

fn first_node_indexes(program: &ConstructionProgram) -> HashMap<TemplateNodeId, usize> {
    let mut indexes = HashMap::new();
    for (index, node) in program.nodes.iter().enumerate() {
        indexes.entry(node.id).or_insert(index);
    }
    indexes
}

fn first_region_indexes(program: &ConstructionProgram) -> HashMap<StructuralRegionId, usize> {
    let mut indexes = HashMap::new();
    for (index, region) in program.regions.iter().enumerate() {
        indexes.entry(region.id).or_insert(index);
    }
    indexes
}

fn validate_records(
    schema: &ValidatedSchema,
    program: &ConstructionProgram,
    nodes: &HashMap<TemplateNodeId, usize>,
    regions: &HashMap<StructuralRegionId, usize>,
) -> Result<(), IrValidationError> {
    let mut seen_nodes = HashSet::new();
    for node in &program.nodes {
        if !node.span.is_valid() {
            return Err(failure(IrValidationErrorKind::InvalidSourceSpan, node.span));
        }
        if !seen_nodes.insert(node.id) {
            return Err(failure(IrValidationErrorKind::DuplicateNode, node.span));
        }
        let component = schema
            .component(node.component)
            .ok_or_else(|| failure(IrValidationErrorKind::MissingComponent, node.span))?;

        let mut seen_properties = HashSet::new();
        for property in &node.initial_properties {
            if !property.span.is_valid() {
                return Err(failure(
                    IrValidationErrorKind::InvalidSourceSpan,
                    property.span,
                ));
            }
            if !seen_properties.insert(property.property) {
                return Err(failure(
                    IrValidationErrorKind::DuplicateInitialProperty,
                    property.span,
                ));
            }
            let declared = component.property(property.property).ok_or_else(|| {
                failure(IrValidationErrorKind::UnknownInitialProperty, property.span)
            })?;
            if declared.value_type() != property.value.value_type() {
                return Err(failure(
                    IrValidationErrorKind::InitialPropertyTypeMismatch,
                    property.span,
                ));
            }
        }

        for slot in &node.children {
            if !slot.span().is_valid() {
                return Err(failure(
                    IrValidationErrorKind::InvalidSourceSpan,
                    slot.span(),
                ));
            }
            match slot.kind {
                ChildSlotKind::Static(child) if !nodes.contains_key(&child) => {
                    return Err(failure(
                        IrValidationErrorKind::MissingStaticChild,
                        slot.span,
                    ));
                }
                ChildSlotKind::Region(region) if !regions.contains_key(&region) => {
                    return Err(failure(IrValidationErrorKind::MissingRegion, slot.span));
                }
                ChildSlotKind::Static(_) | ChildSlotKind::Region(_) => {}
            }
        }
    }

    let mut seen_regions = HashSet::new();
    for region in &program.regions {
        if !region.span.is_valid() {
            return Err(failure(
                IrValidationErrorKind::InvalidSourceSpan,
                region.span,
            ));
        }
        if !seen_regions.insert(region.id) {
            return Err(failure(IrValidationErrorKind::DuplicateRegion, region.span));
        }
        if !nodes.contains_key(&region.owner) {
            return Err(failure(
                IrValidationErrorKind::MissingRegionOwner,
                region.span,
            ));
        }
        if !nodes.contains_key(&region.repeat_body) {
            return Err(failure(
                IrValidationErrorKind::MissingRegionTemplate,
                region.span,
            ));
        }

        let mut seen_keys = HashSet::new();
        for key in &region.initial_keys {
            if !key.span.is_valid() {
                return Err(failure(IrValidationErrorKind::InvalidSourceSpan, key.span));
            }
            if !seen_keys.insert(key.value) {
                return Err(failure(IrValidationErrorKind::DuplicateRegionKey, key.span));
            }
        }
        if region.invalidation.is_empty()
            || !region.invalidation.contains(InvalidationClass::Structure)
            || region.invalidation.contains(InvalidationClass::Surface)
        {
            return Err(failure(
                IrValidationErrorKind::InvalidRegionInvalidation,
                region.span,
            ));
        }
    }
    Ok(())
}

fn validate_region_placement(
    program: &ConstructionProgram,
    regions: &HashMap<StructuralRegionId, usize>,
) -> Result<(), IrValidationError> {
    let mut placements = vec![false; program.regions.len()];
    for node in &program.nodes {
        for slot in &node.children {
            let ChildSlotKind::Region(region) = slot.kind else {
                continue;
            };
            let region_index = regions[&region];
            let declared = &program.regions[region_index];
            if declared.owner != node.id {
                return Err(failure(
                    IrValidationErrorKind::RegionOwnerMismatch,
                    slot.span,
                ));
            }
            if placements[region_index] {
                return Err(failure(
                    IrValidationErrorKind::DuplicateRegionPlacement,
                    slot.span,
                ));
            }
            placements[region_index] = true;
        }
    }

    for (index, region) in program.regions.iter().enumerate() {
        if !placements[index] {
            return Err(failure(IrValidationErrorKind::UnplacedRegion, region.span));
        }
    }
    Ok(())
}
