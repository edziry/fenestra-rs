use fenestra_ui_ir::prototype::{SpatialAnchorComponentV2, SpatialAxisV2};

use crate::diagnostic_v2::AuthoringDiagnosticV2;
use crate::parsed_v2::{
    ParsedAnchorPairV2, ParsedAnchorTargetV2, ParsedContainerV2, ParsedDimensionV2,
    ParsedPaddingV2, ParsedPlacementV2, ParsedPointV2, ParsedTransformV2, ParsedViewportV2,
};
use crate::token::Punctuation;
use crate::vocabulary_v2::AnchorKindV2;

use super::super::ParserV2;

const SCALE: i64 = 65_536;

impl ParserV2 {
    pub(super) fn parse_viewport(&mut self) -> Result<ParsedViewportV2, AuthoringDiagnosticV2> {
        self.expect_keyword("viewport")?;
        let container = self.expect_keyword("container")?;
        let anchor = self.push_anchor(AnchorKindV2::SpatialContainer, &container)?;
        let axis = self.parse_axis()?;
        self.expect_keyword("padding")?;
        self.expect_punctuation(Punctuation::OpenParenthesis)?;
        let left = self.parse_i32_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let right = self.parse_i32_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let top = self.parse_i32_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let bottom = self.parse_i32_field()?;
        self.expect_punctuation(Punctuation::CloseParenthesis)?;
        self.expect_keyword("gap")?;
        let gap = self.parse_i32_field()?;
        self.expect_punctuation(Punctuation::Semicolon)?;
        Ok(ParsedViewportV2 {
            axis,
            left,
            right,
            top,
            bottom,
            gap,
            anchor,
        })
    }

    pub(super) fn parse_container(&mut self) -> Result<ParsedContainerV2, AuthoringDiagnosticV2> {
        let keyword = self.expect_keyword("container")?;
        self.push_anchor(AnchorKindV2::SpatialContainer, &keyword)?;
        let axis = self.parse_axis()?;
        self.expect_keyword("padding")?;
        self.expect_punctuation(Punctuation::OpenParenthesis)?;
        let left = self.parse_i32_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let right = self.parse_i32_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let top = self.parse_i32_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let bottom = self.parse_i32_binding_field()?;
        self.expect_punctuation(Punctuation::CloseParenthesis)?;
        self.expect_keyword("gap")?;
        let gap = self.parse_i32_binding_field()?;
        Ok(ParsedContainerV2 {
            axis,
            padding: ParsedPaddingV2 {
                left,
                right,
                top,
                bottom,
            },
            gap,
        })
    }

    fn parse_axis(&mut self) -> Result<SpatialAxisV2, AuthoringDiagnosticV2> {
        if self.matches_keyword("row") {
            self.expect_keyword("row")?;
            Ok(SpatialAxisV2::Row)
        } else if self.matches_keyword("column") {
            self.expect_keyword("column")?;
            Ok(SpatialAxisV2::Column)
        } else {
            Err(self.unexpected())
        }
    }

    pub(super) fn parse_placement(&mut self) -> Result<ParsedPlacementV2, AuthoringDiagnosticV2> {
        let keyword = self.expect_keyword("placement")?;
        self.push_anchor(AnchorKindV2::SpatialPlacement, &keyword)?;
        if self.matches_keyword("layout") {
            self.expect_keyword("layout")?;
            self.expect_keyword("width")?;
            let width = self.parse_dimension()?;
            self.expect_keyword("height")?;
            let height = self.parse_dimension()?;
            return Ok(ParsedPlacementV2::Layout { width, height });
        }
        self.expect_keyword("free")?;
        self.expect_keyword("width")?;
        let width = self.parse_i32_binding_field()?;
        self.expect_keyword("height")?;
        let height = self.parse_i32_binding_field()?;
        self.expect_keyword("self_anchor")?;
        let self_anchor = self.parse_anchor_pair()?;
        self.expect_keyword("target")?;
        let target = if self.matches_keyword("viewport") {
            self.expect_keyword("viewport")?;
            ParsedAnchorTargetV2::Viewport
        } else if self.matches_keyword("parent") {
            self.expect_keyword("parent")?;
            ParsedAnchorTargetV2::Parent
        } else if self.matches_keyword("node") {
            self.expect_keyword("node")?;
            ParsedAnchorTargetV2::Node(self.parse_name_field()?)
        } else {
            return Err(self.unexpected());
        };
        self.expect_keyword("target_anchor")?;
        let target_anchor = self.parse_anchor_pair()?;
        self.expect_keyword("offset")?;
        let offset = self.parse_point()?;
        Ok(ParsedPlacementV2::Free {
            width,
            height,
            self_anchor,
            target,
            target_anchor,
            offset,
        })
    }

    fn parse_dimension(&mut self) -> Result<ParsedDimensionV2, AuthoringDiagnosticV2> {
        self.expect_keyword("dimension")?;
        self.expect_punctuation(Punctuation::OpenParenthesis)?;
        let minimum = self.parse_i32_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let preferred = self.parse_i32_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let maximum = self.parse_i32_binding_field()?;
        self.expect_punctuation(Punctuation::CloseParenthesis)?;
        Ok(ParsedDimensionV2 {
            minimum,
            preferred,
            maximum,
        })
    }

    fn parse_anchor_pair(&mut self) -> Result<ParsedAnchorPairV2, AuthoringDiagnosticV2> {
        self.expect_keyword("anchor")?;
        self.expect_punctuation(Punctuation::OpenParenthesis)?;
        let horizontal = self.parse_anchor_component()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let vertical = self.parse_anchor_component()?;
        self.expect_punctuation(Punctuation::CloseParenthesis)?;
        Ok(ParsedAnchorPairV2 {
            horizontal,
            vertical,
        })
    }

    fn parse_anchor_component(
        &mut self,
    ) -> Result<SpatialAnchorComponentV2, AuthoringDiagnosticV2> {
        for (keyword, value) in [
            ("start", SpatialAnchorComponentV2::Start),
            ("center", SpatialAnchorComponentV2::Center),
            ("end", SpatialAnchorComponentV2::End),
        ] {
            if self.matches_keyword(keyword) {
                self.expect_keyword(keyword)?;
                return Ok(value);
            }
        }
        Err(self.unexpected())
    }

    pub(super) fn parse_point(&mut self) -> Result<ParsedPointV2, AuthoringDiagnosticV2> {
        self.expect_keyword("point")?;
        self.expect_punctuation(Punctuation::OpenParenthesis)?;
        let x = self.parse_fixed_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let y = self.parse_fixed_binding_field()?;
        self.expect_punctuation(Punctuation::CloseParenthesis)?;
        Ok(ParsedPointV2 { x, y })
    }

    pub(super) fn parse_transform(&mut self) -> Result<ParsedTransformV2, AuthoringDiagnosticV2> {
        let keyword = self.expect_keyword("transform")?;
        let anchor = self.push_anchor(AnchorKindV2::SpatialTransform, &keyword)?;
        let mut invalid_turn = None;
        let coefficients = if self.matches_keyword("identity") {
            let token = self.expect_keyword("identity")?;
            Some(self.constant_transform([SCALE, 0, 0, SCALE, 0, 0], &token)?)
        } else if self.matches_keyword("translate") {
            let token = self.expect_keyword("translate")?;
            let a = self.constant_fixed_field(SCALE, token.label(), token.physical)?;
            let b = self.constant_fixed_field(0, token.label(), token.physical)?;
            let c = self.constant_fixed_field(0, token.label(), token.physical)?;
            let d = self.constant_fixed_field(SCALE, token.label(), token.physical)?;
            self.expect_punctuation(Punctuation::OpenParenthesis)?;
            self.expect_keyword("point")?;
            self.expect_punctuation(Punctuation::OpenParenthesis)?;
            let tx = self.parse_fixed_binding_field()?;
            self.expect_punctuation(Punctuation::Comma)?;
            let ty = self.parse_fixed_binding_field()?;
            self.expect_punctuation(Punctuation::CloseParenthesis)?;
            self.expect_punctuation(Punctuation::CloseParenthesis)?;
            Some([a, b, c, d, tx, ty])
        } else if self.matches_keyword("scale") {
            let token = self.expect_keyword("scale")?;
            let b = self.constant_fixed_field(0, token.label(), token.physical)?;
            let c = self.constant_fixed_field(0, token.label(), token.physical)?;
            let tx = self.constant_fixed_field(0, token.label(), token.physical)?;
            let ty = self.constant_fixed_field(0, token.label(), token.physical)?;
            self.expect_punctuation(Punctuation::OpenParenthesis)?;
            let a = self.parse_fixed_binding_field()?;
            self.expect_punctuation(Punctuation::Comma)?;
            let d = self.parse_fixed_binding_field()?;
            self.expect_punctuation(Punctuation::CloseParenthesis)?;
            Some([a, b, c, d, tx, ty])
        } else if self.matches_keyword("quarter_turn") {
            self.expect_keyword("quarter_turn")?;
            self.expect_punctuation(Punctuation::OpenParenthesis)?;
            let turn = self.parse_unsigned_spelling::<u32>()?;
            self.expect_punctuation(Punctuation::CloseParenthesis)?;
            match turn.literal.value {
                Ok(value @ 0..=3) => {
                    let values = match value {
                        0 => [SCALE, 0, 0, SCALE, 0, 0],
                        1 => [0, SCALE, -SCALE, 0, 0, 0],
                        2 => [-SCALE, 0, 0, -SCALE, 0, 0],
                        3 => [0, -SCALE, SCALE, 0, 0, 0],
                        _ => unreachable!(),
                    };
                    Some(self.constant_transform_parts(
                        values,
                        &turn.label,
                        turn.literal.physical,
                    )?)
                }
                _ => {
                    invalid_turn = Some(turn.literal.physical);
                    None
                }
            }
        } else if self.matches_keyword("affine") {
            self.expect_keyword("affine")?;
            self.expect_punctuation(Punctuation::OpenParenthesis)?;
            let values = self.parse_six_fixed_bindings()?;
            self.expect_punctuation(Punctuation::CloseParenthesis)?;
            Some(values)
        } else {
            return Err(self.unexpected());
        };
        self.expect_keyword("origin")?;
        let origin = self.parse_point()?;
        Ok(ParsedTransformV2 {
            coefficients,
            origin,
            invalid_turn,
            anchor,
        })
    }

    fn parse_six_fixed_bindings(
        &mut self,
    ) -> Result<[crate::parsed_v2::ParsedFixedBindingFieldV2; 6], AuthoringDiagnosticV2> {
        let a = self.parse_fixed_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let b = self.parse_fixed_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let c = self.parse_fixed_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let d = self.parse_fixed_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let tx = self.parse_fixed_binding_field()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let ty = self.parse_fixed_binding_field()?;
        Ok([a, b, c, d, tx, ty])
    }

    fn constant_transform(
        &mut self,
        values: [i64; 6],
        token: &crate::token::AbstractToken<crate::source_v2::PhysicalOriginV2>,
    ) -> Result<[crate::parsed_v2::ParsedFixedBindingFieldV2; 6], AuthoringDiagnosticV2> {
        self.constant_transform_parts(values, token.label(), token.physical)
    }

    fn constant_transform_parts(
        &mut self,
        values: [i64; 6],
        label: &str,
        physical: crate::source_v2::PhysicalOriginV2,
    ) -> Result<[crate::parsed_v2::ParsedFixedBindingFieldV2; 6], AuthoringDiagnosticV2> {
        Ok([
            self.constant_fixed_field(values[0], label, physical)?,
            self.constant_fixed_field(values[1], label, physical)?,
            self.constant_fixed_field(values[2], label, physical)?,
            self.constant_fixed_field(values[3], label, physical)?,
            self.constant_fixed_field(values[4], label, physical)?,
            self.constant_fixed_field(values[5], label, physical)?,
        ])
    }
}
