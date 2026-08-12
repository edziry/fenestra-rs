use fenestra_ui_ir::prototype::{
    SUPPORTED_SPATIAL_FORMAT, SchemaNamespace, SchemaRevision, SpatialAxisV2, SpatialBindingV2,
    SpatialBrushContentV2, SpatialBrushDeclarationV2, SpatialBrushSymbolV2, SpatialClipAddressV2,
    SpatialClipDeclarationV2, SpatialClipSymbolV2, SpatialContainerRecipeV2,
    SpatialCoverageRecipeV2, SpatialDimensionRecipeV2, SpatialFieldV2, SpatialFillRuleV2,
    SpatialHitRecipeV2, SpatialImageDeclarationV2, SpatialImageSymbolV2,
    SpatialLayoutPlacementRecipeV2, SpatialNodeDeclarationV2, SpatialNodeParentV2,
    SpatialNodeSymbolV2, SpatialPaddingRecipeV2, SpatialPaintRecipeV2, SpatialPlacementRecipeV2,
    SpatialPointRecipeV2, SpatialProgramV2, SpatialSemanticRecipeV2, SpatialShapeDeclarationV2,
    SpatialShapeGeometryV2, SpatialShapeSymbolV2, SpatialTransformRecipeV2,
    SpatialViewportContainerV2,
};

use super::{COLOR, IMAGE_COLOR, INNER, OUTER, POLICY, WIDTH, span};

const NAMESPACE: SchemaNamespace = SchemaNamespace::new(9001);
const REVISION: SchemaRevision = SchemaRevision::new(2);
const OUTER_SYMBOL: SpatialNodeSymbolV2 = SpatialNodeSymbolV2::new(7);
const INNER_SYMBOL: SpatialNodeSymbolV2 = SpatialNodeSymbolV2::new(900);
const SHAPE: SpatialShapeSymbolV2 = SpatialShapeSymbolV2::new(71);
const BRUSH: SpatialBrushSymbolV2 = SpatialBrushSymbolV2::new(81);
const CLIP: SpatialClipSymbolV2 = SpatialClipSymbolV2::new(91);
const IMAGE: SpatialImageSymbolV2 = SpatialImageSymbolV2::new(101);
const SCALE: i64 = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialSpans {
    pub program: fenestra_ui_ir::prototype::SourceSpan,
    pub outer_node: fenestra_ui_ir::prototype::SourceSpan,
    pub inner_node: fenestra_ui_ir::prototype::SourceSpan,
    pub outer_width_minimum: fenestra_ui_ir::prototype::SourceSpan,
    pub color_binding: fenestra_ui_ir::prototype::SourceSpan,
    pub policy_binding: fenestra_ui_ir::prototype::SourceSpan,
    pub image: fenestra_ui_ir::prototype::SourceSpan,
}

pub(super) fn program() -> (SpatialProgramV2, SpatialSpans) {
    let mut spans = SpanCursor::new(100);
    let viewport = viewport(&mut spans);
    let (outer, outer_spans) = outer_node(&mut spans);
    let (inner, inner_node_span) = inner_node(&mut spans);
    let (image, image_span) = image(&mut spans);
    let program_span = spans.take();
    let raw = SpatialProgramV2::new(
        SUPPORTED_SPATIAL_FORMAT,
        NAMESPACE,
        REVISION,
        viewport,
        vec![outer, inner],
        vec![image],
        program_span,
    );
    (
        raw,
        SpatialSpans {
            program: program_span,
            outer_node: outer_spans.node,
            inner_node: inner_node_span,
            outer_width_minimum: outer_spans.width_minimum,
            color_binding: outer_spans.color_binding,
            policy_binding: outer_spans.policy_binding,
            image: image_span,
        },
    )
}

fn viewport(spans: &mut SpanCursor) -> SpatialViewportContainerV2 {
    let left = spans.field(0);
    let right = spans.field(0);
    let top = spans.field(0);
    let bottom = spans.field(0);
    let gap = spans.field(0);
    SpatialViewportContainerV2::new(
        SpatialAxisV2::Column,
        left,
        right,
        top,
        bottom,
        gap,
        spans.take(),
    )
}

struct OuterSpans {
    node: fenestra_ui_ir::prototype::SourceSpan,
    width_minimum: fenestra_ui_ir::prototype::SourceSpan,
    color_binding: fenestra_ui_ir::prototype::SourceSpan,
    policy_binding: fenestra_ui_ir::prototype::SourceSpan,
}

fn outer_node(spans: &mut SpanCursor) -> (SpatialNodeDeclarationV2, OuterSpans) {
    let symbol = spans.field(OUTER_SYMBOL);
    let template = spans.field(OUTER);
    let width_minimum = spans.take();
    let width = SpatialDimensionRecipeV2::new(
        SpatialFieldV2::new(SpatialBindingV2::Property(WIDTH), width_minimum),
        spans.binding_property(WIDTH),
        spans.binding_property(WIDTH),
    );
    let height = dimension_literal(spans, 6);
    let placement = SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
        width,
        height,
        transform(spans),
    ));
    let container = container(spans);
    let shape = shape(spans);
    let (brush, color_binding) = brush(spans);
    let clip = clip(spans);
    let paints = vec![coverage_paint(spans), image_paint(spans)];
    let (hit, policy_binding) = hit(spans);
    let semantic = semantic(spans);
    let node_span = spans.take();
    (
        SpatialNodeDeclarationV2::new(
            symbol,
            template,
            SpatialNodeParentV2::Viewport,
            placement,
            container,
            vec![shape],
            vec![brush],
            vec![clip],
            paints,
            vec![hit],
            vec![semantic],
            node_span,
        ),
        OuterSpans {
            node: node_span,
            width_minimum,
            color_binding,
            policy_binding,
        },
    )
}

fn inner_node(
    spans: &mut SpanCursor,
) -> (
    SpatialNodeDeclarationV2,
    fenestra_ui_ir::prototype::SourceSpan,
) {
    let symbol = spans.field(INNER_SYMBOL);
    let template = spans.field(INNER);
    let parent = SpatialNodeParentV2::Node(spans.field(OUTER_SYMBOL));
    let placement = SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
        dimension_literal(spans, 4),
        dimension_literal(spans, 3),
        transform(spans),
    ));
    let node_span = spans.take();
    (
        SpatialNodeDeclarationV2::new(
            symbol,
            template,
            parent,
            placement,
            container(spans),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            node_span,
        ),
        node_span,
    )
}

fn shape(spans: &mut SpanCursor) -> SpatialShapeDeclarationV2 {
    SpatialShapeDeclarationV2::new(
        spans.field(SHAPE),
        SpatialShapeGeometryV2::Rect {
            origin: point_literal(spans, 0, 0),
            width: spans.binding_property(WIDTH),
            height: spans.binding_literal(6 * SCALE),
        },
        spans.take(),
    )
}

fn brush(
    spans: &mut SpanCursor,
) -> (
    SpatialBrushDeclarationV2,
    fenestra_ui_ir::prototype::SourceSpan,
) {
    let source = spans.take();
    let color = SpatialFieldV2::new(SpatialBindingV2::Property(COLOR), source);
    (
        SpatialBrushDeclarationV2::new(
            spans.field(BRUSH),
            SpatialBrushContentV2::Solid { color },
            spans.take(),
        ),
        source,
    )
}

fn clip(spans: &mut SpanCursor) -> SpatialClipDeclarationV2 {
    SpatialClipDeclarationV2::new(
        spans.field(CLIP),
        None,
        spans.field(SHAPE),
        SpatialFillRuleV2::NonZero,
        spans.take(),
    )
}

fn coverage_paint(spans: &mut SpanCursor) -> SpatialPaintRecipeV2 {
    SpatialPaintRecipeV2::CoveragePaint {
        coverage: coverage(spans),
        brush: spans.field(BRUSH),
        opacity: spans.field(255),
        clip: Some(clip_address(spans)),
        span: spans.take(),
    }
}

fn image_paint(spans: &mut SpanCursor) -> SpatialPaintRecipeV2 {
    SpatialPaintRecipeV2::ImagePaint {
        image: spans.field(IMAGE),
        source_x: spans.field(0),
        source_y: spans.field(0),
        source_width: spans.field(1),
        source_height: spans.field(1),
        destination_origin: point_literal(spans, 0, 0),
        destination_width: spans.binding_literal(SCALE),
        destination_height: spans.binding_literal(SCALE),
        opacity: spans.field(255),
        clip: None,
        span: spans.take(),
    }
}

fn hit(spans: &mut SpanCursor) -> (SpatialHitRecipeV2, fenestra_ui_ir::prototype::SourceSpan) {
    let source = spans.take();
    (
        SpatialHitRecipeV2::new(
            coverage(spans),
            Some(clip_address(spans)),
            SpatialFieldV2::new(SpatialBindingV2::Property(POLICY), source),
            spans.take(),
        ),
        source,
    )
}

fn semantic(spans: &mut SpanCursor) -> SpatialSemanticRecipeV2 {
    SpatialSemanticRecipeV2::new(
        spans.field(SHAPE),
        SpatialFillRuleV2::NonZero,
        Some(clip_address(spans)),
        spans.take(),
    )
}

fn coverage(spans: &mut SpanCursor) -> SpatialCoverageRecipeV2 {
    SpatialCoverageRecipeV2::Fill {
        shape: spans.field(SHAPE),
        rule: SpatialFillRuleV2::NonZero,
    }
}

fn clip_address(spans: &mut SpanCursor) -> SpatialClipAddressV2 {
    SpatialClipAddressV2::new(spans.field(OUTER_SYMBOL), spans.field(CLIP))
}

fn image(
    spans: &mut SpanCursor,
) -> (
    SpatialImageDeclarationV2,
    fenestra_ui_ir::prototype::SourceSpan,
) {
    let source = spans.take();
    (
        SpatialImageDeclarationV2::new(
            spans.field(IMAGE),
            spans.field(1),
            spans.field(1),
            spans.field(4),
            IMAGE_COLOR.into(),
            source,
        ),
        source,
    )
}

fn dimension_literal(spans: &mut SpanCursor, value: i32) -> SpatialDimensionRecipeV2 {
    SpatialDimensionRecipeV2::new(
        spans.binding_literal(value),
        spans.binding_literal(value),
        spans.binding_literal(value),
    )
}

fn transform(spans: &mut SpanCursor) -> SpatialTransformRecipeV2 {
    SpatialTransformRecipeV2::new(
        spans.binding_literal(SCALE),
        spans.binding_literal(0),
        spans.binding_literal(0),
        spans.binding_literal(SCALE),
        spans.binding_literal(0),
        spans.binding_literal(0),
        point_literal(spans, 0, 0),
    )
}

fn container(spans: &mut SpanCursor) -> SpatialContainerRecipeV2 {
    SpatialContainerRecipeV2::new(
        SpatialAxisV2::Column,
        SpatialPaddingRecipeV2::new(
            spans.binding_literal(0),
            spans.binding_literal(0),
            spans.binding_literal(0),
            spans.binding_literal(0),
        ),
        spans.binding_literal(0),
    )
}

fn point_literal(spans: &mut SpanCursor, x: i64, y: i64) -> SpatialPointRecipeV2 {
    SpatialPointRecipeV2::new(spans.binding_literal(x), spans.binding_literal(y))
}

struct SpanCursor {
    next: u32,
}

impl SpanCursor {
    const fn new(next: u32) -> Self {
        Self { next }
    }

    fn take(&mut self) -> fenestra_ui_ir::prototype::SourceSpan {
        let source = span(self.next);
        self.next += 1;
        source
    }

    fn field<T>(&mut self, value: T) -> SpatialFieldV2<T> {
        SpatialFieldV2::new(value, self.take())
    }

    fn binding_literal<T>(&mut self, value: T) -> SpatialFieldV2<SpatialBindingV2<T>> {
        self.field(SpatialBindingV2::Literal(value))
    }

    fn binding_property<T>(
        &mut self,
        property: fenestra_ui_ir::prototype::PropertyId,
    ) -> SpatialFieldV2<SpatialBindingV2<T>> {
        self.field(SpatialBindingV2::Property(property))
    }
}
