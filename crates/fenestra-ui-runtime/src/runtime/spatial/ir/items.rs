use fenestra_ui_ir::prototype::{
    SourceSpan, SpatialClipAddressV2, SpatialCoverageRecipeV2, SpatialPaintRecipeV2,
};
use fenestra_ui_spatial::prototype::{
    SpatialCoverageV2, SpatialHitFieldV2 as HitField, SpatialHitV2, SpatialImageDestinationRectV2,
    SpatialImageSourceRectV2, SpatialNodeKeyV2, SpatialPaintContentV2,
    SpatialPaintFieldV2 as PaintField, SpatialPaintV2, SpatialSemanticFieldV2 as SemanticField,
    SpatialSemanticGeometryV2,
};

use super::super::error::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};
use super::super::view::RuntimeSpatialBuildViewV2;
use super::bindings;
use super::geometry::{self, GeometryTables};
use super::model::{ExpandedSpatialNode, LiveProgram};
use super::provenance::FieldSpans;
use super::resources::ResourceTables;

pub(super) struct ItemTables {
    pub(super) paints: Vec<SpatialPaintV2>,
    pub(super) hits: Vec<SpatialHitV2>,
    pub(super) semantics: Vec<SpatialSemanticGeometryV2>,
    pub(super) paint_provenance: Vec<FieldSpans<PaintField>>,
    pub(super) hit_provenance: Vec<FieldSpans<HitField>>,
    pub(super) semantic_provenance: Vec<FieldSpans<SemanticField>>,
}

pub(super) fn materialize(
    live: &LiveProgram<'_>,
    view: RuntimeSpatialBuildViewV2<'_>,
    geometry: &GeometryTables,
    resources: &ResourceTables,
) -> Result<ItemTables, RuntimeSpatialIrErrorV2> {
    let mut tables = ItemTables {
        paints: Vec::new(),
        hits: Vec::new(),
        semantics: Vec::new(),
        paint_provenance: Vec::new(),
        hit_provenance: Vec::new(),
        semantic_provenance: Vec::new(),
    };
    for expanded in live.expanded() {
        materialize_paints(&mut tables, live, expanded, view, geometry, resources)?;
        materialize_hits(&mut tables, live, expanded, view, geometry)?;
        materialize_semantics(&mut tables, live, expanded, geometry)?;
    }
    Ok(tables)
}

fn materialize_paints(
    tables: &mut ItemTables,
    live: &LiveProgram<'_>,
    expanded: &ExpandedSpatialNode<'_>,
    view: RuntimeSpatialBuildViewV2<'_>,
    geometry: &GeometryTables,
    resources: &ResourceTables,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    for (ordinal, paint) in expanded.declaration().paint_items().iter().enumerate() {
        let record = paint.span();
        let item_ordinal = key(ordinal);
        let owner = SpatialNodeKeyV2::new(expanded.key());
        let mut fields = vec![
            (PaintField::Owner, record),
            (PaintField::ItemOrdinal, record),
            (PaintField::Kind, record),
        ];
        let content = match *paint {
            SpatialPaintRecipeV2::CoveragePaint {
                coverage,
                brush,
                opacity,
                clip,
                ..
            } => {
                let coverage =
                    paint_coverage(coverage, expanded, view, geometry, record, &mut fields)?;
                let brush_key = resources
                    .brush_key(expanded.key(), *brush.value())
                    .ok_or_else(|| invariant(brush.span()))?;
                fields.extend([
                    (PaintField::Brush, brush.span()),
                    (PaintField::Opacity, opacity.span()),
                ]);
                let clip_key = clip_key(live, expanded, geometry, clip, |span| {
                    fields.push((PaintField::Clip, span));
                })?;
                SpatialPaintContentV2::CoveragePaint {
                    coverage,
                    brush: brush_key,
                    opacity: *opacity.value(),
                    clip: clip_key,
                }
            }
            SpatialPaintRecipeV2::ImagePaint {
                image,
                source_x,
                source_y,
                source_width,
                source_height,
                destination_origin,
                destination_width,
                destination_height,
                opacity,
                clip,
                ..
            } => {
                let image_key = resources
                    .image_key(*image.value())
                    .ok_or_else(|| invariant(image.span()))?;
                fields.extend([
                    (PaintField::Image, image.span()),
                    (PaintField::SourceX, source_x.span()),
                    (PaintField::SourceY, source_y.span()),
                    (PaintField::SourceWidth, source_width.span()),
                    (PaintField::SourceHeight, source_height.span()),
                    (PaintField::DestinationX, destination_origin.x().span()),
                    (PaintField::DestinationY, destination_origin.y().span()),
                    (PaintField::DestinationWidth, destination_width.span()),
                    (PaintField::DestinationHeight, destination_height.span()),
                    (PaintField::Opacity, opacity.span()),
                ]);
                let clip_key = clip_key(live, expanded, geometry, clip, |span| {
                    fields.push((PaintField::Clip, span));
                })?;
                SpatialPaintContentV2::ImagePaint {
                    image: image_key,
                    source: SpatialImageSourceRectV2::new(
                        *source_x.value(),
                        *source_y.value(),
                        *source_width.value(),
                        *source_height.value(),
                    ),
                    destination: SpatialImageDestinationRectV2::new(
                        bindings::scalar(destination_origin.x(), expanded.logical(), view)?,
                        bindings::scalar(destination_origin.y(), expanded.logical(), view)?,
                        bindings::scalar(destination_width, expanded.logical(), view)?,
                        bindings::scalar(destination_height, expanded.logical(), view)?,
                    ),
                    opacity: *opacity.value(),
                    clip: clip_key,
                }
            }
        };
        tables
            .paints
            .push(SpatialPaintV2::new(owner, item_ordinal, content));
        tables
            .paint_provenance
            .push(FieldSpans::new(record, fields));
    }
    Ok(())
}

fn materialize_hits(
    tables: &mut ItemTables,
    live: &LiveProgram<'_>,
    expanded: &ExpandedSpatialNode<'_>,
    view: RuntimeSpatialBuildViewV2<'_>,
    geometry: &GeometryTables,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    for (ordinal, hit) in expanded
        .declaration()
        .hit_items()
        .iter()
        .copied()
        .enumerate()
    {
        let record = hit.span();
        let mut fields = vec![
            (HitField::Owner, record),
            (HitField::ItemOrdinal, record),
            (HitField::CoverageKind, record),
        ];
        let coverage = hit_coverage(
            hit.coverage(),
            expanded,
            view,
            geometry,
            record,
            &mut fields,
        )?;
        let clip = clip_key(live, expanded, geometry, hit.clip(), |span| {
            fields.push((HitField::Clip, span));
        })?;
        fields.push((HitField::InputPolicy, hit.input_policy().span()));
        tables.hits.push(SpatialHitV2::new(
            SpatialNodeKeyV2::new(expanded.key()),
            key(ordinal),
            coverage,
            clip,
            bindings::input_policy(hit.input_policy(), expanded.logical(), view)?,
        ));
        tables.hit_provenance.push(FieldSpans::new(record, fields));
    }
    Ok(())
}

fn materialize_semantics(
    tables: &mut ItemTables,
    live: &LiveProgram<'_>,
    expanded: &ExpandedSpatialNode<'_>,
    geometry: &GeometryTables,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    for (ordinal, semantic) in expanded
        .declaration()
        .semantic_items()
        .iter()
        .copied()
        .enumerate()
    {
        let record = semantic.span();
        let shape = geometry
            .shape_key(expanded.key(), *semantic.shape().value())
            .ok_or_else(|| invariant(semantic.shape().span()))?;
        let mut fields = vec![
            (SemanticField::Owner, record),
            (SemanticField::ItemOrdinal, record),
            (SemanticField::Shape, semantic.shape().span()),
            (SemanticField::FillRule, record),
        ];
        let clip = clip_key(live, expanded, geometry, semantic.clip(), |span| {
            fields.push((SemanticField::Clip, span));
        })?;
        tables.semantics.push(SpatialSemanticGeometryV2::new(
            SpatialNodeKeyV2::new(expanded.key()),
            key(ordinal),
            shape,
            geometry::fill_rule(semantic.fill_rule()),
            clip,
        ));
        tables
            .semantic_provenance
            .push(FieldSpans::new(record, fields));
    }
    Ok(())
}

fn paint_coverage(
    recipe: SpatialCoverageRecipeV2,
    expanded: &ExpandedSpatialNode<'_>,
    view: RuntimeSpatialBuildViewV2<'_>,
    geometry: &GeometryTables,
    record: SourceSpan,
    fields: &mut Vec<(PaintField, SourceSpan)>,
) -> Result<SpatialCoverageV2, RuntimeSpatialIrErrorV2> {
    fields.push((PaintField::CoverageKind, record));
    Ok(match recipe {
        SpatialCoverageRecipeV2::Fill { shape, rule } => {
            fields.extend([
                (PaintField::Shape, shape.span()),
                (PaintField::FillRule, record),
            ]);
            SpatialCoverageV2::Fill {
                shape: geometry
                    .shape_key(expanded.key(), *shape.value())
                    .ok_or_else(|| invariant(shape.span()))?,
                rule: geometry::fill_rule(rule),
            }
        }
        SpatialCoverageRecipeV2::RoundStroke { shape, width } => {
            fields.extend([
                (PaintField::Shape, shape.span()),
                (PaintField::StrokeWidth, width.span()),
            ]);
            SpatialCoverageV2::RoundStroke {
                shape: geometry
                    .shape_key(expanded.key(), *shape.value())
                    .ok_or_else(|| invariant(shape.span()))?,
                width: bindings::scalar(width, expanded.logical(), view)?,
            }
        }
    })
}

fn hit_coverage(
    recipe: SpatialCoverageRecipeV2,
    expanded: &ExpandedSpatialNode<'_>,
    view: RuntimeSpatialBuildViewV2<'_>,
    geometry: &GeometryTables,
    record: SourceSpan,
    fields: &mut Vec<(HitField, SourceSpan)>,
) -> Result<SpatialCoverageV2, RuntimeSpatialIrErrorV2> {
    Ok(match recipe {
        SpatialCoverageRecipeV2::Fill { shape, rule } => {
            fields.extend([
                (HitField::Shape, shape.span()),
                (HitField::FillRule, record),
            ]);
            SpatialCoverageV2::Fill {
                shape: geometry
                    .shape_key(expanded.key(), *shape.value())
                    .ok_or_else(|| invariant(shape.span()))?,
                rule: geometry::fill_rule(rule),
            }
        }
        SpatialCoverageRecipeV2::RoundStroke { shape, width } => {
            fields.extend([
                (HitField::Shape, shape.span()),
                (HitField::StrokeWidth, width.span()),
            ]);
            SpatialCoverageV2::RoundStroke {
                shape: geometry
                    .shape_key(expanded.key(), *shape.value())
                    .ok_or_else(|| invariant(shape.span()))?,
                width: bindings::scalar(width, expanded.logical(), view)?,
            }
        }
    })
}

fn clip_key<F>(
    live: &LiveProgram<'_>,
    expanded: &ExpandedSpatialNode<'_>,
    geometry: &GeometryTables,
    address: Option<SpatialClipAddressV2>,
    mut capture: F,
) -> Result<Option<fenestra_ui_spatial::prototype::SpatialClipKeyV2>, RuntimeSpatialIrErrorV2>
where
    F: FnMut(SourceSpan),
{
    let Some(address) = address else {
        return Ok(None);
    };
    capture(address.clip().span());
    geometry
        .clip_key(live, expanded, address)
        .map(Some)
        .ok_or_else(|| invariant(address.clip().span()))
}

fn key(value: usize) -> u32 {
    u32::try_from(value).expect("representation preflight guards owner-local item ordinals")
}

fn invariant(span: SourceSpan) -> RuntimeSpatialIrErrorV2 {
    RuntimeSpatialIrErrorV2::new(RuntimeSpatialIrErrorKindV2::InvariantViolation, span)
}
