use std::collections::{HashMap, HashSet};

use crate::error::{IrValidationError, IrValidationErrorKind, ValidationLimitKind};
use crate::ids::SUPPORTED_SCHEMA_FORMAT;
use crate::invalidation::InvalidationClass;
use crate::limits::ValidationLimits;
use crate::schema::SchemaManifest;
use crate::validated::ValidatedSchema;

use super::{add_count, failure, limit_failure};

/// Validates a schema manifest into an immutable receiver-scoped domain.
pub fn validate_schema(
    manifest: SchemaManifest,
    limits: ValidationLimits,
) -> Result<ValidatedSchema, IrValidationError> {
    if !manifest.span.is_valid() {
        return Err(failure(
            IrValidationErrorKind::InvalidSourceSpan,
            manifest.span,
        ));
    }
    if manifest.format != SUPPORTED_SCHEMA_FORMAT {
        return Err(failure(
            IrValidationErrorKind::UnsupportedSchemaFormat,
            manifest.span,
        ));
    }

    preflight_counts(&manifest, limits)?;

    let mut component_indexes = HashMap::new();
    let mut property_indexes = Vec::with_capacity(manifest.components.len());
    let mut seen_components = HashSet::new();

    for (component_index, component) in manifest.components.iter().enumerate() {
        if !component.span.is_valid() {
            return Err(failure(
                IrValidationErrorKind::InvalidSourceSpan,
                component.span,
            ));
        }
        if !seen_components.insert(component.id) {
            return Err(failure(
                IrValidationErrorKind::DuplicateComponent,
                component.span,
            ));
        }
        component_indexes.insert(component.id, component_index);

        let mut indexes = HashMap::new();
        let mut seen_properties = HashSet::new();
        for (property_index, property) in component.properties.iter().enumerate() {
            if !property.span.is_valid() {
                return Err(failure(
                    IrValidationErrorKind::InvalidSourceSpan,
                    property.span,
                ));
            }
            if !seen_properties.insert(property.id) {
                return Err(failure(
                    IrValidationErrorKind::DuplicateProperty,
                    property.span,
                ));
            }
            if property.default.value_type() != property.value_type {
                return Err(failure(
                    IrValidationErrorKind::PropertyDefaultTypeMismatch,
                    property.span,
                ));
            }
            if property.invalidation.is_empty() {
                return Err(failure(
                    IrValidationErrorKind::EmptyPropertyInvalidation,
                    property.span,
                ));
            }
            if property.invalidation.contains(InvalidationClass::Structure)
                || property.invalidation.contains(InvalidationClass::Surface)
            {
                return Err(failure(
                    IrValidationErrorKind::InvalidPropertyInvalidation,
                    property.span,
                ));
            }
            indexes.insert(property.id, property_index);
        }
        property_indexes.push(indexes);
    }

    Ok(ValidatedSchema::new(
        manifest,
        component_indexes,
        property_indexes,
    ))
}

fn preflight_counts(
    manifest: &SchemaManifest,
    limits: ValidationLimits,
) -> Result<(), IrValidationError> {
    if manifest.components.len() > limits.components() {
        let crossing = &manifest.components[limits.components()];
        return Err(limit_failure(
            ValidationLimitKind::Components,
            crossing.span,
        ));
    }

    let mut properties = 0;
    for component in &manifest.components {
        for property in &component.properties {
            add_count(
                &mut properties,
                limits.properties(),
                ValidationLimitKind::Properties,
                property.span,
            )?;
        }
    }
    Ok(())
}
