mod logical;
mod spatial;
mod support;
#[cfg(test)]
mod tests;
mod value;

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::{AuthoringLimitKindV2, AuthoringLimitsV2};
use crate::parsed_v2::{ParsedAnchorV2, ParsedDocumentV2};
use crate::source_v2::PhysicalOriginV2;
use crate::token::AbstractToken;
use crate::version_v2::SUPPORTED_AUTHORING_FORMAT_V2;
use crate::vocabulary_v2::{AnchorKindV2, AuthoringFrontendV2};

pub(crate) fn parse_document_v2(
    frontend: AuthoringFrontendV2,
    eof: PhysicalOriginV2,
    tokens: Vec<AbstractToken<PhysicalOriginV2>>,
    limits: AuthoringLimitsV2,
) -> Result<ParsedDocumentV2, AuthoringDiagnosticV2> {
    ParserV2::new(frontend, eof, tokens, limits).parse_document()
}

pub(super) struct ParserV2 {
    frontend: AuthoringFrontendV2,
    eof: PhysicalOriginV2,
    tokens: Vec<AbstractToken<PhysicalOriginV2>>,
    next: usize,
    limits: AuthoringLimitsV2,
    anchors: Vec<ParsedAnchorV2>,
    record_counts: [usize; 22],
}

impl ParserV2 {
    fn new(
        frontend: AuthoringFrontendV2,
        eof: PhysicalOriginV2,
        tokens: Vec<AbstractToken<PhysicalOriginV2>>,
        limits: AuthoringLimitsV2,
    ) -> Self {
        Self {
            frontend,
            eof,
            tokens,
            next: 0,
            limits,
            anchors: Vec::new(),
            record_counts: [0; 22],
        }
    }

    fn parse_document(mut self) -> Result<ParsedDocumentV2, AuthoringDiagnosticV2> {
        let keyword = self.expect_keyword("format")?;
        let document_anchor = self.push_anchor(AnchorKindV2::Document, &keyword)?;
        let format = self.parse_u32()?;
        let format = format.value.map_err(|physical| {
            self.anchored_failure_at(
                AuthoringDiagnosticKindV2::InvalidLiteral,
                document_anchor,
                physical,
            )
        })?;
        if format != SUPPORTED_AUTHORING_FORMAT_V2.get() {
            return Err(self.anchored_failure(
                AuthoringDiagnosticKindV2::UnsupportedAuthoringFormat,
                document_anchor,
            ));
        }
        self.expect_punctuation(crate::token::Punctuation::Semicolon)?;

        let schema = self.parse_schema()?;
        let construction = self.parse_construction()?;
        let style = self.parse_style()?;
        let spatial = self.parse_spatial()?;
        self.expect_eof()?;

        Ok(ParsedDocumentV2 {
            frontend: self.frontend,
            format,
            document_anchor,
            schema,
            construction,
            style,
            spatial,
            anchors: self.anchors,
        })
    }
}

pub(super) struct SpelledTokenV2 {
    pub(super) text: Box<str>,
    pub(super) physical: PhysicalOriginV2,
}

#[derive(Clone, Copy)]
#[repr(usize)]
pub(super) enum RecordCountV2 {
    Components,
    Properties,
    Templates,
    Regions,
    ChildSlots,
    InitialProperties,
    InitialKeys,
    StyleAssignments,
    Images,
    ImageBytes,
    SpatialNodes,
    SpatialFields,
    Shapes,
    Paths,
    PathVerbs,
    PolygonPoints,
    Brushes,
    GradientStops,
    Clips,
    PaintItems,
    HitItems,
    SemanticItems,
}

impl RecordCountV2 {
    const fn limit_kind(self) -> AuthoringLimitKindV2 {
        match self {
            Self::Components => AuthoringLimitKindV2::Components,
            Self::Properties => AuthoringLimitKindV2::Properties,
            Self::Templates => AuthoringLimitKindV2::Templates,
            Self::Regions => AuthoringLimitKindV2::Regions,
            Self::ChildSlots => AuthoringLimitKindV2::ChildSlots,
            Self::InitialProperties => AuthoringLimitKindV2::InitialProperties,
            Self::InitialKeys => AuthoringLimitKindV2::InitialKeys,
            Self::StyleAssignments => AuthoringLimitKindV2::StyleAssignments,
            Self::Images => AuthoringLimitKindV2::Images,
            Self::ImageBytes => AuthoringLimitKindV2::ImageBytes,
            Self::SpatialNodes => AuthoringLimitKindV2::SpatialNodes,
            Self::SpatialFields => AuthoringLimitKindV2::SpatialFields,
            Self::Shapes => AuthoringLimitKindV2::Shapes,
            Self::Paths => AuthoringLimitKindV2::Paths,
            Self::PathVerbs => AuthoringLimitKindV2::PathVerbs,
            Self::PolygonPoints => AuthoringLimitKindV2::PolygonPoints,
            Self::Brushes => AuthoringLimitKindV2::Brushes,
            Self::GradientStops => AuthoringLimitKindV2::GradientStops,
            Self::Clips => AuthoringLimitKindV2::Clips,
            Self::PaintItems => AuthoringLimitKindV2::PaintItems,
            Self::HitItems => AuthoringLimitKindV2::HitItems,
            Self::SemanticItems => AuthoringLimitKindV2::SemanticItems,
        }
    }
}
