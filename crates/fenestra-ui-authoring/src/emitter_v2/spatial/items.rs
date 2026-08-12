use fenestra_ui_ir::prototype::{
    SpatialClipDeclarationV2, SpatialCoverageRecipeV2, SpatialFillRuleV2, SpatialHitRecipeV2,
    SpatialPaintRecipeV2, SpatialSemanticRecipeV2,
};
use proc_macro2::TokenStream;

use crate::emitter::builder::{ir_call, ir_path, ir_record};

use super::layout::point;
use super::records;
use super::value::{
    field_brush, field_i64_binding, field_image, field_input_binding, field_shape, field_u8,
    field_u32, optional_clip, source_span,
};

pub(super) fn clips(values: &[SpatialClipDeclarationV2]) -> TokenStream {
    records(values.iter().copied().map(clip).collect())
}

pub(super) fn paint_items(values: &[SpatialPaintRecipeV2]) -> TokenStream {
    records(values.iter().copied().map(paint).collect())
}

pub(super) fn hit_items(values: &[SpatialHitRecipeV2]) -> TokenStream {
    records(values.iter().copied().map(hit).collect())
}

pub(super) fn semantic_items(values: &[SpatialSemanticRecipeV2]) -> TokenStream {
    records(values.iter().copied().map(semantic).collect())
}

fn clip(value: SpatialClipDeclarationV2) -> TokenStream {
    ir_call(
        &["SpatialClipDeclarationV2", "new"],
        vec![
            super::value::field_clip(value.symbol()),
            optional_clip(value.parent()),
            field_shape(value.shape()),
            fill_rule(value.fill_rule()),
            source_span(value.span()),
        ],
        true,
    )
}

fn paint(value: SpatialPaintRecipeV2) -> TokenStream {
    match value {
        SpatialPaintRecipeV2::CoveragePaint {
            coverage,
            brush,
            opacity,
            clip,
            span,
        } => ir_record(
            &["SpatialPaintRecipeV2", "CoveragePaint"],
            vec![
                ("coverage", coverage_recipe(coverage)),
                ("brush", field_brush(brush)),
                ("opacity", field_u8(opacity)),
                ("clip", optional_clip(clip)),
                ("span", source_span(span)),
            ],
        ),
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
            span,
        } => ir_record(
            &["SpatialPaintRecipeV2", "ImagePaint"],
            vec![
                ("image", field_image(image)),
                ("source_x", field_u32(source_x)),
                ("source_y", field_u32(source_y)),
                ("source_width", field_u32(source_width)),
                ("source_height", field_u32(source_height)),
                ("destination_origin", point(destination_origin)),
                ("destination_width", field_i64_binding(destination_width)),
                ("destination_height", field_i64_binding(destination_height)),
                ("opacity", field_u8(opacity)),
                ("clip", optional_clip(clip)),
                ("span", source_span(span)),
            ],
        ),
    }
}

fn hit(value: SpatialHitRecipeV2) -> TokenStream {
    ir_call(
        &["SpatialHitRecipeV2", "new"],
        vec![
            coverage_recipe(value.coverage()),
            optional_clip(value.clip()),
            field_input_binding(value.input_policy()),
            source_span(value.span()),
        ],
        true,
    )
}

fn semantic(value: SpatialSemanticRecipeV2) -> TokenStream {
    ir_call(
        &["SpatialSemanticRecipeV2", "new"],
        vec![
            field_shape(value.shape()),
            fill_rule(value.fill_rule()),
            optional_clip(value.clip()),
            source_span(value.span()),
        ],
        true,
    )
}

fn coverage_recipe(value: SpatialCoverageRecipeV2) -> TokenStream {
    match value {
        SpatialCoverageRecipeV2::Fill { shape, rule } => ir_record(
            &["SpatialCoverageRecipeV2", "Fill"],
            vec![("shape", field_shape(shape)), ("rule", fill_rule(rule))],
        ),
        SpatialCoverageRecipeV2::RoundStroke { shape, width } => ir_record(
            &["SpatialCoverageRecipeV2", "RoundStroke"],
            vec![
                ("shape", field_shape(shape)),
                ("width", field_i64_binding(width)),
            ],
        ),
    }
}

fn fill_rule(value: SpatialFillRuleV2) -> TokenStream {
    let variant = match value {
        SpatialFillRuleV2::NonZero => "NonZero",
        SpatialFillRuleV2::EvenOdd => "EvenOdd",
    };
    ir_path(&["SpatialFillRuleV2", variant])
}
