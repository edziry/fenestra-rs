use fenestra_ui_ir::prototype::{
    InputPolicy, InvalidationClass, InvalidationSet, PropertyValue, ValueType,
};

use crate::diagnostic::AuthoringDiagnosticV1;
use crate::parsed::ParsedLiteralV1;
use crate::token::PunctuationV1;

use super::{ParserV1, SpelledTokenV1};

impl ParserV1 {
    pub(super) fn parse_u32(&mut self) -> Result<ParsedLiteralV1<u32>, AuthoringDiagnosticV1> {
        let token = self.take_unsigned()?;
        let value = self
            .parse_u64_spelled(&token)
            .value
            .and_then(|value| u32::try_from(value).map_err(|_| token.physical));
        Ok(ParsedLiteralV1 {
            value,
            physical: token.physical,
        })
    }

    pub(super) fn parse_u64(&mut self) -> Result<ParsedLiteralV1<u64>, AuthoringDiagnosticV1> {
        let token = self.take_unsigned()?;
        Ok(self.parse_u64_spelled(&token))
    }

    pub(super) fn parse_u64_spelled(&self, token: &SpelledTokenV1) -> ParsedLiteralV1<u64> {
        let physical = token.physical;
        let value = if token.text.len() > 1 && token.text.starts_with('0') {
            Err(physical)
        } else {
            token.text.parse::<u64>().map_err(|_| physical)
        };
        ParsedLiteralV1 { value, physical }
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
    ) -> Result<ParsedLiteralV1<PropertyValue>, AuthoringDiagnosticV1> {
        if self.matches_keyword("true") {
            let token = self.expect_keyword("true")?;
            return Ok(ParsedLiteralV1 {
                value: Ok(PropertyValue::Bool(true)),
                physical: token.physical,
            });
        }
        if self.matches_keyword("false") {
            let token = self.expect_keyword("false")?;
            return Ok(ParsedLiteralV1 {
                value: Ok(PropertyValue::Bool(false)),
                physical: token.physical,
            });
        }
        if self.matches_keyword("accept") {
            let token = self.expect_keyword("accept")?;
            return Ok(ParsedLiteralV1 {
                value: Ok(PropertyValue::InputPolicy(InputPolicy::Accept)),
                physical: token.physical,
            });
        }
        if self.matches_keyword("ignore") {
            let token = self.expect_keyword("ignore")?;
            return Ok(ParsedLiteralV1 {
                value: Ok(PropertyValue::InputPolicy(InputPolicy::Ignore)),
                physical: token.physical,
            });
        }
        if self.matches_keyword("rgba8") {
            return self.parse_rgba8();
        }

        let minus = if self.matches_punctuation(PunctuationV1::Minus) {
            Some(self.expect_punctuation(PunctuationV1::Minus)?)
        } else {
            None
        };
        let token = self.take_unsigned()?;
        let magnitude = self.parse_u64_spelled(&token).value;
        let physical = token.physical;
        let value = magnitude.and_then(|magnitude| {
            let scalar = if minus.is_some() {
                if magnitude > i32::MAX as u64 + 1 {
                    return Err(physical);
                }
                -(magnitude as i64)
            } else {
                i64::try_from(magnitude).map_err(|_| physical)?
            };
            i32::try_from(scalar)
                .map(PropertyValue::ScalarI32)
                .map_err(|_| physical)
        });
        let representative = minus.map_or(token.physical, |minus| minus.physical);
        Ok(ParsedLiteralV1 {
            value,
            physical: representative,
        })
    }

    fn parse_rgba8(&mut self) -> Result<ParsedLiteralV1<PropertyValue>, AuthoringDiagnosticV1> {
        let opening = self.expect_keyword("rgba8")?;
        self.expect_punctuation(PunctuationV1::OpenParenthesis)?;
        let red = self.parse_byte()?;
        self.expect_punctuation(PunctuationV1::Comma)?;
        let green = self.parse_byte()?;
        self.expect_punctuation(PunctuationV1::Comma)?;
        let blue = self.parse_byte()?;
        self.expect_punctuation(PunctuationV1::Comma)?;
        let alpha = self.parse_byte()?;
        self.expect_punctuation(PunctuationV1::CloseParenthesis)?;
        let value = red.and_then(|red| {
            green.and_then(|green| {
                blue.and_then(|blue| {
                    alpha.map(|alpha| PropertyValue::Rgba8([red, green, blue, alpha]))
                })
            })
        });
        Ok(ParsedLiteralV1 {
            value,
            physical: opening.physical,
        })
    }

    fn parse_byte(
        &mut self,
    ) -> Result<Result<u8, crate::source::PhysicalOriginV1>, AuthoringDiagnosticV1> {
        let token = self.take_unsigned()?;
        Ok(self
            .parse_u64_spelled(&token)
            .value
            .and_then(|value| u8::try_from(value).map_err(|_| token.physical)))
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
