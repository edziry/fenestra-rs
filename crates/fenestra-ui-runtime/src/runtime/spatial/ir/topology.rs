use fenestra_ui_ir::prototype::{
    SourceSpan, SpatialAnchorComponentV2 as IrAnchor, SpatialAnchorTargetRecipeV2,
    SpatialAxisV2 as IrAxis, SpatialDimensionRecipeV2, SpatialFreePlacementRecipeV2,
    SpatialLayoutPlacementRecipeV2, SpatialNodeDeclarationV2, SpatialNodeParentV2,
    SpatialPlacementRecipeV2, SpatialTransformRecipeV2, ValidatedSpatialProgramV2,
};
use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};
use fenestra_ui_spatial::prototype::{
    Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialAnchorV2,
    SpatialContainerV2, SpatialFreePlacementV2, SpatialLayoutPlacementV2, SpatialLocalTransformV2,
    SpatialNodeFieldV2 as Field, SpatialNodeKeyV2, SpatialNodeV2, SpatialOffsetV2,
    SpatialPlacementV2,
};

use super::super::error::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};
use super::super::view::RuntimeSpatialBuildViewV2;
use super::bindings;
use super::model::{ExpandedSpatialNode, LiveProgram};
use super::provenance::FieldSpans;
use crate::logical_tree::NodeId;

pub(super) struct TopologyTables {
    pub(super) nodes: Vec<SpatialNodeV2>,
    pub(super) logical_nodes: Box<[NodeId]>,
    pub(super) provenance: Vec<FieldSpans<Field>>,
}

pub(super) fn materialize(
    program: &ValidatedSpatialProgramV2,
    live: &LiveProgram<'_>,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<TopologyTables, RuntimeSpatialIrErrorV2> {
    let raw = program.program();
    let viewport = raw.viewport_container();
    let program_span = raw.span();
    let sentinel_container = SpatialContainerV2::new(
        axis(viewport.axis()),
        LayoutPaddingV1::new(
            *viewport.left().value(),
            *viewport.right().value(),
            *viewport.top().value(),
            *viewport.bottom().value(),
        ),
        *viewport.gap().value(),
    );
    let mut nodes = vec![SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        sentinel_container,
    )];
    let mut provenance = vec![sentinel_provenance(program_span, viewport)];

    for expanded in live.expanded() {
        let declaration = expanded.declaration();
        let placement = placement(live, expanded, declaration, view)?;
        let container = container(declaration, expanded.logical(), view)?;
        nodes.push(SpatialNodeV2::new(
            SpatialNodeKeyV2::new(expanded.key()),
            Some(SpatialNodeKeyV2::new(expanded.parent_key())),
            placement,
            container,
        ));
        provenance.push(node_provenance(declaration));
    }

    Ok(TopologyTables {
        nodes,
        logical_nodes: live.logical_mapping(),
        provenance,
    })
}

fn placement(
    live: &LiveProgram<'_>,
    expanded: &ExpandedSpatialNode<'_>,
    declaration: &SpatialNodeDeclarationV2,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<SpatialPlacementV2, RuntimeSpatialIrErrorV2> {
    let owner = expanded.logical();
    match declaration.placement() {
        SpatialPlacementRecipeV2::Layout(recipe) => {
            Ok(SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
                dimension(recipe.width(), owner, view)?,
                dimension(recipe.height(), owner, view)?,
                transform(recipe.transform(), owner, view)?,
            )))
        }
        SpatialPlacementRecipeV2::Free(recipe) => {
            let target = match recipe.target() {
                SpatialAnchorTargetRecipeV2::Viewport => SpatialAnchorTargetV2::Viewport,
                SpatialAnchorTargetRecipeV2::Parent => SpatialAnchorTargetV2::Parent,
                SpatialAnchorTargetRecipeV2::Node(symbol) => {
                    let target = live
                        .resolve_node(expanded.context(), *symbol.value())
                        .ok_or_else(|| invariant(symbol.span()))?;
                    SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(target.key()))
                }
            };
            Ok(SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
                bindings::i32_value(recipe.width(), owner, view)?,
                bindings::i32_value(recipe.height(), owner, view)?,
                anchor(recipe.self_anchor()),
                target,
                anchor(recipe.target_anchor()),
                SpatialOffsetV2::new(
                    bindings::scalar(recipe.offset().x(), owner, view)?,
                    bindings::scalar(recipe.offset().y(), owner, view)?,
                ),
                transform(recipe.transform(), owner, view)?,
            )))
        }
    }
}

fn container(
    declaration: &SpatialNodeDeclarationV2,
    owner: NodeId,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<SpatialContainerV2, RuntimeSpatialIrErrorV2> {
    let recipe = declaration.container();
    let padding = recipe.padding();
    Ok(SpatialContainerV2::new(
        axis(recipe.axis()),
        LayoutPaddingV1::new(
            bindings::i32_value(padding.left(), owner, view)?,
            bindings::i32_value(padding.right(), owner, view)?,
            bindings::i32_value(padding.top(), owner, view)?,
            bindings::i32_value(padding.bottom(), owner, view)?,
        ),
        bindings::i32_value(recipe.gap(), owner, view)?,
    ))
}

fn dimension(
    recipe: SpatialDimensionRecipeV2,
    owner: NodeId,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<LayoutDimensionV1, RuntimeSpatialIrErrorV2> {
    Ok(LayoutDimensionV1::new(
        bindings::i32_value(recipe.minimum(), owner, view)?,
        bindings::i32_value(recipe.preferred(), owner, view)?,
        bindings::i32_value(recipe.maximum(), owner, view)?,
    ))
}

fn transform(
    recipe: SpatialTransformRecipeV2,
    owner: NodeId,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<SpatialLocalTransformV2, RuntimeSpatialIrErrorV2> {
    Ok(SpatialLocalTransformV2::new(
        Affine2V2::new(
            bindings::scalar(recipe.a(), owner, view)?,
            bindings::scalar(recipe.b(), owner, view)?,
            bindings::scalar(recipe.c(), owner, view)?,
            bindings::scalar(recipe.d(), owner, view)?,
            bindings::scalar(recipe.tx(), owner, view)?,
            bindings::scalar(recipe.ty(), owner, view)?,
        ),
        bindings::point(recipe.origin(), owner, view)?,
    ))
}

fn sentinel_provenance(
    program_span: SourceSpan,
    viewport: fenestra_ui_ir::prototype::SpatialViewportContainerV2,
) -> FieldSpans<Field> {
    FieldSpans::new(
        program_span,
        vec![
            (Field::Key, SourceSpan::Synthetic),
            (Field::Parent, SourceSpan::Synthetic),
            (Field::Placement, SourceSpan::Synthetic),
            (Field::ContainerAxis, viewport.span()),
            (Field::PaddingLeft, viewport.left().span()),
            (Field::PaddingRight, viewport.right().span()),
            (Field::PaddingTop, viewport.top().span()),
            (Field::PaddingBottom, viewport.bottom().span()),
            (Field::Gap, viewport.gap().span()),
        ],
    )
}

fn node_provenance(declaration: &SpatialNodeDeclarationV2) -> FieldSpans<Field> {
    let record = declaration.span();
    let mut fields = vec![(Field::Key, record), (Field::Placement, record)];
    fields.push((
        Field::Parent,
        match declaration.parent() {
            SpatialNodeParentV2::Viewport => record,
            SpatialNodeParentV2::Node(parent) => parent.span(),
        },
    ));
    match declaration.placement() {
        SpatialPlacementRecipeV2::Layout(recipe) => layout_spans(&mut fields, recipe),
        SpatialPlacementRecipeV2::Free(recipe) => free_spans(&mut fields, recipe, record),
    }
    let container = declaration.container();
    let padding = container.padding();
    fields.extend([
        (Field::ContainerAxis, record),
        (Field::PaddingLeft, padding.left().span()),
        (Field::PaddingRight, padding.right().span()),
        (Field::PaddingTop, padding.top().span()),
        (Field::PaddingBottom, padding.bottom().span()),
        (Field::Gap, container.gap().span()),
    ]);
    FieldSpans::new(record, fields)
}

fn layout_spans(fields: &mut Vec<(Field, SourceSpan)>, recipe: SpatialLayoutPlacementRecipeV2) {
    let width = recipe.width();
    let height = recipe.height();
    fields.extend([
        (Field::LayoutWidthMinimum, width.minimum().span()),
        (Field::LayoutWidthPreferred, width.preferred().span()),
        (Field::LayoutWidthMaximum, width.maximum().span()),
        (Field::LayoutHeightMinimum, height.minimum().span()),
        (Field::LayoutHeightPreferred, height.preferred().span()),
        (Field::LayoutHeightMaximum, height.maximum().span()),
    ]);
    transform_spans(fields, recipe.transform());
}

fn free_spans(
    fields: &mut Vec<(Field, SourceSpan)>,
    recipe: SpatialFreePlacementRecipeV2,
    record: SourceSpan,
) {
    let target_span = match recipe.target() {
        SpatialAnchorTargetRecipeV2::Node(target) => target.span(),
        SpatialAnchorTargetRecipeV2::Viewport | SpatialAnchorTargetRecipeV2::Parent => record,
    };
    fields.extend([
        (Field::FreeWidth, recipe.width().span()),
        (Field::FreeHeight, recipe.height().span()),
        (Field::SelfAnchorHorizontal, record),
        (Field::SelfAnchorVertical, record),
        (Field::TargetKind, target_span),
        (Field::TargetAnchorHorizontal, record),
        (Field::TargetAnchorVertical, record),
        (Field::FreeOffsetX, recipe.offset().x().span()),
        (Field::FreeOffsetY, recipe.offset().y().span()),
    ]);
    if matches!(recipe.target(), SpatialAnchorTargetRecipeV2::Node(_)) {
        fields.push((Field::TargetKey, target_span));
    }
    transform_spans(fields, recipe.transform());
}

fn transform_spans(fields: &mut Vec<(Field, SourceSpan)>, recipe: SpatialTransformRecipeV2) {
    fields.extend([
        (Field::AffineA, recipe.a().span()),
        (Field::AffineB, recipe.b().span()),
        (Field::AffineC, recipe.c().span()),
        (Field::AffineD, recipe.d().span()),
        (Field::AffineTx, recipe.tx().span()),
        (Field::AffineTy, recipe.ty().span()),
        (Field::TransformOriginX, recipe.origin().x().span()),
        (Field::TransformOriginY, recipe.origin().y().span()),
    ]);
}

const fn axis(value: IrAxis) -> LayoutAxisV1 {
    match value {
        IrAxis::Row => LayoutAxisV1::Row,
        IrAxis::Column => LayoutAxisV1::Column,
    }
}

const fn anchor(value: [IrAnchor; 2]) -> SpatialAnchorV2 {
    SpatialAnchorV2::new(anchor_component(value[0]), anchor_component(value[1]))
}

const fn anchor_component(value: IrAnchor) -> SpatialAnchorComponentV2 {
    match value {
        IrAnchor::Start => SpatialAnchorComponentV2::Start,
        IrAnchor::Center => SpatialAnchorComponentV2::Center,
        IrAnchor::End => SpatialAnchorComponentV2::End,
    }
}

fn invariant(span: SourceSpan) -> RuntimeSpatialIrErrorV2 {
    RuntimeSpatialIrErrorV2::new(RuntimeSpatialIrErrorKindV2::InvariantViolation, span)
}
