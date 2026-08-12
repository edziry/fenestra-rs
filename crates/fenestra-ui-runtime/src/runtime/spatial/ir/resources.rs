use fenestra_ui_ir::prototype::{
    SourceSpan, SpatialBrushContentV2 as IrBrushContent, SpatialBrushSymbolV2,
    SpatialImageSymbolV2, ValidatedSpatialProgramV2,
};
use fenestra_ui_spatial::prototype::{
    SpatialBrushContentV2, SpatialBrushFieldV2 as BrushField, SpatialBrushKeyV2, SpatialBrushV2,
    SpatialGradientStopFieldV2 as StopField, SpatialGradientStopV2,
    SpatialImageFieldV2 as ImageField, SpatialImageKeyV2, SpatialImageV2,
};

use super::super::error::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};
use super::super::view::RuntimeSpatialBuildViewV2;
use super::bindings;
use super::model::LiveProgram;
use super::provenance::FieldSpans;

pub(super) struct ResourceTables {
    pub(super) gradient_stops: Vec<SpatialGradientStopV2>,
    pub(super) brushes: Vec<SpatialBrushV2>,
    pub(super) images: Vec<SpatialImageV2>,
    pub(super) gradient_stop_provenance: Vec<FieldSpans<StopField>>,
    pub(super) brush_provenance: Vec<FieldSpans<BrushField>>,
    pub(super) image_provenance: Vec<FieldSpans<ImageField>>,
    owner_brushes: Vec<Vec<(SpatialBrushSymbolV2, SpatialBrushKeyV2)>>,
    image_keys: Vec<(SpatialImageSymbolV2, SpatialImageKeyV2)>,
}

impl ResourceTables {
    pub(super) fn brush_key(
        &self,
        owner_key: u32,
        symbol: SpatialBrushSymbolV2,
    ) -> Option<SpatialBrushKeyV2> {
        self.owner_brushes
            .get(usize::try_from(owner_key).ok()?)?
            .iter()
            .find_map(|(candidate, key)| (*candidate == symbol).then_some(*key))
    }

    pub(super) fn image_key(&self, symbol: SpatialImageSymbolV2) -> Option<SpatialImageKeyV2> {
        self.image_keys
            .iter()
            .find_map(|(candidate, key)| (*candidate == symbol).then_some(*key))
    }
}

pub(super) fn materialize(
    program: &ValidatedSpatialProgramV2,
    live: &LiveProgram<'_>,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<ResourceTables, RuntimeSpatialIrErrorV2> {
    let mut tables = ResourceTables {
        gradient_stops: Vec::new(),
        brushes: Vec::new(),
        images: Vec::new(),
        gradient_stop_provenance: Vec::new(),
        brush_provenance: Vec::new(),
        image_provenance: Vec::new(),
        owner_brushes: (0..=live.expanded().len()).map(|_| Vec::new()).collect(),
        image_keys: Vec::new(),
    };

    for expanded in live.expanded() {
        for brush in expanded.declaration().brushes() {
            let brush_key = key(tables.brushes.len());
            let content = match brush.content() {
                IrBrushContent::Solid { color } => SpatialBrushContentV2::Solid {
                    color: bindings::color(*color, expanded.logical(), view)?,
                },
                IrBrushContent::LinearGradient { start, end, stops } => {
                    let stop_start = key(tables.gradient_stops.len());
                    let stop_length = key(stops.len());
                    for stop in stops {
                        tables.gradient_stops.push(SpatialGradientStopV2::new(
                            *stop.offset().value(),
                            bindings::color(stop.color(), expanded.logical(), view)?,
                        ));
                        tables.gradient_stop_provenance.push(FieldSpans::new(
                            stop.span(),
                            vec![
                                (StopField::Offset, stop.offset().span()),
                                (StopField::R, stop.color().span()),
                                (StopField::G, stop.color().span()),
                                (StopField::B, stop.color().span()),
                                (StopField::A, stop.color().span()),
                            ],
                        ));
                    }
                    SpatialBrushContentV2::LinearGradient {
                        stop_start,
                        stop_length,
                        start: bindings::point(*start, expanded.logical(), view)?,
                        end: bindings::point(*end, expanded.logical(), view)?,
                    }
                }
            };
            tables.brushes.push(SpatialBrushV2::new(
                SpatialBrushKeyV2::new(brush_key),
                content,
            ));
            tables
                .owner_brushes
                .get_mut(
                    usize::try_from(expanded.key())
                        .expect("representation preflight guards node keys"),
                )
                .ok_or_else(|| invariant(brush.span()))?
                .push((*brush.symbol().value(), SpatialBrushKeyV2::new(brush_key)));
            tables.brush_provenance.push(brush_provenance(brush));
        }
    }

    for image in program.program().images() {
        let image_key = key(tables.images.len());
        tables.images.push(SpatialImageV2::new(
            SpatialImageKeyV2::new(image_key),
            *image.width().value(),
            *image.height().value(),
            *image.stride().value(),
            image.bytes().into(),
        ));
        tables
            .image_keys
            .push((*image.symbol().value(), SpatialImageKeyV2::new(image_key)));
        tables.image_provenance.push(FieldSpans::new(
            image.span(),
            vec![
                (ImageField::Key, image.span()),
                (ImageField::Width, image.width().span()),
                (ImageField::Height, image.height().span()),
                (ImageField::Stride, image.stride().span()),
                (ImageField::ByteLength, image.span()),
                (ImageField::Pixel, image.span()),
            ],
        ));
    }
    Ok(tables)
}

fn brush_provenance(
    brush: &fenestra_ui_ir::prototype::SpatialBrushDeclarationV2,
) -> FieldSpans<BrushField> {
    let record = brush.span();
    let mut fields = vec![(BrushField::Key, record), (BrushField::Kind, record)];
    match brush.content() {
        IrBrushContent::Solid { color } => fields.extend([
            (BrushField::ColorR, color.span()),
            (BrushField::ColorG, color.span()),
            (BrushField::ColorB, color.span()),
            (BrushField::ColorA, color.span()),
        ]),
        IrBrushContent::LinearGradient { start, end, .. } => fields.extend([
            (BrushField::GradientStopStart, record),
            (BrushField::GradientStopLength, record),
            (BrushField::GradientStartX, start.x().span()),
            (BrushField::GradientStartY, start.y().span()),
            (BrushField::GradientEndX, end.x().span()),
            (BrushField::GradientEndY, end.y().span()),
        ]),
    }
    FieldSpans::new(record, fields)
}

fn key(value: usize) -> u32 {
    u32::try_from(value).expect("representation preflight guards raw keys and ranges")
}

fn invariant(span: SourceSpan) -> RuntimeSpatialIrErrorV2 {
    RuntimeSpatialIrErrorV2::new(RuntimeSpatialIrErrorKindV2::InvariantViolation, span)
}
