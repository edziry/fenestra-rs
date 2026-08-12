use crate::diagnostic_v2::AuthoringDiagnosticV2;
use crate::parsed_v2::{
    ParsedBrushContentV2, ParsedBrushV2, ParsedClipAddressV2, ParsedClipV2, ParsedGradientStopV2,
    ParsedHitV2, ParsedPaintKindV2, ParsedPaintV2, ParsedSemanticV2,
};
use crate::token::Punctuation;
use crate::vocabulary_v2::AnchorKindV2;

use super::super::{ParserV2, RecordCountV2};

impl ParserV2 {
    pub(super) fn parse_brush(&mut self) -> Result<ParsedBrushV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("brush")?;
        self.claim_record(RecordCountV2::Brushes, opening.physical)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::SpatialBrush, &name)?;
        let symbol = self.push_field_spelled(name.text.clone(), &name)?;
        let content = if self.matches_keyword("solid") {
            self.expect_keyword("solid")?;
            self.expect_punctuation(Punctuation::OpenBrace)?;
            self.expect_keyword("color")?;
            let color = self.parse_color_binding_field()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_punctuation(Punctuation::CloseBrace)?;
            ParsedBrushContentV2::Solid(color)
        } else if self.matches_keyword("linear_gradient") {
            self.expect_keyword("linear_gradient")?;
            self.expect_punctuation(Punctuation::OpenBrace)?;
            self.expect_keyword("start")?;
            let start = self.parse_point()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("end")?;
            let end = self.parse_point()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            let mut stops = Vec::new();
            while self.matches_keyword("stop") {
                stops.push(self.parse_gradient_stop()?);
            }
            self.expect_punctuation(Punctuation::CloseBrace)?;
            ParsedBrushContentV2::LinearGradient { start, end, stops }
        } else {
            return Err(self.unexpected());
        };
        Ok(ParsedBrushV2 {
            name: name.text,
            symbol,
            content,
            anchor,
        })
    }

    fn parse_gradient_stop(&mut self) -> Result<ParsedGradientStopV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("stop")?;
        self.claim_record(RecordCountV2::GradientStops, opening.physical)?;
        let anchor = self.push_anchor(AnchorKindV2::SpatialGradientStop, &opening)?;
        let offset = self.parse_u16_field()?;
        let color = self.parse_color_binding_field()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        Ok(ParsedGradientStopV2 {
            offset,
            color,
            anchor,
        })
    }

    pub(super) fn parse_clip(&mut self) -> Result<ParsedClipV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("clip")?;
        self.claim_record(RecordCountV2::Clips, opening.physical)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::SpatialClip, &name)?;
        let symbol = self.push_field_spelled(name.text.clone(), &name)?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        self.expect_keyword("parent")?;
        let parent = self.parse_optional_clip()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_keyword("shape")?;
        let shape = self.parse_name_field()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_keyword("fill_rule")?;
        let fill_rule = self.parse_fill_rule()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedClipV2 {
            name: name.text,
            symbol,
            parent,
            shape,
            fill_rule,
            anchor,
        })
    }

    fn parse_optional_clip(
        &mut self,
    ) -> Result<Option<ParsedClipAddressV2>, AuthoringDiagnosticV2> {
        if self.matches_keyword("none") {
            self.expect_keyword("none")?;
            return Ok(None);
        }
        let owner = self.parse_name_field()?;
        self.expect_punctuation(Punctuation::Dot)?;
        let clip = self.parse_name_field()?;
        Ok(Some(ParsedClipAddressV2 { owner, clip }))
    }

    pub(super) fn parse_paint(&mut self) -> Result<ParsedPaintV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("paint")?;
        self.claim_record(RecordCountV2::PaintItems, opening.physical)?;
        let anchor = self.push_anchor(AnchorKindV2::SpatialPaint, &opening)?;
        let kind = if self.matches_keyword("coverage") {
            self.expect_keyword("coverage")?;
            self.expect_punctuation(Punctuation::OpenBrace)?;
            let coverage = self.parse_coverage()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("brush")?;
            let brush = self.parse_name_field()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("opacity")?;
            let opacity = self.parse_u8_field()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("clip")?;
            let clip = self.parse_optional_clip()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_punctuation(Punctuation::CloseBrace)?;
            ParsedPaintKindV2::Coverage {
                coverage,
                brush,
                opacity,
                clip,
            }
        } else if self.matches_keyword("image") {
            self.expect_keyword("image")?;
            self.expect_punctuation(Punctuation::OpenBrace)?;
            self.expect_keyword("image")?;
            let image = self.parse_name_field()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("source")?;
            self.expect_punctuation(Punctuation::OpenParenthesis)?;
            let source_x = self.parse_u32_field()?;
            self.expect_punctuation(Punctuation::Comma)?;
            let source_y = self.parse_u32_field()?;
            self.expect_punctuation(Punctuation::Comma)?;
            let source_width = self.parse_u32_field()?;
            self.expect_punctuation(Punctuation::Comma)?;
            let source_height = self.parse_u32_field()?;
            self.expect_punctuation(Punctuation::CloseParenthesis)?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("destination")?;
            let destination = self.parse_point()?;
            let destination_width = self.parse_fixed_binding_field()?;
            let destination_height = self.parse_fixed_binding_field()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("opacity")?;
            let opacity = self.parse_u8_field()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("clip")?;
            let clip = self.parse_optional_clip()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_punctuation(Punctuation::CloseBrace)?;
            ParsedPaintKindV2::Image {
                image,
                source: Box::new([source_x, source_y, source_width, source_height]),
                destination,
                destination_width,
                destination_height,
                opacity,
                clip,
            }
        } else {
            return Err(self.unexpected());
        };
        Ok(ParsedPaintV2 { kind, anchor })
    }

    pub(super) fn parse_hit(&mut self) -> Result<ParsedHitV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("hit")?;
        self.claim_record(RecordCountV2::HitItems, opening.physical)?;
        let anchor = self.push_anchor(AnchorKindV2::SpatialHit, &opening)?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        let coverage = self.parse_coverage()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_keyword("clip")?;
        let clip = self.parse_optional_clip()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_keyword("input")?;
        let input = self.parse_input_binding_field()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedHitV2 {
            coverage,
            clip,
            input,
            anchor,
        })
    }

    pub(super) fn parse_semantic(&mut self) -> Result<ParsedSemanticV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("semantic")?;
        self.claim_record(RecordCountV2::SemanticItems, opening.physical)?;
        let anchor = self.push_anchor(AnchorKindV2::SpatialSemantic, &opening)?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        self.expect_keyword("shape")?;
        let shape = self.parse_name_field()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_keyword("fill_rule")?;
        let fill_rule = self.parse_fill_rule()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_keyword("clip")?;
        let clip = self.parse_optional_clip()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedSemanticV2 {
            shape,
            fill_rule,
            clip,
            anchor,
        })
    }
}
