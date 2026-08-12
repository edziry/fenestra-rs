use crate::*;
use fenestra_ui_ir::prototype::{
    SchemaNamespace, SchemaRevision, SourceSpan, StructuralRegionId, TemplateNodeId,
    ValidatedStyleProgram,
};

#[test]
fn owned_shape_brush_and_image_signatures_are_exact() {
    let _: fn(
        SpatialFieldV2<SpatialShapeSymbolV2>,
        SpatialShapeGeometryV2,
        SourceSpan,
    ) -> SpatialShapeDeclarationV2 = SpatialShapeDeclarationV2::new;
    let _: fn(&SpatialShapeDeclarationV2) -> SpatialFieldV2<SpatialShapeSymbolV2> =
        SpatialShapeDeclarationV2::symbol;
    let _: for<'a> fn(&'a SpatialShapeDeclarationV2) -> &'a SpatialShapeGeometryV2 =
        SpatialShapeDeclarationV2::geometry;
    let _: fn(&SpatialShapeDeclarationV2) -> SourceSpan = SpatialShapeDeclarationV2::span;

    let _: fn(
        SpatialFieldV2<SpatialBrushSymbolV2>,
        SpatialBrushContentV2,
        SourceSpan,
    ) -> SpatialBrushDeclarationV2 = SpatialBrushDeclarationV2::new;
    let _: fn(&SpatialBrushDeclarationV2) -> SpatialFieldV2<SpatialBrushSymbolV2> =
        SpatialBrushDeclarationV2::symbol;
    let _: for<'a> fn(&'a SpatialBrushDeclarationV2) -> &'a SpatialBrushContentV2 =
        SpatialBrushDeclarationV2::content;
    let _: fn(&SpatialBrushDeclarationV2) -> SourceSpan = SpatialBrushDeclarationV2::span;

    assert_image_signatures();
}

#[allow(clippy::type_complexity)]
fn assert_image_signatures() {
    let _: fn(
        SpatialFieldV2<SpatialImageSymbolV2>,
        SpatialFieldV2<u32>,
        SpatialFieldV2<u32>,
        SpatialFieldV2<u32>,
        Box<[u8]>,
        SourceSpan,
    ) -> SpatialImageDeclarationV2 = SpatialImageDeclarationV2::new;
    let _: fn(&SpatialImageDeclarationV2) -> SpatialFieldV2<SpatialImageSymbolV2> =
        SpatialImageDeclarationV2::symbol;
    let _: fn(&SpatialImageDeclarationV2) -> SpatialFieldV2<u32> = SpatialImageDeclarationV2::width;
    let _: fn(&SpatialImageDeclarationV2) -> SpatialFieldV2<u32> =
        SpatialImageDeclarationV2::height;
    let _: fn(&SpatialImageDeclarationV2) -> SpatialFieldV2<u32> =
        SpatialImageDeclarationV2::stride;
    let _: for<'a> fn(&'a SpatialImageDeclarationV2) -> &'a [u8] = SpatialImageDeclarationV2::bytes;
    let _: fn(&SpatialImageDeclarationV2) -> SourceSpan = SpatialImageDeclarationV2::span;
}

type NodeConstructor = fn(
    SpatialFieldV2<SpatialNodeSymbolV2>,
    SpatialFieldV2<TemplateNodeId>,
    SpatialNodeParentV2,
    SpatialPlacementRecipeV2,
    SpatialContainerRecipeV2,
    Vec<SpatialShapeDeclarationV2>,
    Vec<SpatialBrushDeclarationV2>,
    Vec<SpatialClipDeclarationV2>,
    Vec<SpatialPaintRecipeV2>,
    Vec<SpatialHitRecipeV2>,
    Vec<SpatialSemanticRecipeV2>,
    SourceSpan,
) -> SpatialNodeDeclarationV2;

#[test]
fn node_declaration_signatures_are_exact() {
    let _: NodeConstructor = SpatialNodeDeclarationV2::new;
    assert_node_signatures();
}

fn assert_node_signatures() {
    let _: fn(&SpatialNodeDeclarationV2) -> SpatialFieldV2<SpatialNodeSymbolV2> =
        SpatialNodeDeclarationV2::symbol;
    let _: fn(&SpatialNodeDeclarationV2) -> SpatialFieldV2<TemplateNodeId> =
        SpatialNodeDeclarationV2::template;
    let _: fn(&SpatialNodeDeclarationV2) -> SpatialNodeParentV2 = SpatialNodeDeclarationV2::parent;
    let _: fn(&SpatialNodeDeclarationV2) -> SpatialPlacementRecipeV2 =
        SpatialNodeDeclarationV2::placement;
    let _: fn(&SpatialNodeDeclarationV2) -> SpatialContainerRecipeV2 =
        SpatialNodeDeclarationV2::container;
    let _: for<'a> fn(&'a SpatialNodeDeclarationV2) -> &'a [SpatialShapeDeclarationV2] =
        SpatialNodeDeclarationV2::shapes;
    let _: for<'a> fn(&'a SpatialNodeDeclarationV2) -> &'a [SpatialBrushDeclarationV2] =
        SpatialNodeDeclarationV2::brushes;
    let _: for<'a> fn(&'a SpatialNodeDeclarationV2) -> &'a [SpatialClipDeclarationV2] =
        SpatialNodeDeclarationV2::clips;
    let _: for<'a> fn(&'a SpatialNodeDeclarationV2) -> &'a [SpatialPaintRecipeV2] =
        SpatialNodeDeclarationV2::paint_items;
    let _: for<'a> fn(&'a SpatialNodeDeclarationV2) -> &'a [SpatialHitRecipeV2] =
        SpatialNodeDeclarationV2::hit_items;
    let _: for<'a> fn(&'a SpatialNodeDeclarationV2) -> &'a [SpatialSemanticRecipeV2] =
        SpatialNodeDeclarationV2::semantic_items;
    let _: fn(&SpatialNodeDeclarationV2) -> SourceSpan = SpatialNodeDeclarationV2::span;
}

#[test]
#[allow(clippy::type_complexity)]
fn program_validation_and_validated_view_signatures_are_exact() {
    let _: fn(
        SpatialFormatVersion,
        SchemaNamespace,
        SchemaRevision,
        SpatialViewportContainerV2,
        Vec<SpatialNodeDeclarationV2>,
        Vec<SpatialImageDeclarationV2>,
        SourceSpan,
    ) -> SpatialProgramV2 = SpatialProgramV2::new;
    assert_program_getters();

    let _: fn([usize; 13]) -> SpatialValidationLimitsV2 = SpatialValidationLimitsV2::new;
    let _: fn(
        &ValidatedStyleProgram,
        SpatialProgramV2,
        SpatialValidationLimitsV2,
    )
        -> Result<ValidatedSpatialProgramV2, fenestra_ui_ir::prototype::IrValidationError> =
        validate_spatial;

    assert_validated_getters();
}

fn assert_program_getters() {
    let _: fn(&SpatialProgramV2) -> SpatialFormatVersion = SpatialProgramV2::format;
    let _: fn(&SpatialProgramV2) -> SchemaNamespace = SpatialProgramV2::schema_namespace;
    let _: fn(&SpatialProgramV2) -> SchemaRevision = SpatialProgramV2::schema_revision;
    let _: fn(&SpatialProgramV2) -> SpatialViewportContainerV2 =
        SpatialProgramV2::viewport_container;
    let _: for<'a> fn(&'a SpatialProgramV2) -> &'a [SpatialNodeDeclarationV2] =
        SpatialProgramV2::nodes;
    let _: for<'a> fn(&'a SpatialProgramV2) -> &'a [SpatialImageDeclarationV2] =
        SpatialProgramV2::images;
    let _: fn(&SpatialProgramV2) -> SourceSpan = SpatialProgramV2::span;
}

fn assert_validated_getters() {
    let _: for<'a> fn(&'a ValidatedSpatialProgramV2) -> &'a SpatialProgramV2 =
        ValidatedSpatialProgramV2::program;
    let _: for<'a> fn(&'a ValidatedSpatialProgramV2) -> &'a ValidatedStyleProgram =
        ValidatedSpatialProgramV2::style;
    let _: for<'a> fn(
        &'a ValidatedSpatialProgramV2,
        SpatialNodeSymbolV2,
    ) -> Option<&'a SpatialNodeDeclarationV2> = ValidatedSpatialProgramV2::node;
    let _: for<'a> fn(
        &'a ValidatedSpatialProgramV2,
        TemplateNodeId,
    ) -> Option<&'a SpatialNodeDeclarationV2> = ValidatedSpatialProgramV2::node_for_template;
    let _: for<'a> fn(
        &'a ValidatedSpatialProgramV2,
        SpatialNodeSymbolV2,
    ) -> Option<&'a [StructuralRegionId]> = ValidatedSpatialProgramV2::region_signature;
    let _: fn(&ValidatedSpatialProgramV2, &ValidatedSpatialProgramV2) -> bool =
        ValidatedSpatialProgramV2::shares_domain_with;
}

#[test]
fn payload_enum_receiver_signatures_are_exact() {
    let _: fn(&SpatialPathVerbRecipeV2) -> SourceSpan = SpatialPathVerbRecipeV2::span;
    let _: fn(&SpatialPaintRecipeV2) -> SourceSpan = SpatialPaintRecipeV2::span;
}
