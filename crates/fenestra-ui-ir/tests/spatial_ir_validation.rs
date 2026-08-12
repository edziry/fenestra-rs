#[path = "spatial_ir_validation/support.rs"]
mod support;

use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InputPolicy,
    InvalidationClass, InvalidationSet, IrValidationError, IrValidationErrorKind, PropertyId,
    PropertySchema, PropertyValue, SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT,
    SUPPORTED_SPATIAL_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest, SchemaNamespace,
    SchemaRevision, SourceId, SourceSpan, SpatialAnchorComponentV2, SpatialAnchorTargetRecipeV2,
    SpatialAxisV2, SpatialBindingV2, SpatialBrushContentV2, SpatialBrushDeclarationV2,
    SpatialBrushSymbolV2, SpatialClipAddressV2, SpatialClipDeclarationV2, SpatialClipSymbolV2,
    SpatialContainerRecipeV2, SpatialCoverageRecipeV2, SpatialDimensionRecipeV2, SpatialFieldV2,
    SpatialFillRuleV2, SpatialFormatVersion, SpatialFreePlacementRecipeV2, SpatialGradientStopV2,
    SpatialHitRecipeV2, SpatialImageDeclarationV2, SpatialImageSymbolV2,
    SpatialLayoutPlacementRecipeV2, SpatialNodeDeclarationV2, SpatialNodeParentV2,
    SpatialNodeSymbolV2, SpatialPaddingRecipeV2, SpatialPaintRecipeV2, SpatialPathVerbRecipeV2,
    SpatialPlacementRecipeV2, SpatialPointRecipeV2, SpatialPolygonPointV2, SpatialProgramV2,
    SpatialSemanticRecipeV2, SpatialShapeDeclarationV2, SpatialShapeGeometryV2,
    SpatialShapeSymbolV2, SpatialTransformRecipeV2, SpatialValidationLimitsV2,
    SpatialViewportContainerV2, StructuralRegion, StructuralRegionId, StyleProgram,
    StyleValidationLimits, TemplateNode, TemplateNodeId, ValidatedSpatialProgramV2,
    ValidatedStyleProgram, ValidationLimitKind, ValidationLimits, ValueType, validate_construction,
    validate_schema, validate_spatial, validate_style,
};

#[path = "spatial_ir_validation/counts_source.rs"]
mod counts_source;
#[path = "spatial_ir_validation/deferred_semantics.rs"]
mod deferred_semantics;
#[path = "spatial_ir_validation/error_registry.rs"]
mod error_registry;
#[path = "spatial_ir_validation/global_priority.rs"]
mod global_priority;
#[path = "spatial_ir_validation/item_field_order.rs"]
mod item_field_order;
#[path = "spatial_ir_validation/item_references.rs"]
mod item_references;
#[path = "spatial_ir_validation/item_semantic_priority.rs"]
mod item_semantic_priority;
#[path = "spatial_ir_validation/layout_field_order.rs"]
mod layout_field_order;
#[path = "spatial_ir_validation/limit_priority.rs"]
mod limit_priority;
#[path = "spatial_ir_validation/limits.rs"]
mod limits;
#[path = "spatial_ir_validation/nodes_and_domains.rs"]
mod nodes_and_domains;
#[path = "spatial_ir_validation/owner_scopes.rs"]
mod owner_scopes;
#[path = "spatial_ir_validation/phase_semantic_priority.rs"]
mod phase_semantic_priority;
#[path = "spatial_ir_validation/program_and_fields.rs"]
mod program_and_fields;
#[path = "spatial_ir_validation/resource_bindings.rs"]
mod resource_bindings;
#[path = "spatial_ir_validation/resource_field_order.rs"]
mod resource_field_order;
#[path = "spatial_ir_validation/resources_and_references.rs"]
mod resources_and_references;
#[path = "spatial_ir_validation/semantic_priority.rs"]
mod semantic_priority;
#[path = "spatial_ir_validation/shape_field_order.rs"]
mod shape_field_order;
#[path = "spatial_ir_validation/source_spans.rs"]
mod source_spans;
#[path = "spatial_ir_validation/sparse_symbols.rs"]
mod sparse_symbols;
