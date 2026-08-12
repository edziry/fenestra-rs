use fenestra_ui_ir::prototype::{
    SUPPORTED_SPATIAL_FORMAT, SchemaNamespace, SchemaRevision, SourceSpan,
    SpatialAnchorComponentV2, SpatialAnchorTargetRecipeV2, SpatialAxisV2, SpatialBindingV2,
    SpatialContainerRecipeV2, SpatialDimensionRecipeV2, SpatialFieldV2,
    SpatialFreePlacementRecipeV2, SpatialLayoutPlacementRecipeV2, SpatialNodeDeclarationV2,
    SpatialNodeParentV2, SpatialNodeSymbolV2, SpatialPaddingRecipeV2, SpatialPlacementRecipeV2,
    SpatialPointRecipeV2, SpatialProgramV2, SpatialTransformRecipeV2, SpatialViewportContainerV2,
};

use super::mapper_content::content;
use super::{INNER, LEAF, NODE_ANCHOR, OUTER, VIEW_ANCHOR, span};

const NAMESPACE: SchemaNamespace = SchemaNamespace::new(9001);
const REVISION: SchemaRevision = SchemaRevision::new(2);
pub(super) const OUTER_SYMBOL: SpatialNodeSymbolV2 = SpatialNodeSymbolV2::new(201);
const INNER_SYMBOL: SpatialNodeSymbolV2 = SpatialNodeSymbolV2::new(301);
const LEAF_SYMBOL: SpatialNodeSymbolV2 = SpatialNodeSymbolV2::new(401);
const NODE_ANCHOR_SYMBOL: SpatialNodeSymbolV2 = SpatialNodeSymbolV2::new(501);
const VIEW_ANCHOR_SYMBOL: SpatialNodeSymbolV2 = SpatialNodeSymbolV2::new(601);
pub(super) const SCALE: i64 = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapperFault {
    None,
    PathVerb,
    GradientStop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MapperSpans {
    pub program: SourceSpan,
    pub path_first_verb: SourceSpan,
    pub gradient_last_offset: SourceSpan,
}

pub(super) fn program(fault: MapperFault) -> (SpatialProgramV2, MapperSpans) {
    let mut spans = SpanCursor::new(500);
    let viewport = viewport(&mut spans);
    let (outer, content_spans) = outer(&mut spans, fault);
    let super::mapper_content::ContentSpans {
        image,
        path_first_verb,
        gradient_last_offset,
    } = content_spans;
    let inner = inner(&mut spans);
    let leaf = leaf(&mut spans);
    let node_anchor = node_anchor(&mut spans);
    let viewport_anchor = viewport_anchor(&mut spans);
    let program_span = spans.take();
    (
        SpatialProgramV2::new(
            SUPPORTED_SPATIAL_FORMAT,
            NAMESPACE,
            REVISION,
            viewport,
            vec![outer, inner, leaf, node_anchor, viewport_anchor],
            vec![image],
            program_span,
        ),
        MapperSpans {
            program: program_span,
            path_first_verb,
            gradient_last_offset,
        },
    )
}

fn viewport(spans: &mut SpanCursor) -> SpatialViewportContainerV2 {
    SpatialViewportContainerV2::new(
        SpatialAxisV2::Column,
        spans.field(0),
        spans.field(0),
        spans.field(0),
        spans.field(0),
        spans.field(0),
        spans.take(),
    )
}

fn outer(
    spans: &mut SpanCursor,
    fault: MapperFault,
) -> (
    SpatialNodeDeclarationV2,
    super::mapper_content::ContentSpans,
) {
    let content = content(spans, fault);
    let node = SpatialNodeDeclarationV2::new(
        spans.field(OUTER_SYMBOL),
        spans.field(OUTER),
        SpatialNodeParentV2::Viewport,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            dimension(spans, 12),
            dimension(spans, 10),
            identity(spans),
        )),
        container(spans),
        content.shapes,
        content.brushes,
        content.clips,
        content.paints,
        content.hits,
        content.semantics,
        spans.take(),
    );
    (node, content.spans)
}

fn inner(spans: &mut SpanCursor) -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        spans.field(INNER_SYMBOL),
        spans.field(INNER),
        SpatialNodeParentV2::Node(spans.field(OUTER_SYMBOL)),
        free(
            spans,
            SpatialAnchorTargetRecipeV2::Parent,
            [SpatialAnchorComponentV2::Start; 2],
            [SpatialAnchorComponentV2::Center; 2],
        ),
        container(spans),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        spans.take(),
    )
}

fn leaf(spans: &mut SpanCursor) -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        spans.field(LEAF_SYMBOL),
        spans.field(LEAF),
        SpatialNodeParentV2::Node(spans.field(INNER_SYMBOL)),
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            dimension(spans, 3),
            dimension(spans, 2),
            quarter_turn(spans),
        )),
        container(spans),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        spans.take(),
    )
}

fn node_anchor(spans: &mut SpanCursor) -> SpatialNodeDeclarationV2 {
    let target = SpatialAnchorTargetRecipeV2::Node(spans.field(OUTER_SYMBOL));
    SpatialNodeDeclarationV2::new(
        spans.field(NODE_ANCHOR_SYMBOL),
        spans.field(NODE_ANCHOR),
        SpatialNodeParentV2::Node(spans.field(LEAF_SYMBOL)),
        free(
            spans,
            target,
            [SpatialAnchorComponentV2::Center; 2],
            [SpatialAnchorComponentV2::End; 2],
        ),
        container(spans),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        spans.take(),
    )
}

fn viewport_anchor(spans: &mut SpanCursor) -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        spans.field(VIEW_ANCHOR_SYMBOL),
        spans.field(VIEW_ANCHOR),
        SpatialNodeParentV2::Node(spans.field(LEAF_SYMBOL)),
        free(
            spans,
            SpatialAnchorTargetRecipeV2::Viewport,
            [SpatialAnchorComponentV2::End; 2],
            [SpatialAnchorComponentV2::Start; 2],
        ),
        container(spans),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        spans.take(),
    )
}

fn free(
    spans: &mut SpanCursor,
    target: SpatialAnchorTargetRecipeV2,
    self_anchor: [SpatialAnchorComponentV2; 2],
    target_anchor: [SpatialAnchorComponentV2; 2],
) -> SpatialPlacementRecipeV2 {
    SpatialPlacementRecipeV2::Free(SpatialFreePlacementRecipeV2::new(
        spans.binding_literal(5),
        spans.binding_literal(4),
        self_anchor,
        target,
        target_anchor,
        point(spans, SCALE, SCALE),
        identity(spans),
    ))
}

fn dimension(spans: &mut SpanCursor, value: i32) -> SpatialDimensionRecipeV2 {
    SpatialDimensionRecipeV2::new(
        spans.binding_literal(value),
        spans.binding_literal(value),
        spans.binding_literal(value),
    )
}

fn container(spans: &mut SpanCursor) -> SpatialContainerRecipeV2 {
    SpatialContainerRecipeV2::new(
        SpatialAxisV2::Row,
        SpatialPaddingRecipeV2::new(
            spans.binding_literal(0),
            spans.binding_literal(0),
            spans.binding_literal(0),
            spans.binding_literal(0),
        ),
        spans.binding_literal(0),
    )
}

fn identity(spans: &mut SpanCursor) -> SpatialTransformRecipeV2 {
    transform(spans, [SCALE, 0, 0, SCALE, 0, 0])
}

fn quarter_turn(spans: &mut SpanCursor) -> SpatialTransformRecipeV2 {
    transform(spans, [0, SCALE, -SCALE, 0, SCALE, 0])
}

fn transform(spans: &mut SpanCursor, values: [i64; 6]) -> SpatialTransformRecipeV2 {
    SpatialTransformRecipeV2::new(
        spans.binding_literal(values[0]),
        spans.binding_literal(values[1]),
        spans.binding_literal(values[2]),
        spans.binding_literal(values[3]),
        spans.binding_literal(values[4]),
        spans.binding_literal(values[5]),
        point(spans, 0, 0),
    )
}

pub(super) fn point(spans: &mut SpanCursor, x: i64, y: i64) -> SpatialPointRecipeV2 {
    SpatialPointRecipeV2::new(spans.binding_literal(x), spans.binding_literal(y))
}

pub(super) struct SpanCursor {
    next: u32,
}

impl SpanCursor {
    const fn new(next: u32) -> Self {
        Self { next }
    }

    pub(super) fn take(&mut self) -> SourceSpan {
        let source = span(self.next);
        self.next += 1;
        source
    }

    pub(super) fn field<T>(&mut self, value: T) -> SpatialFieldV2<T> {
        SpatialFieldV2::new(value, self.take())
    }

    pub(super) fn binding_literal<T>(&mut self, value: T) -> SpatialFieldV2<SpatialBindingV2<T>> {
        self.field(SpatialBindingV2::Literal(value))
    }

    pub(super) fn binding_property<T>(
        &mut self,
        property: fenestra_ui_ir::prototype::PropertyId,
    ) -> SpatialFieldV2<SpatialBindingV2<T>> {
        self.field(SpatialBindingV2::Property(property))
    }
}
