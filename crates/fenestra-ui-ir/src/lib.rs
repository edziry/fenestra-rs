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
mod spatial;
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
    #[rustfmt::skip]
    pub use crate::ids::{
        SpatialFormatVersion, SUPPORTED_SPATIAL_FORMAT, SpatialNodeSymbolV2,
        SpatialShapeSymbolV2, SpatialBrushSymbolV2, SpatialClipSymbolV2,
        SpatialImageSymbolV2,
    };
    #[rustfmt::skip]
    pub use crate::spatial::{
        SpatialFieldV2, SpatialBindingV2, SpatialAxisV2,
        SpatialAnchorComponentV2, SpatialFillRuleV2, SpatialNodeParentV2,
        SpatialAnchorTargetRecipeV2, SpatialClipAddressV2, SpatialPointRecipeV2,
        SpatialPaddingRecipeV2, SpatialDimensionRecipeV2, SpatialTransformRecipeV2,
        SpatialViewportContainerV2, SpatialContainerRecipeV2,
        SpatialLayoutPlacementRecipeV2, SpatialFreePlacementRecipeV2,
        SpatialPlacementRecipeV2, SpatialPathVerbRecipeV2, SpatialPolygonPointV2,
        SpatialShapeGeometryV2, SpatialShapeDeclarationV2, SpatialGradientStopV2,
        SpatialBrushContentV2, SpatialBrushDeclarationV2, SpatialCoverageRecipeV2,
        SpatialClipDeclarationV2, SpatialPaintRecipeV2, SpatialHitRecipeV2,
        SpatialSemanticRecipeV2, SpatialImageDeclarationV2,
        SpatialNodeDeclarationV2, SpatialProgramV2,
    };
    pub use crate::limits::SpatialValidationLimitsV2;
    pub use crate::validated::ValidatedSpatialProgramV2;
    pub use crate::validation::validate_spatial;
}
