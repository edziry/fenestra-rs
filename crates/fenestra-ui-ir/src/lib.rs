#![forbid(unsafe_code)]

//! Provisional typed intermediate representation for Fenestra experiments.
//!
//! Construction and style programs will remain distinct and linked through
//! shared typed schemas.

mod construction;
mod error;
mod ids;
mod invalidation;
mod limits;
mod schema;
mod source;
mod style;
mod validated;
mod validation;
mod value;

/// Unstable cross-crate surface used only by unpublished feasibility probes.
#[doc(hidden)]
pub mod prototype {
    pub use crate::construction::{
        ChildSlot, ConstructionProgram, InitialKey, InitialProperty, StructuralRegion, TemplateNode,
    };
    pub use crate::error::{IrValidationError, IrValidationErrorKind, ValidationLimitKind};
    pub use crate::ids::{
        ComponentTypeId, ConstructionFormatVersion, PropertyId, SUPPORTED_CONSTRUCTION_FORMAT,
        SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaFormatVersion, SchemaNamespace,
        SchemaRevision, SourceId, StructuralRegionId, StyleFormatVersion, TemplateNodeId,
    };
    pub use crate::invalidation::{InvalidationClass, InvalidationIter, InvalidationSet};
    pub use crate::limits::{StyleValidationLimits, ValidationLimits};
    pub use crate::schema::{ComponentSchema, PropertySchema, SchemaManifest};
    pub use crate::source::SourceSpan;
    pub use crate::style::{StyleAssignment, StyleProgram};
    pub use crate::validated::{
        ChildFactory, ChildFactoryIter, ComponentPropertiesIter, ComponentSchemaView,
        InitialKeyIter, InitialKeyView, InitialPropertyIter, InitialPropertyView,
        LinkedStyleValueView, PropertySchemaView, RegionFactory, StyleAssignmentIter,
        StyleAssignmentView, StyleValueOrigin, TemplateFactory, ValidatedConstruction,
        ValidatedSchema, ValidatedStyleProgram,
    };
    pub use crate::validation::{validate_construction, validate_schema, validate_style};
    pub use crate::value::{InputPolicy, PropertyValue, ValueType};
}
