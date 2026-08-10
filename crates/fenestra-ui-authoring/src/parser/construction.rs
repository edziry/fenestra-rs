use crate::diagnostic::AuthoringDiagnosticV1;
use crate::parsed::{
    ParsedChildV1, ParsedConstructionV1, ParsedInitialKeyV1, ParsedInitialPropertyV1,
    ParsedRegionV1, ParsedStyleAssignmentV1, ParsedStyleV1, ParsedTemplateItemV1, ParsedTemplateV1,
};
use crate::token::PunctuationV1;
use crate::vocabulary::AnchorKindV1;

use super::{ParserV1, RecordCountV1};

impl ParserV1 {
    pub(super) fn parse_construction(
        &mut self,
    ) -> Result<ParsedConstructionV1, AuthoringDiagnosticV1> {
        let keyword = self.expect_keyword("construction")?;
        let anchor = self.push_anchor(AnchorKindV1::Construction, &keyword)?;
        self.expect_punctuation(PunctuationV1::OpenBrace)?;

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
        self.expect_punctuation(PunctuationV1::CloseBrace)?;
        Ok(ParsedConstructionV1 {
            templates,
            regions,
            anchor,
        })
    }

    fn parse_template(&mut self) -> Result<ParsedTemplateV1, AuthoringDiagnosticV1> {
        let opening = self.expect_keyword("template")?;
        self.claim_record(RecordCountV1::Templates, &opening)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV1::Template, &name)?;
        self.expect_punctuation(PunctuationV1::Equals)?;
        let id = self.parse_u32()?;
        self.expect_punctuation(PunctuationV1::Colon)?;
        let component = self.parse_name()?;
        self.expect_punctuation(PunctuationV1::OpenBrace)?;

        let mut items = Vec::new();
        while self.matches_keyword("set") || self.matches_keyword("child") {
            if self.matches_keyword("set") {
                items.push(ParsedTemplateItemV1::Initial(
                    self.parse_initial_property()?,
                ));
            } else {
                items.push(ParsedTemplateItemV1::Child(self.parse_child()?));
            }
        }
        self.expect_punctuation(PunctuationV1::CloseBrace)?;
        Ok(ParsedTemplateV1 {
            name: name.text,
            id,
            component: self.spanned_name(component),
            items,
            anchor,
        })
    }

    fn parse_initial_property(&mut self) -> Result<ParsedInitialPropertyV1, AuthoringDiagnosticV1> {
        let opening = self.expect_keyword("set")?;
        self.claim_record(RecordCountV1::InitialProperties, &opening)?;
        let property = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV1::InitialProperty, &property)?;
        self.expect_punctuation(PunctuationV1::Equals)?;
        let value = self.parse_value()?;
        self.expect_punctuation(PunctuationV1::Semicolon)?;
        Ok(ParsedInitialPropertyV1 {
            property: property.text,
            value,
            anchor,
        })
    }

    fn parse_child(&mut self) -> Result<ParsedChildV1, AuthoringDiagnosticV1> {
        let opening = self.expect_keyword("child")?;
        self.claim_record(RecordCountV1::ChildSlots, &opening)?;
        let kind = if self.matches_keyword("template") {
            self.expect_keyword("template")?;
            AnchorKindV1::StaticChild
        } else if self.matches_keyword("region") {
            self.expect_keyword("region")?;
            AnchorKindV1::RegionChild
        } else {
            return Err(self.unexpected());
        };
        let referenced = self.parse_name()?;
        let anchor = self.push_spelled_anchor(kind, &referenced)?;
        self.expect_punctuation(PunctuationV1::Semicolon)?;
        Ok(if kind == AnchorKindV1::StaticChild {
            ParsedChildV1::Static {
                template: referenced.text,
                anchor,
            }
        } else {
            ParsedChildV1::Region {
                region: referenced.text,
                anchor,
            }
        })
    }

    fn parse_region(&mut self) -> Result<ParsedRegionV1, AuthoringDiagnosticV1> {
        let opening = self.expect_keyword("region")?;
        self.claim_record(RecordCountV1::Regions, &opening)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV1::Region, &name)?;
        self.expect_punctuation(PunctuationV1::Equals)?;
        let id = self.parse_u32()?;
        self.expect_keyword("owner")?;
        let owner = self.parse_name()?;
        self.expect_keyword("repeat")?;
        let repeat_body = self.parse_name()?;
        self.expect_keyword("keys")?;
        self.expect_punctuation(PunctuationV1::OpenBracket)?;

        let mut initial_keys = Vec::new();
        if !self.matches_punctuation(PunctuationV1::CloseBracket) {
            loop {
                let key = self.take_unsigned()?;
                self.claim_spelled_record(RecordCountV1::InitialKeys, &key)?;
                let key_anchor = self.push_spelled_anchor(AnchorKindV1::InitialKey, &key)?;
                initial_keys.push(ParsedInitialKeyV1 {
                    value: self.parse_u64_spelled(&key),
                    anchor: key_anchor,
                });
                if !self.matches_punctuation(PunctuationV1::Comma) {
                    break;
                }
                self.expect_punctuation(PunctuationV1::Comma)?;
            }
        }
        self.expect_punctuation(PunctuationV1::CloseBracket)?;
        self.expect_keyword("invalidates")?;
        let invalidation = self.parse_invalidation_set()?;
        self.expect_punctuation(PunctuationV1::Semicolon)?;
        Ok(ParsedRegionV1 {
            name: name.text,
            id,
            owner: self.spanned_name(owner),
            repeat_body: self.spanned_name(repeat_body),
            initial_keys,
            invalidation,
            anchor,
        })
    }

    pub(super) fn parse_style(&mut self) -> Result<ParsedStyleV1, AuthoringDiagnosticV1> {
        let keyword = self.expect_keyword("style")?;
        let anchor = self.push_anchor(AnchorKindV1::Style, &keyword)?;
        self.expect_punctuation(PunctuationV1::OpenBrace)?;
        let mut assignments = Vec::new();
        while self.matches_keyword("set") {
            assignments.push(self.parse_style_assignment()?);
        }
        self.expect_punctuation(PunctuationV1::CloseBrace)?;
        Ok(ParsedStyleV1 {
            assignments,
            anchor,
        })
    }

    fn parse_style_assignment(&mut self) -> Result<ParsedStyleAssignmentV1, AuthoringDiagnosticV1> {
        let opening = self.expect_keyword("set")?;
        self.claim_record(RecordCountV1::StyleAssignments, &opening)?;
        let target = self.parse_name()?;
        self.expect_punctuation(PunctuationV1::Dot)?;
        let property = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV1::StyleAssignment, &property)?;
        self.expect_punctuation(PunctuationV1::Equals)?;
        let value = self.parse_value()?;
        self.expect_punctuation(PunctuationV1::Semicolon)?;
        Ok(ParsedStyleAssignmentV1 {
            target: self.spanned_name(target),
            property: property.text,
            value,
            anchor,
        })
    }
}
