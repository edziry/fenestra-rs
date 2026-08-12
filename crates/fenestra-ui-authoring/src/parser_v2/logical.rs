use crate::diagnostic_v2::AuthoringDiagnosticV2;
use crate::parsed_v2::{
    ParsedChildV2, ParsedComponentV2, ParsedConstructionV2, ParsedInitialKeyV2,
    ParsedInitialPropertyV2, ParsedPropertyV2, ParsedRegionV2, ParsedSchemaV2,
    ParsedStyleAssignmentV2, ParsedStyleV2, ParsedTemplateItemV2, ParsedTemplateV2,
};
use crate::token::Punctuation;
use crate::vocabulary_v2::AnchorKindV2;

use super::{ParserV2, RecordCountV2};

impl ParserV2 {
    pub(super) fn parse_schema(&mut self) -> Result<ParsedSchemaV2, AuthoringDiagnosticV2> {
        let keyword = self.expect_keyword("schema")?;
        let anchor = self.push_anchor(AnchorKindV2::Schema, &keyword)?;
        self.expect_keyword("namespace")?;
        let namespace = self.parse_u64()?;
        self.expect_keyword("revision")?;
        let revision = self.parse_u32()?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        if !self.matches_keyword("component") {
            return Err(self.unexpected());
        }
        let mut components = Vec::new();
        while self.matches_keyword("component") {
            components.push(self.parse_component()?);
        }
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedSchemaV2 {
            namespace,
            revision,
            components,
            anchor,
        })
    }

    fn parse_component(&mut self) -> Result<ParsedComponentV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("component")?;
        self.claim_record(RecordCountV2::Components, opening.physical)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::Component, &name)?;
        self.expect_punctuation(Punctuation::Equals)?;
        let id = self.parse_u32()?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        if !self.matches_keyword("property") {
            return Err(self.unexpected());
        }
        let mut properties = Vec::new();
        while self.matches_keyword("property") {
            properties.push(self.parse_property()?);
        }
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedComponentV2 {
            name: name.text,
            id,
            properties,
            anchor,
        })
    }

    fn parse_property(&mut self) -> Result<ParsedPropertyV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("property")?;
        self.claim_record(RecordCountV2::Properties, opening.physical)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::Property, &name)?;
        self.expect_punctuation(Punctuation::Equals)?;
        let id = self.parse_u32()?;
        self.expect_punctuation(Punctuation::Colon)?;
        let value_type = self.parse_value_type()?;
        self.expect_punctuation(Punctuation::Equals)?;
        let default = self.parse_value()?;
        self.expect_keyword("invalidates")?;
        let invalidation = self.parse_invalidation_set()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        Ok(ParsedPropertyV2 {
            name: name.text,
            id,
            value_type,
            default,
            invalidation,
            anchor,
        })
    }

    pub(super) fn parse_construction(
        &mut self,
    ) -> Result<ParsedConstructionV2, AuthoringDiagnosticV2> {
        let keyword = self.expect_keyword("construction")?;
        let anchor = self.push_anchor(AnchorKindV2::Construction, &keyword)?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        if !self.matches_keyword("template") {
            return Err(self.unexpected());
        }
        let mut templates = Vec::new();
        while self.matches_keyword("template") {
            templates.push(self.parse_template()?);
        }
        if !self.matches_keyword("region") {
            return Err(self.unexpected());
        }
        let mut regions = Vec::new();
        while self.matches_keyword("region") {
            regions.push(self.parse_region()?);
        }
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedConstructionV2 {
            templates,
            regions,
            anchor,
        })
    }

    fn parse_template(&mut self) -> Result<ParsedTemplateV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("template")?;
        self.claim_record(RecordCountV2::Templates, opening.physical)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::Template, &name)?;
        self.expect_punctuation(Punctuation::Equals)?;
        let id = self.parse_u32()?;
        self.expect_punctuation(Punctuation::Colon)?;
        let component = self.parse_name()?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        let mut items = Vec::new();
        while self.matches_keyword("set") || self.matches_keyword("child") {
            if self.matches_keyword("set") {
                items.push(ParsedTemplateItemV2::Initial(
                    self.parse_initial_property()?,
                ));
            } else {
                items.push(ParsedTemplateItemV2::Child(self.parse_child()?));
            }
        }
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedTemplateV2 {
            name: name.text,
            id,
            component: self.spanned_name(component),
            items,
            anchor,
        })
    }

    fn parse_initial_property(&mut self) -> Result<ParsedInitialPropertyV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("set")?;
        self.claim_record(RecordCountV2::InitialProperties, opening.physical)?;
        let property = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::InitialProperty, &property)?;
        self.expect_punctuation(Punctuation::Equals)?;
        let value = self.parse_value()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        Ok(ParsedInitialPropertyV2 {
            property: property.text,
            value,
            anchor,
        })
    }

    fn parse_child(&mut self) -> Result<ParsedChildV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("child")?;
        self.claim_record(RecordCountV2::ChildSlots, opening.physical)?;
        let kind = if self.matches_keyword("template") {
            self.expect_keyword("template")?;
            AnchorKindV2::StaticChild
        } else if self.matches_keyword("region") {
            self.expect_keyword("region")?;
            AnchorKindV2::RegionChild
        } else {
            return Err(self.unexpected());
        };
        let referenced = self.parse_name()?;
        let anchor = self.push_spelled_anchor(kind, &referenced)?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        Ok(if kind == AnchorKindV2::StaticChild {
            ParsedChildV2::Static {
                template: referenced.text,
                anchor,
            }
        } else {
            ParsedChildV2::Region {
                region: referenced.text,
                anchor,
            }
        })
    }

    fn parse_region(&mut self) -> Result<ParsedRegionV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("region")?;
        self.claim_record(RecordCountV2::Regions, opening.physical)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::Region, &name)?;
        self.expect_punctuation(Punctuation::Equals)?;
        let id = self.parse_u32()?;
        self.expect_keyword("owner")?;
        let owner = self.parse_name()?;
        self.expect_keyword("repeat")?;
        let repeat_body = self.parse_name()?;
        self.expect_keyword("keys")?;
        self.expect_punctuation(Punctuation::OpenBracket)?;
        let mut initial_keys = Vec::new();
        if !self.matches_punctuation(Punctuation::CloseBracket) {
            loop {
                let key = self.take_unsigned()?;
                self.claim_record(RecordCountV2::InitialKeys, key.physical)?;
                let key_anchor = self.push_spelled_anchor(AnchorKindV2::InitialKey, &key)?;
                initial_keys.push(ParsedInitialKeyV2 {
                    value: self.parse_u64_spelled(&key),
                    anchor: key_anchor,
                });
                if !self.matches_punctuation(Punctuation::Comma) {
                    break;
                }
                self.expect_punctuation(Punctuation::Comma)?;
            }
        }
        self.expect_punctuation(Punctuation::CloseBracket)?;
        self.expect_keyword("invalidates")?;
        let invalidation = self.parse_invalidation_set()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        Ok(ParsedRegionV2 {
            name: name.text,
            id,
            owner: self.spanned_name(owner),
            repeat_body: self.spanned_name(repeat_body),
            initial_keys,
            invalidation,
            anchor,
        })
    }

    pub(super) fn parse_style(&mut self) -> Result<ParsedStyleV2, AuthoringDiagnosticV2> {
        let keyword = self.expect_keyword("style")?;
        let anchor = self.push_anchor(AnchorKindV2::Style, &keyword)?;
        self.expect_punctuation(Punctuation::OpenBrace)?;
        let mut assignments = Vec::new();
        while self.matches_keyword("set") {
            assignments.push(self.parse_style_assignment()?);
        }
        self.expect_punctuation(Punctuation::CloseBrace)?;
        Ok(ParsedStyleV2 {
            assignments,
            anchor,
        })
    }

    fn parse_style_assignment(&mut self) -> Result<ParsedStyleAssignmentV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("set")?;
        self.claim_record(RecordCountV2::StyleAssignments, opening.physical)?;
        let target = self.parse_name()?;
        self.expect_punctuation(Punctuation::Dot)?;
        let property = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::StyleAssignment, &property)?;
        self.expect_punctuation(Punctuation::Equals)?;
        let value = self.parse_value()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        Ok(ParsedStyleAssignmentV2 {
            target: self.spanned_name(target),
            property: property.text,
            value,
            anchor,
        })
    }
}
