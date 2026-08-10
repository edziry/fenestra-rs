mod construction;
mod support;
#[cfg(test)]
mod tests;
mod value;

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::limits::{AuthoringLimitKindV1, AuthoringLimitsV1};
use crate::parsed::{
    ParsedAnchorV1, ParsedComponentV1, ParsedDocumentV1, ParsedPropertyV1, ParsedSchemaV1,
};
use crate::source::PhysicalOriginV1;
use crate::token::{AbstractTokenV1, PunctuationV1};
use crate::version::SUPPORTED_AUTHORING_FORMAT;
use crate::vocabulary::{AnchorKindV1, AuthoringFrontendV1};

pub(crate) fn parse_document_v1(
    frontend: AuthoringFrontendV1,
    eof: PhysicalOriginV1,
    tokens: Vec<AbstractTokenV1>,
    limits: AuthoringLimitsV1,
) -> Result<ParsedDocumentV1, AuthoringDiagnosticV1> {
    ParserV1::new(frontend, eof, tokens, limits).parse_document()
}

pub(super) struct ParserV1 {
    frontend: AuthoringFrontendV1,
    eof: PhysicalOriginV1,
    tokens: Vec<AbstractTokenV1>,
    next: usize,
    limits: AuthoringLimitsV1,
    anchors: Vec<ParsedAnchorV1>,
    record_counts: [usize; 8],
}

impl ParserV1 {
    fn new(
        frontend: AuthoringFrontendV1,
        eof: PhysicalOriginV1,
        tokens: Vec<AbstractTokenV1>,
        limits: AuthoringLimitsV1,
    ) -> Self {
        Self {
            frontend,
            eof,
            tokens,
            next: 0,
            limits,
            anchors: Vec::new(),
            record_counts: [0; 8],
        }
    }

    fn parse_document(mut self) -> Result<ParsedDocumentV1, AuthoringDiagnosticV1> {
        let format_keyword = self.expect_keyword("format")?;
        let document_anchor = self.push_anchor(AnchorKindV1::Document, &format_keyword)?;
        let format = self.parse_u32(document_anchor)?;
        if format != SUPPORTED_AUTHORING_FORMAT.get() {
            return Err(self.anchored_failure(
                AuthoringDiagnosticKindV1::UnsupportedAuthoringFormat,
                document_anchor,
            ));
        }
        self.expect_punctuation(PunctuationV1::Semicolon)?;

        let schema = self.parse_schema()?;
        let construction = self.parse_construction()?;
        let style = self.parse_style()?;
        self.expect_eof()?;

        Ok(ParsedDocumentV1 {
            frontend: self.frontend,
            format,
            document_anchor,
            schema,
            construction,
            style,
            anchors: self.anchors,
        })
    }

    fn parse_schema(&mut self) -> Result<ParsedSchemaV1, AuthoringDiagnosticV1> {
        let schema_keyword = self.expect_keyword("schema")?;
        let anchor = self.push_anchor(AnchorKindV1::Schema, &schema_keyword)?;
        self.expect_keyword("namespace")?;
        let namespace = self.parse_u64(anchor)?;
        self.expect_keyword("revision")?;
        let revision = self.parse_u32(anchor)?;
        self.expect_punctuation(PunctuationV1::OpenBrace)?;

        if !self.matches_keyword("component") {
            return Err(self.unexpected());
        }
        let mut components = Vec::new();
        while self.matches_keyword("component") {
            components.push(self.parse_component()?);
        }
        self.expect_punctuation(PunctuationV1::CloseBrace)?;
        Ok(ParsedSchemaV1 {
            namespace,
            revision,
            components,
            anchor,
        })
    }

    fn parse_component(&mut self) -> Result<ParsedComponentV1, AuthoringDiagnosticV1> {
        let opening = self.expect_keyword("component")?;
        self.claim_record(RecordCountV1::Components, &opening)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV1::Component, &name)?;
        self.expect_punctuation(PunctuationV1::Equals)?;
        let id = self.parse_u32(anchor)?;
        self.expect_punctuation(PunctuationV1::OpenBrace)?;

        if !self.matches_keyword("property") {
            return Err(self.unexpected());
        }
        let mut properties = Vec::new();
        while self.matches_keyword("property") {
            properties.push(self.parse_property()?);
        }
        self.expect_punctuation(PunctuationV1::CloseBrace)?;
        Ok(ParsedComponentV1 {
            name: name.text,
            id,
            properties,
            anchor,
        })
    }

    fn parse_property(&mut self) -> Result<ParsedPropertyV1, AuthoringDiagnosticV1> {
        let opening = self.expect_keyword("property")?;
        self.claim_record(RecordCountV1::Properties, &opening)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV1::Property, &name)?;
        self.expect_punctuation(PunctuationV1::Equals)?;
        let id = self.parse_u32(anchor)?;
        self.expect_punctuation(PunctuationV1::Colon)?;
        let value_type = self.parse_value_type()?;
        self.expect_punctuation(PunctuationV1::Equals)?;
        let default = self.parse_value(anchor)?;
        self.expect_keyword("invalidates")?;
        let invalidation = self.parse_invalidation_set()?;
        self.expect_punctuation(PunctuationV1::Semicolon)?;
        Ok(ParsedPropertyV1 {
            name: name.text,
            id,
            value_type,
            default,
            invalidation,
            anchor,
        })
    }
}

pub(super) struct SpelledTokenV1 {
    pub(super) text: Box<str>,
    pub(super) physical: PhysicalOriginV1,
}

#[derive(Clone, Copy)]
pub(super) enum RecordCountV1 {
    Components,
    Properties,
    Templates,
    Regions,
    ChildSlots,
    InitialProperties,
    InitialKeys,
    StyleAssignments,
}

impl RecordCountV1 {
    const fn limit_kind(self) -> AuthoringLimitKindV1 {
        match self {
            Self::Components => AuthoringLimitKindV1::Components,
            Self::Properties => AuthoringLimitKindV1::Properties,
            Self::Templates => AuthoringLimitKindV1::Templates,
            Self::Regions => AuthoringLimitKindV1::Regions,
            Self::ChildSlots => AuthoringLimitKindV1::ChildSlots,
            Self::InitialProperties => AuthoringLimitKindV1::InitialProperties,
            Self::InitialKeys => AuthoringLimitKindV1::InitialKeys,
            Self::StyleAssignments => AuthoringLimitKindV1::StyleAssignments,
        }
    }
}
