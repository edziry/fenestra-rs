use fenestra_ui_ir::prototype::SpatialFillRuleV2;

use crate::diagnostic_v2::AuthoringDiagnosticV2;
use crate::parsed_v2::{
    ParsedCoverageV2, ParsedPathVerbKindV2, ParsedPathVerbV2, ParsedPolygonPointV2,
    ParsedShapeGeometryV2, ParsedShapeV2,
};
use crate::token::Punctuation;
use crate::vocabulary_v2::AnchorKindV2;

use super::super::{ParserV2, RecordCountV2};

impl ParserV2 {
    pub(super) fn parse_shape(&mut self) -> Result<ParsedShapeV2, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("shape")?;
        self.claim_record(RecordCountV2::Shapes, opening.physical)?;
        let name = self.parse_name()?;
        let anchor = self.push_spelled_anchor(AnchorKindV2::SpatialShape, &name)?;
        let symbol = self.push_field_spelled(name.text.clone(), &name)?;
        let geometry = if self.matches_keyword("rect") {
            self.expect_keyword("rect")?;
            self.expect_punctuation(Punctuation::OpenBrace)?;
            self.expect_keyword("origin")?;
            let origin = self.parse_point()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("width")?;
            let width = self.parse_fixed_binding_field()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("height")?;
            let height = self.parse_fixed_binding_field()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_punctuation(Punctuation::CloseBrace)?;
            ParsedShapeGeometryV2::Rect {
                origin,
                width,
                height,
            }
        } else if self.matches_keyword("circle") {
            self.expect_keyword("circle")?;
            self.expect_punctuation(Punctuation::OpenBrace)?;
            self.expect_keyword("center")?;
            let center = self.parse_point()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_keyword("radius")?;
            let radius = self.parse_fixed_binding_field()?;
            self.expect_punctuation(Punctuation::Semicolon)?;
            self.expect_punctuation(Punctuation::CloseBrace)?;
            ParsedShapeGeometryV2::Circle { center, radius }
        } else if self.matches_keyword("polygon") {
            self.expect_keyword("polygon")?;
            self.expect_punctuation(Punctuation::OpenBrace)?;
            let mut points = Vec::new();
            while self.matches_keyword("point") {
                let keyword = self.expect_keyword("point")?;
                self.claim_record(RecordCountV2::PolygonPoints, keyword.physical)?;
                let point_anchor = self.push_anchor(AnchorKindV2::SpatialPolygonPoint, &keyword)?;
                self.expect_punctuation(Punctuation::OpenParenthesis)?;
                let x = self.parse_fixed_binding_field()?;
                self.expect_punctuation(Punctuation::Comma)?;
                let y = self.parse_fixed_binding_field()?;
                self.expect_punctuation(Punctuation::CloseParenthesis)?;
                self.expect_punctuation(Punctuation::Semicolon)?;
                points.push(ParsedPolygonPointV2 {
                    point: crate::parsed_v2::ParsedPointV2 { x, y },
                    anchor: point_anchor,
                });
            }
            self.expect_punctuation(Punctuation::CloseBrace)?;
            ParsedShapeGeometryV2::Polygon(points)
        } else if self.matches_keyword("path") {
            let path = self.expect_keyword("path")?;
            self.claim_record(RecordCountV2::Paths, path.physical)?;
            self.expect_punctuation(Punctuation::OpenBrace)?;
            let mut verbs = Vec::new();
            while self.is_path_verb() {
                verbs.push(self.parse_path_verb()?);
            }
            self.expect_punctuation(Punctuation::CloseBrace)?;
            ParsedShapeGeometryV2::Path(verbs)
        } else {
            return Err(self.unexpected());
        };
        Ok(ParsedShapeV2 {
            name: name.text,
            symbol,
            geometry,
            anchor,
        })
    }

    fn is_path_verb(&self) -> bool {
        ["move_to", "line_to", "quadratic_to", "cubic_to", "close"]
            .iter()
            .any(|keyword| self.matches_keyword(keyword))
    }

    fn parse_path_verb(&mut self) -> Result<ParsedPathVerbV2, AuthoringDiagnosticV2> {
        let keyword = if self.matches_keyword("move_to") {
            "move_to"
        } else if self.matches_keyword("line_to") {
            "line_to"
        } else if self.matches_keyword("quadratic_to") {
            "quadratic_to"
        } else if self.matches_keyword("cubic_to") {
            "cubic_to"
        } else {
            "close"
        };
        let opening = self.expect_keyword(keyword)?;
        self.claim_record(RecordCountV2::PathVerbs, opening.physical)?;
        let anchor = self.push_anchor(AnchorKindV2::SpatialPathVerb, &opening)?;
        let kind = match keyword {
            "move_to" => ParsedPathVerbKindV2::MoveTo(self.parse_point()?),
            "line_to" => ParsedPathVerbKindV2::LineTo(self.parse_point()?),
            "quadratic_to" => ParsedPathVerbKindV2::QuadraticTo {
                control: self.parse_point()?,
                to: self.parse_point()?,
            },
            "cubic_to" => ParsedPathVerbKindV2::CubicTo {
                control1: self.parse_point()?,
                control2: self.parse_point()?,
                to: self.parse_point()?,
            },
            "close" => ParsedPathVerbKindV2::Close,
            _ => unreachable!(),
        };
        self.expect_punctuation(Punctuation::Semicolon)?;
        Ok(ParsedPathVerbV2 { kind, anchor })
    }

    pub(super) fn parse_fill_rule(&mut self) -> Result<SpatialFillRuleV2, AuthoringDiagnosticV2> {
        if self.matches_keyword("non_zero") {
            self.expect_keyword("non_zero")?;
            Ok(SpatialFillRuleV2::NonZero)
        } else if self.matches_keyword("even_odd") {
            self.expect_keyword("even_odd")?;
            Ok(SpatialFillRuleV2::EvenOdd)
        } else {
            Err(self.unexpected())
        }
    }

    pub(super) fn parse_coverage(&mut self) -> Result<ParsedCoverageV2, AuthoringDiagnosticV2> {
        if self.matches_keyword("fill") {
            self.expect_keyword("fill")?;
            let shape = self.parse_name_field()?;
            self.expect_keyword("rule")?;
            let rule = self.parse_fill_rule()?;
            Ok(ParsedCoverageV2::Fill { shape, rule })
        } else if self.matches_keyword("round_stroke") {
            self.expect_keyword("round_stroke")?;
            let shape = self.parse_name_field()?;
            self.expect_keyword("width")?;
            let width = self.parse_fixed_binding_field()?;
            Ok(ParsedCoverageV2::RoundStroke { shape, width })
        } else {
            Err(self.unexpected())
        }
    }
}
