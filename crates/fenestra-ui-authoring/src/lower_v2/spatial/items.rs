use fenestra_ui_ir::prototype::{
    SpatialClipAddressV2, SpatialClipDeclarationV2, SpatialClipSymbolV2, SpatialCoverageRecipeV2,
    SpatialFieldV2, SpatialHitRecipeV2, SpatialPaintRecipeV2, SpatialSemanticRecipeV2, ValueType,
};

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::AuthoringLimitKindV2;
use crate::parsed_v2::{
    ParsedClipAddressV2, ParsedClipV2, ParsedCoverageV2, ParsedHitV2, ParsedNameFieldV2,
    ParsedPaintKindV2, ParsedPaintV2, ParsedSemanticV2,
};
use crate::resolved::logical_span;

use super::SpatialLowerer;

impl SpatialLowerer<'_> {
    pub(super) fn lower_clips(
        &self,
        owner: usize,
    ) -> Result<Vec<SpatialClipDeclarationV2>, AuthoringDiagnosticV2> {
        self.nodes[owner]
            .node
            .clips
            .iter()
            .enumerate()
            .map(|(index, clip)| self.lower_clip(owner, index, clip))
            .collect()
    }

    fn lower_clip(
        &self,
        owner: usize,
        index: usize,
        clip: &ParsedClipV2,
    ) -> Result<SpatialClipDeclarationV2, AuthoringDiagnosticV2> {
        if self.owner_symbols[owner].clips.is_duplicate(index) {
            return Err(self.error(
                AuthoringDiagnosticKindV2::DuplicateSpatialClipName,
                clip.symbol.anchor,
            ));
        }
        Ok(SpatialClipDeclarationV2::new(
            self.clip_symbol_field(&clip.symbol, index)?,
            clip.parent
                .as_ref()
                .map(|address| self.resolve_clip(address))
                .transpose()?,
            self.resolve_shape(owner, &clip.shape)?,
            clip.fill_rule,
            logical_span(clip.anchor),
        ))
    }

    pub(super) fn lower_paint_items(
        &self,
        owner: usize,
        component: u32,
    ) -> Result<Vec<SpatialPaintRecipeV2>, AuthoringDiagnosticV2> {
        self.nodes[owner]
            .node
            .paint
            .iter()
            .map(|paint| self.lower_paint(owner, component, paint))
            .collect()
    }

    fn lower_paint(
        &self,
        owner: usize,
        component: u32,
        paint: &ParsedPaintV2,
    ) -> Result<SpatialPaintRecipeV2, AuthoringDiagnosticV2> {
        let span = logical_span(paint.anchor);
        Ok(match &paint.kind {
            ParsedPaintKindV2::Coverage {
                coverage,
                brush,
                opacity,
                clip,
            } => SpatialPaintRecipeV2::CoveragePaint {
                coverage: self.lower_coverage(owner, component, coverage)?,
                brush: self.resolve_brush(owner, brush)?,
                opacity: self.literal_field(opacity)?,
                clip: clip
                    .as_ref()
                    .map(|address| self.resolve_clip(address))
                    .transpose()?,
                span,
            },
            ParsedPaintKindV2::Image {
                image,
                source,
                destination,
                destination_width,
                destination_height,
                opacity,
                clip,
            } => SpatialPaintRecipeV2::ImagePaint {
                image: self.resolve_image(image)?,
                source_x: self.literal_field(&source[0])?,
                source_y: self.literal_field(&source[1])?,
                source_width: self.literal_field(&source[2])?,
                source_height: self.literal_field(&source[3])?,
                destination_origin: self.lower_point(destination, component)?,
                destination_width: self.binding_field(
                    destination_width,
                    component,
                    ValueType::ScalarI32,
                )?,
                destination_height: self.binding_field(
                    destination_height,
                    component,
                    ValueType::ScalarI32,
                )?,
                opacity: self.literal_field(opacity)?,
                clip: clip
                    .as_ref()
                    .map(|address| self.resolve_clip(address))
                    .transpose()?,
                span,
            },
        })
    }

    pub(super) fn lower_hit_items(
        &self,
        owner: usize,
        component: u32,
    ) -> Result<Vec<SpatialHitRecipeV2>, AuthoringDiagnosticV2> {
        self.nodes[owner]
            .node
            .hit
            .iter()
            .map(|hit| self.lower_hit(owner, component, hit))
            .collect()
    }

    fn lower_hit(
        &self,
        owner: usize,
        component: u32,
        hit: &ParsedHitV2,
    ) -> Result<SpatialHitRecipeV2, AuthoringDiagnosticV2> {
        Ok(SpatialHitRecipeV2::new(
            self.lower_coverage(owner, component, &hit.coverage)?,
            hit.clip
                .as_ref()
                .map(|address| self.resolve_clip(address))
                .transpose()?,
            self.binding_field(&hit.input, component, ValueType::InputPolicy)?,
            logical_span(hit.anchor),
        ))
    }

    pub(super) fn lower_semantic_items(
        &self,
        owner: usize,
    ) -> Result<Vec<SpatialSemanticRecipeV2>, AuthoringDiagnosticV2> {
        self.nodes[owner]
            .node
            .semantics
            .iter()
            .map(|semantic| self.lower_semantic(owner, semantic))
            .collect()
    }

    fn lower_semantic(
        &self,
        owner: usize,
        semantic: &ParsedSemanticV2,
    ) -> Result<SpatialSemanticRecipeV2, AuthoringDiagnosticV2> {
        Ok(SpatialSemanticRecipeV2::new(
            self.resolve_shape(owner, &semantic.shape)?,
            semantic.fill_rule,
            semantic
                .clip
                .as_ref()
                .map(|address| self.resolve_clip(address))
                .transpose()?,
            logical_span(semantic.anchor),
        ))
    }

    fn lower_coverage(
        &self,
        owner: usize,
        component: u32,
        coverage: &ParsedCoverageV2,
    ) -> Result<SpatialCoverageRecipeV2, AuthoringDiagnosticV2> {
        Ok(match coverage {
            ParsedCoverageV2::Fill { shape, rule } => SpatialCoverageRecipeV2::Fill {
                shape: self.resolve_shape(owner, shape)?,
                rule: *rule,
            },
            ParsedCoverageV2::RoundStroke { shape, width } => {
                SpatialCoverageRecipeV2::RoundStroke {
                    shape: self.resolve_shape(owner, shape)?,
                    width: self.binding_field(width, component, ValueType::ScalarI32)?,
                }
            }
        })
    }

    fn resolve_clip(
        &self,
        address: &ParsedClipAddressV2,
    ) -> Result<SpatialClipAddressV2, AuthoringDiagnosticV2> {
        let owner = self.node_symbols.get(&address.owner.value).ok_or_else(|| {
            self.error(
                AuthoringDiagnosticKindV2::UnknownSpatialNodeName,
                address.owner.anchor,
            )
        })?;
        let owner_field = self.node_symbol_field(&address.owner, owner)?;
        let clip = self.owner_symbols[owner]
            .clips
            .get(&address.clip.value)
            .ok_or_else(|| {
                self.error(
                    AuthoringDiagnosticKindV2::UnknownSpatialClipName,
                    address.clip.anchor,
                )
            })?;
        Ok(SpatialClipAddressV2::new(
            owner_field,
            self.clip_symbol_field(&address.clip, clip)?,
        ))
    }

    fn clip_symbol_field(
        &self,
        field: &ParsedNameFieldV2,
        index: usize,
    ) -> Result<SpatialFieldV2<SpatialClipSymbolV2>, AuthoringDiagnosticV2> {
        let value = self.dense_symbol(index, AuthoringLimitKindV2::Clips, field.anchor)?;
        Ok(self.field_value(field.anchor, SpatialClipSymbolV2::new(value)))
    }
}
