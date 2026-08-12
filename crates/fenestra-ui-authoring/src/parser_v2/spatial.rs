mod geometry;
mod items;
mod layout;
mod value;

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::parsed_v2::{ParsedImageV2, ParsedNodeV2, ParsedSpatialV2};
use crate::token::Punctuation;
use crate::version_v2::SUPPORTED_AUTHORING_FORMAT_V2;
use crate::vocabulary_v2::AnchorKindV2;

use super::{ParserV2, RecordCountV2};

impl ParserV2 {
    pub(super) fn parse_spatial(&mut self) -> Result<ParsedSpatialV2, AuthoringDiagnosticV2> {
        let keyword = self.expect_keyword("spatial")?;
        let anchor = self.push_anchor(AnchorKindV2::Spatial, &keyword)?;
        self.expect_keyword("format")?;
        let format = self.parse_u32()?;
        let format = format.value.map_err(|physical| {
            self.anchored_failure_at(AuthoringDiagnosticKindV2::InvalidLiteral, anchor, physical)
        })?;
        if format != SUPPORTED_AUTHORING_FORMAT_V2.get() {
            return Err(self.anchored_failure(
                AuthoringDiagnosticKindV2::UnsupportedAuthoringFormat,
                anchor,
            ));
        }
        self.expect_punctuation(Punctuation::OpenBrace)?;
        let viewport = self.parse_viewport()?;
        let (resources_anchor, images) = self.parse_resources()?;
        let mut nodes = Vec::new();
        while self.matches_keyword("node") {
            nodes.push(self.parse_node(None)?);
        }
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedSpatialV2 {
            format,
            viewport,
            resources_anchor,
            images,
            nodes,
            field_count: self.record_count(RecordCountV2::SpatialFields),
            anchor,
        })
    }

    fn parse_resources(&mut self) -> Result<(u32, Vec<ParsedImageV2>), AuthoringDiagnosticV2> {
        let keyword = self.expect_keyword("resources")?;
        let anchor = self.push_anchor(AnchorKindV2::Resources, &keyword)?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        let mut images = Vec::new();
        while self.matches_keyword("image") {
            images.push(self.parse_image()?);
        }
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok((anchor, images))
    }

    fn parse_image(&mut self) -> Result<ParsedImageV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("image")?;
        self.claim_record(RecordCountV2::Images, opening.physical)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::Image, &name)?;
        let symbol = self.push_field_spelled(name.text.clone(), &name)?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        self.expect_keyword("width")?;
        let width = self.parse_u32_field()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_keyword("height")?;
        let height = self.parse_u32_field()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_keyword("stride")?;
        let stride = self.parse_u32_field()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_keyword("bytes")?;
        self.expect_punctuation(Punctuation::OpenBracket)?;
        let mut bytes = Vec::new();
        if !self.matches_punctuation(Punctuation::CloseBracket) {
            loop {
                let byte = self.parse_unsigned_spelling::<u8>()?;
                self.claim_image_byte(anchor, byte.literal.physical)?;
                bytes.push(byte.literal);
                if !self.matches_punctuation(Punctuation::Comma) {
                    break;
                }
                self.expect_punctuation(Punctuation::Comma)?;
            }
        }
        self.expect_punctuation(Punctuation::CloseBracket)?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedImageV2 {
            name: name.text,
            symbol,
            width,
            height,
            stride,
            bytes,
            anchor,
        })
    }

    fn parse_node(&mut self, parent: Option<&str>) -> Result<ParsedNodeV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("node")?;
        self.claim_record(RecordCountV2::SpatialNodes, opening.physical)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::SpatialNode, &name)?;
        let symbol = self.push_field_spelled(name.text.clone(), &name)?;
        let parent = parent
            .map(|parent| self.push_field_spelled(parent.into(), &name))
            .transpose()?;
        self.expect_punctuation(Punctuation::Colon)?;
        let template_name = self.parse_name()?;
        let template = self.push_field_spelled(template_name.text.clone(), &template_name)?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        let container = self.parse_container()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        let placement = self.parse_placement()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        let transform = self.parse_transform()?;
        self.expect_punctuation(Punctuation::Semicolon)?;

        let mut shapes = Vec::new();
        while self.matches_keyword("shape") {
            shapes.push(self.parse_shape()?);
        }
        let mut brushes = Vec::new();
        while self.matches_keyword("brush") {
            brushes.push(self.parse_brush()?);
        }
        let mut clips = Vec::new();
        while self.matches_keyword("clip") {
            clips.push(self.parse_clip()?);
        }
        let mut paint = Vec::new();
        while self.matches_keyword("paint") {
            paint.push(self.parse_paint()?);
        }
        let mut hit = Vec::new();
        while self.matches_keyword("hit") {
            hit.push(self.parse_hit()?);
        }
        let mut semantics = Vec::new();
        while self.matches_keyword("semantic") {
            semantics.push(self.parse_semantic()?);
        }
        let mut children = Vec::new();
        while self.matches_keyword("node") {
            children.push(self.parse_node(Some(&name.text))?);
        }
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedNodeV2 {
            name: name.text,
            symbol,
            template,
            parent,
            container,
            placement,
            transform,
            shapes,
            brushes,
            clips,
            paint,
            hit,
            semantics,
            children,
            anchor,
        })
    }
}
