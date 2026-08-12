use fenestra_ui_ir::prototype::SourceSpan;
use fenestra_ui_spatial::prototype::{
    SpatialBrushContentV2, SpatialBrushFieldV2, SpatialClipFieldV2, SpatialErrorLocationV2,
    SpatialGradientStopFieldV2, SpatialHitFieldV2, SpatialImageFieldV2, SpatialInputV2,
    SpatialNodeFieldV2, SpatialNodeV2, SpatialOutputTableV2, SpatialPaintFieldV2,
    SpatialPathFieldV2, SpatialPathVerbFieldV2, SpatialPlacementV2, SpatialPolygonPointFieldV2,
    SpatialSemanticFieldV2, SpatialShapeFieldV2, SpatialShapeGeometryV2,
};

pub(super) struct FieldSpans<F> {
    pub(super) record: SourceSpan,
    pub(super) fields: Vec<(F, SourceSpan)>,
}

impl<F> FieldSpans<F> {
    pub(super) fn new(record: SourceSpan, fields: Vec<(F, SourceSpan)>) -> Self {
        Self { record, fields }
    }
}

pub(super) struct Provenance {
    pub(super) program: SourceSpan,
    pub(super) nodes: Vec<FieldSpans<SpatialNodeFieldV2>>,
    pub(super) path_verbs: Vec<FieldSpans<SpatialPathVerbFieldV2>>,
    pub(super) paths: Vec<FieldSpans<SpatialPathFieldV2>>,
    pub(super) polygon_points: Vec<FieldSpans<SpatialPolygonPointFieldV2>>,
    pub(super) shapes: Vec<FieldSpans<SpatialShapeFieldV2>>,
    pub(super) gradient_stops: Vec<FieldSpans<SpatialGradientStopFieldV2>>,
    pub(super) brushes: Vec<FieldSpans<SpatialBrushFieldV2>>,
    pub(super) images: Vec<FieldSpans<SpatialImageFieldV2>>,
    pub(super) clips: Vec<FieldSpans<SpatialClipFieldV2>>,
    pub(super) paints: Vec<FieldSpans<SpatialPaintFieldV2>>,
    pub(super) hits: Vec<FieldSpans<SpatialHitFieldV2>>,
    pub(super) semantics: Vec<FieldSpans<SpatialSemanticFieldV2>>,
    pub(super) islands: Vec<SourceSpan>,
}

impl Provenance {
    pub(super) fn validate(&mut self, input: SpatialInputV2<'_>) -> bool {
        let topology = input.topology();
        let geometry = input.geometry();
        let resources = input.resources();
        let items = input.items();
        if self.nodes.len() != topology.nodes().len()
            || self.path_verbs.len() != geometry.path_verbs().len()
            || self.paths.len() != geometry.paths().len()
            || self.polygon_points.len() != geometry.polygon_points().len()
            || self.shapes.len() != geometry.shapes().len()
            || self.gradient_stops.len() != resources.gradient_stops().len()
            || self.brushes.len() != resources.brushes().len()
            || self.images.len() != resources.images().len()
            || self.clips.len() != geometry.clips().len()
            || self.paints.len() != items.paint_items().len()
            || self.hits.len() != items.hit_items().len()
            || self.semantics.len() != items.semantic_items().len()
        {
            return false;
        }
        let Some(islands) = island_spans(topology.nodes(), &self.nodes) else {
            return false;
        };
        self.islands = islands;
        true
    }

    pub(super) fn span_for(
        &self,
        location: SpatialErrorLocationV2,
        input: SpatialInputV2<'_>,
    ) -> Option<SourceSpan> {
        match location {
            SpatialErrorLocationV2::Input | SpatialErrorLocationV2::Output { .. } => {
                Some(self.program)
            }
            SpatialErrorLocationV2::Viewport { .. } => Some(SourceSpan::Synthetic),
            SpatialErrorLocationV2::Node { index }
            | SpatialErrorLocationV2::Dependency { ordinal: index } => record(&self.nodes, index),
            SpatialErrorLocationV2::NodeField { index, field } => {
                field_span(&self.nodes, index, field)
            }
            SpatialErrorLocationV2::Path { index, field } => field_span(&self.paths, index, field),
            SpatialErrorLocationV2::PathVerb { path, verb, field } => {
                let start = input.geometry().paths().get(index(path)?)?.verb_start();
                nested_field_span(&self.path_verbs, start, verb, field)
            }
            SpatialErrorLocationV2::Shape { index, field } => {
                field_span(&self.shapes, index, field)
            }
            SpatialErrorLocationV2::PolygonPoint {
                shape,
                point,
                field,
            } => {
                let SpatialShapeGeometryV2::Polygon { point_start, .. } =
                    input.geometry().shapes().get(index(shape)?)?.geometry()
                else {
                    return None;
                };
                nested_field_span(&self.polygon_points, point_start, point, field)
            }
            SpatialErrorLocationV2::Brush { index, field } => {
                field_span(&self.brushes, index, field)
            }
            SpatialErrorLocationV2::GradientStop { brush, stop, field } => {
                let SpatialBrushContentV2::LinearGradient { stop_start, .. } =
                    input.resources().brushes().get(index(brush)?)?.content()
                else {
                    return None;
                };
                nested_field_span(&self.gradient_stops, stop_start, stop, field)
            }
            SpatialErrorLocationV2::Image { index, field } => {
                field_span(&self.images, index, field)
            }
            SpatialErrorLocationV2::ImagePixel { image, .. } => record(&self.images, image),
            SpatialErrorLocationV2::Clip { index, field } => field_span(&self.clips, index, field),
            SpatialErrorLocationV2::Paint { index, field } => {
                field_span(&self.paints, index, field)
            }
            SpatialErrorLocationV2::Hit { index, field } => field_span(&self.hits, index, field),
            SpatialErrorLocationV2::Semantic { index, field } => {
                field_span(&self.semantics, index, field)
            }
            SpatialErrorLocationV2::Island { index } => {
                self.islands.get(usize::try_from(index).ok()?).copied()
            }
            SpatialErrorLocationV2::OutputRecord { table, index, .. } => {
                output_record_span(self, table, index)
            }
        }
    }
}

fn record<F>(records: &[FieldSpans<F>], index: u32) -> Option<SourceSpan> {
    records
        .get(usize::try_from(index).ok()?)
        .map(|row| row.record)
}

fn field_span<F: Copy + Eq>(records: &[FieldSpans<F>], index: u32, field: F) -> Option<SourceSpan> {
    records
        .get(usize::try_from(index).ok()?)?
        .fields
        .iter()
        .find_map(|(candidate, span)| (*candidate == field).then_some(*span))
}

fn nested_field_span<F: Copy + Eq>(
    records: &[FieldSpans<F>],
    start: u32,
    local: u32,
    field: F,
) -> Option<SourceSpan> {
    field_span(records, start.checked_add(local)?, field)
}

fn output_record_span(
    provenance: &Provenance,
    table: SpatialOutputTableV2,
    index: u32,
) -> Option<SourceSpan> {
    match table {
        SpatialOutputTableV2::Geometry => record(&provenance.nodes, index),
        SpatialOutputTableV2::Clip => record(&provenance.clips, index),
        SpatialOutputTableV2::Paint => record(&provenance.paints, index),
        SpatialOutputTableV2::Hit => record(&provenance.hits, index),
        SpatialOutputTableV2::Semantic => record(&provenance.semantics, index),
    }
}

fn island_spans(
    nodes: &[SpatialNodeV2],
    provenance: &[FieldSpans<SpatialNodeFieldV2>],
) -> Option<Vec<SourceSpan>> {
    let mut hosted = vec![None; nodes.len()];
    let mut membership = vec![None; nodes.len()];
    let mut spans = Vec::new();
    for (node_index, node) in nodes.iter().copied().enumerate().skip(1) {
        let SpatialPlacementV2::Layout(_) = node.placement() else {
            if matches!(node.placement(), SpatialPlacementV2::Root) {
                return None;
            }
            continue;
        };
        let parent = usize::try_from(node.parent()?.get()).ok()?;
        let parent_node = nodes.get(parent)?;
        let island = match parent_node.placement() {
            SpatialPlacementV2::Layout(_) => membership[parent]?,
            SpatialPlacementV2::Root | SpatialPlacementV2::Free(_) => match hosted[parent] {
                Some(island) => island,
                None => {
                    let island = spans.len();
                    spans.push(provenance.get(parent)?.record);
                    hosted[parent] = Some(island);
                    island
                }
            },
        };
        membership[node_index] = Some(island);
    }
    Some(spans)
}

fn index(value: u32) -> Option<usize> {
    usize::try_from(value).ok()
}
