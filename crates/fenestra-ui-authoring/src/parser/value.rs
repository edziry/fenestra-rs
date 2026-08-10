use fenestra_ui_ir::prototype::{
    InputPolicy, InvalidationClass, InvalidationSet, PropertyValue, ValueType,
};

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::token::PunctuationV1;

use super::{ParserV1, SpelledTokenV1};

impl ParserV1 {
    pub(super) fn parse_u32(&mut self, anchor: u32) -> Result<u32, AuthoringDiagnosticV1> {
        let token = self.take_unsigned()?;
        let value = self.parse_u64_spelled(&token, anchor)?;
        u32::try_from(value)
            .map_err(|_| self.anchored_failure(AuthoringDiagnosticKindV1::InvalidLiteral, anchor))
    }

    pub(super) fn parse_u64(&mut self, anchor: u32) -> Result<u64, AuthoringDiagnosticV1> {
        let token = self.take_unsigned()?;
        self.parse_u64_spelled(&token, anchor)
    }

    pub(super) fn parse_u64_spelled(
        &self,
        token: &SpelledTokenV1,
        anchor: u32,
    ) -> Result<u64, AuthoringDiagnosticV1> {
        if token.text.len() > 1 && token.text.starts_with('0') {
            return Err(self.anchored_failure(AuthoringDiagnosticKindV1::InvalidLiteral, anchor));
        }
        token
            .text
            .parse::<u64>()
            .map_err(|_| self.anchored_failure(AuthoringDiagnosticKindV1::InvalidLiteral, anchor))
    }

    pub(super) fn parse_value_type(&mut self) -> Result<ValueType, AuthoringDiagnosticV1> {
        if self.matches_keyword("bool") {
            self.expect_keyword("bool")?;
            Ok(ValueType::Bool)
        } else if self.matches_keyword("scalar_i32") {
            self.expect_keyword("scalar_i32")?;
            Ok(ValueType::ScalarI32)
        } else if self.matches_keyword("rgba8") {
            self.expect_keyword("rgba8")?;
            Ok(ValueType::Rgba8)
        } else if self.matches_keyword("input_policy") {
            self.expect_keyword("input_policy")?;
            Ok(ValueType::InputPolicy)
        } else {
            Err(self.unexpected())
        }
    }

    pub(super) fn parse_value(
        &mut self,
        anchor: u32,
    ) -> Result<PropertyValue, AuthoringDiagnosticV1> {
        if self.matches_keyword("true") {
            self.expect_keyword("true")?;
            return Ok(PropertyValue::Bool(true));
        }
        if self.matches_keyword("false") {
            self.expect_keyword("false")?;
            return Ok(PropertyValue::Bool(false));
        }
        if self.matches_keyword("accept") {
            self.expect_keyword("accept")?;
            return Ok(PropertyValue::InputPolicy(InputPolicy::Accept));
        }
        if self.matches_keyword("ignore") {
            self.expect_keyword("ignore")?;
            return Ok(PropertyValue::InputPolicy(InputPolicy::Ignore));
        }
        if self.matches_keyword("rgba8") {
            return self.parse_rgba8(anchor);
        }

        let negative = if self.matches_punctuation(PunctuationV1::Minus) {
            self.expect_punctuation(PunctuationV1::Minus)?;
            true
        } else {
            false
        };
        let token = self.take_unsigned()?;
        let magnitude = self.parse_u64_spelled(&token, anchor)?;
        let scalar = if negative {
            if magnitude > i32::MAX as u64 + 1 {
                return Err(
                    self.anchored_failure(AuthoringDiagnosticKindV1::InvalidLiteral, anchor)
                );
            }
            -(magnitude as i64)
        } else {
            i64::try_from(magnitude).map_err(|_| {
                self.anchored_failure(AuthoringDiagnosticKindV1::InvalidLiteral, anchor)
            })?
        };
        i32::try_from(scalar)
            .map(PropertyValue::ScalarI32)
            .map_err(|_| self.anchored_failure(AuthoringDiagnosticKindV1::InvalidLiteral, anchor))
    }

    fn parse_rgba8(&mut self, anchor: u32) -> Result<PropertyValue, AuthoringDiagnosticV1> {
        self.expect_keyword("rgba8")?;
        self.expect_punctuation(PunctuationV1::OpenParenthesis)?;
        let red = self.parse_byte(anchor)?;
        self.expect_punctuation(PunctuationV1::Comma)?;
        let green = self.parse_byte(anchor)?;
        self.expect_punctuation(PunctuationV1::Comma)?;
        let blue = self.parse_byte(anchor)?;
        self.expect_punctuation(PunctuationV1::Comma)?;
        let alpha = self.parse_byte(anchor)?;
        self.expect_punctuation(PunctuationV1::CloseParenthesis)?;
        Ok(PropertyValue::Rgba8([red, green, blue, alpha]))
    }

    fn parse_byte(&mut self, anchor: u32) -> Result<u8, AuthoringDiagnosticV1> {
        let token = self.take_unsigned()?;
        let value = self.parse_u64_spelled(&token, anchor)?;
        u8::try_from(value)
            .map_err(|_| self.anchored_failure(AuthoringDiagnosticKindV1::InvalidLiteral, anchor))
    }

    pub(super) fn parse_invalidation_set(
        &mut self,
    ) -> Result<InvalidationSet, AuthoringDiagnosticV1> {
        self.expect_punctuation(PunctuationV1::OpenBracket)?;
        let mut set = InvalidationSet::from_class(self.parse_invalidation()?);
        while self.matches_punctuation(PunctuationV1::Comma) {
            self.expect_punctuation(PunctuationV1::Comma)?;
            set = set.union(InvalidationSet::from_class(self.parse_invalidation()?));
        }
        self.expect_punctuation(PunctuationV1::CloseBracket)?;
        Ok(set)
    }

    fn parse_invalidation(&mut self) -> Result<InvalidationClass, AuthoringDiagnosticV1> {
        let choices = [
            ("structure", InvalidationClass::Structure),
            ("style_match", InvalidationClass::StyleMatch),
            ("intrinsic", InvalidationClass::Intrinsic),
            ("layout", InvalidationClass::Layout),
            ("semantics", InvalidationClass::Semantics),
            ("hit_test", InvalidationClass::HitTest),
            ("paint", InvalidationClass::Paint),
            ("composition", InvalidationClass::Composition),
            ("surface", InvalidationClass::Surface),
        ];
        for (keyword, class) in choices {
            if self.matches_keyword(keyword) {
                self.expect_keyword(keyword)?;
                return Ok(class);
            }
        }
        Err(self.unexpected())
    }
}
