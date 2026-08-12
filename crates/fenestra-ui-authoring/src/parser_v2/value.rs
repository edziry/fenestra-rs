use fenestra_ui_ir::prototype::{
    InputPolicy, InvalidationClass, InvalidationSet, PropertyValue, ValueType,
};

use crate::diagnostic_v2::AuthoringDiagnosticV2;
use crate::parsed_v2::ParsedLiteralV2;
use crate::source_v2::PhysicalOriginV2;
use crate::token::Punctuation;

use super::{ParserV2, SpelledTokenV2};

pub(super) struct ParsedSpellingV2<T> {
    pub(super) literal: ParsedLiteralV2<T>,
    pub(super) label: Box<str>,
}

impl ParserV2 {
    pub(super) fn parse_u32(&mut self) -> Result<ParsedLiteralV2<u32>, AuthoringDiagnosticV2> {
        let token = self.take_unsigned()?;
        Ok(self.unsigned_spelling::<u32>(&token).literal)
    }

    pub(super) fn parse_u64(&mut self) -> Result<ParsedLiteralV2<u64>, AuthoringDiagnosticV2> {
        let token = self.take_unsigned()?;
        Ok(self.parse_u64_spelled(&token))
    }

    pub(super) fn parse_u64_spelled(&self, token: &SpelledTokenV2) -> ParsedLiteralV2<u64> {
        let physical = token.physical;
        let value = if token.text.len() > 1 && token.text.starts_with('0') {
            Err(physical)
        } else {
            token.text.parse::<u64>().map_err(|_| physical)
        };
        ParsedLiteralV2 { value, physical }
    }

    pub(super) fn unsigned_spelling<T>(&self, token: &SpelledTokenV2) -> ParsedSpellingV2<T>
    where
        T: TryFrom<u64> + ToString + Copy,
    {
        let physical = token.physical;
        let value = self
            .parse_u64_spelled(token)
            .value
            .and_then(|value| T::try_from(value).map_err(|_| physical));
        let label = value
            .as_ref()
            .map_or_else(|_| token.text.clone(), |value| value.to_string().into());
        ParsedSpellingV2 {
            literal: ParsedLiteralV2 { value, physical },
            label,
        }
    }

    pub(super) fn parse_signed_i32_spelling(
        &mut self,
    ) -> Result<ParsedSpellingV2<i32>, AuthoringDiagnosticV2> {
        let (negative, token, physical) = self.take_signed()?;
        let magnitude = self.parse_u64_spelled(&token).value.map_err(|_| physical);
        let value = magnitude.and_then(|magnitude| {
            let signed = signed_magnitude(magnitude, negative, i32::MAX as u64, physical)?;
            i32::try_from(signed).map_err(|_| physical)
        });
        let label = value.as_ref().map_or_else(
            |_| raw_signed_label(negative, &token.text),
            |value| value.to_string().into(),
        );
        Ok(ParsedSpellingV2 {
            literal: ParsedLiteralV2 { value, physical },
            label,
        })
    }

    pub(super) fn parse_signed_i64_spelling(
        &mut self,
    ) -> Result<ParsedSpellingV2<i64>, AuthoringDiagnosticV2> {
        let (negative, token, physical) = self.take_signed()?;
        let magnitude = self.parse_u64_spelled(&token).value.map_err(|_| physical);
        let value = magnitude
            .and_then(|magnitude| signed_magnitude(magnitude, negative, i64::MAX as u64, physical));
        let label = value.as_ref().map_or_else(
            |_| raw_signed_label(negative, &token.text),
            |value| value.to_string().into(),
        );
        Ok(ParsedSpellingV2 {
            literal: ParsedLiteralV2 { value, physical },
            label,
        })
    }

    fn take_signed(
        &mut self,
    ) -> Result<(bool, SpelledTokenV2, PhysicalOriginV2), AuthoringDiagnosticV2> {
        let minus = if self.matches_punctuation(Punctuation::Minus) {
            Some(self.expect_punctuation(Punctuation::Minus)?)
        } else {
            None
        };
        let token = self.take_unsigned()?;
        let physical = minus.as_ref().map_or(token.physical, |minus| {
            cover_signed_origin(minus.physical, token.physical)
        });
        Ok((minus.is_some(), token, physical))
    }

    pub(super) fn parse_value_type(&mut self) -> Result<ValueType, AuthoringDiagnosticV2> {
        let choices = [
            ("bool", ValueType::Bool),
            ("scalar_i32", ValueType::ScalarI32),
            ("rgba8", ValueType::Rgba8),
            ("input_policy", ValueType::InputPolicy),
        ];
        for (keyword, value_type) in choices {
            if self.matches_keyword(keyword) {
                self.expect_keyword(keyword)?;
                return Ok(value_type);
            }
        }
        Err(self.unexpected())
    }

    pub(super) fn parse_value(
        &mut self,
    ) -> Result<ParsedLiteralV2<PropertyValue>, AuthoringDiagnosticV2> {
        for (keyword, value) in [
            ("true", PropertyValue::Bool(true)),
            ("false", PropertyValue::Bool(false)),
            ("accept", PropertyValue::InputPolicy(InputPolicy::Accept)),
            ("ignore", PropertyValue::InputPolicy(InputPolicy::Ignore)),
        ] {
            if self.matches_keyword(keyword) {
                let token = self.expect_keyword(keyword)?;
                return Ok(ParsedLiteralV2 {
                    value: Ok(value),
                    physical: token.physical,
                });
            }
        }
        if self.matches_keyword("rgba8") {
            let parsed = self.parse_rgba8_spelling()?;
            return Ok(ParsedLiteralV2 {
                value: parsed.literal.value.map(PropertyValue::Rgba8),
                physical: parsed.literal.physical,
            });
        }
        let parsed = self.parse_signed_i32_spelling()?;
        Ok(ParsedLiteralV2 {
            value: parsed.literal.value.map(PropertyValue::ScalarI32),
            physical: parsed.literal.physical,
        })
    }

    pub(super) fn parse_rgba8_spelling(
        &mut self,
    ) -> Result<ParsedSpellingV2<[u8; 4]>, AuthoringDiagnosticV2> {
        let opening = self.expect_keyword("rgba8")?;
        self.expect_punctuation(Punctuation::OpenParenthesis)?;
        let red = self.parse_unsigned_spelling::<u8>()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let green = self.parse_unsigned_spelling::<u8>()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let blue = self.parse_unsigned_spelling::<u8>()?;
        self.expect_punctuation(Punctuation::Comma)?;
        let alpha = self.parse_unsigned_spelling::<u8>()?;
        self.expect_punctuation(Punctuation::CloseParenthesis)?;
        let labels = [&*red.label, &*green.label, &*blue.label, &*alpha.label];
        let value = red.literal.value.and_then(|red| {
            green.literal.value.and_then(|green| {
                blue.literal
                    .value
                    .and_then(|blue| alpha.literal.value.map(|alpha| [red, green, blue, alpha]))
            })
        });
        Ok(ParsedSpellingV2 {
            literal: ParsedLiteralV2 {
                value,
                physical: opening.physical,
            },
            label: format!(
                "rgba8({},{},{},{})",
                labels[0], labels[1], labels[2], labels[3]
            )
            .into(),
        })
    }

    pub(super) fn parse_unsigned_spelling<T>(
        &mut self,
    ) -> Result<ParsedSpellingV2<T>, AuthoringDiagnosticV2>
    where
        T: TryFrom<u64> + ToString + Copy,
    {
        let token = self.take_unsigned()?;
        Ok(self.unsigned_spelling(&token))
    }

    pub(super) fn parse_invalidation_set(
        &mut self,
    ) -> Result<InvalidationSet, AuthoringDiagnosticV2> {
        self.expect_punctuation(Punctuation::OpenBracket)?;
        let mut set = InvalidationSet::from_class(self.parse_invalidation()?);
        while self.matches_punctuation(Punctuation::Comma) {
            self.expect_punctuation(Punctuation::Comma)?;
            set = set.union(InvalidationSet::from_class(self.parse_invalidation()?));
        }
        self.expect_punctuation(Punctuation::CloseBracket)?;
        Ok(set)
    }

    fn parse_invalidation(&mut self) -> Result<InvalidationClass, AuthoringDiagnosticV2> {
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

fn signed_magnitude(
    magnitude: u64,
    negative: bool,
    positive_maximum: u64,
    physical: PhysicalOriginV2,
) -> Result<i64, PhysicalOriginV2> {
    if negative {
        if magnitude > positive_maximum + 1 {
            return Err(physical);
        }
        if magnitude == i64::MAX as u64 + 1 {
            return Ok(i64::MIN);
        }
        Ok(-(magnitude as i64))
    } else {
        if magnitude > positive_maximum {
            return Err(physical);
        }
        Ok(magnitude as i64)
    }
}

fn raw_signed_label(negative: bool, magnitude: &str) -> Box<str> {
    if negative {
        format!("-{magnitude}").into()
    } else {
        magnitude.into()
    }
}

fn cover_signed_origin(minus: PhysicalOriginV2, magnitude: PhysicalOriginV2) -> PhysicalOriginV2 {
    match (
        minus.source_id(),
        minus.fen_byte_range(),
        magnitude.source_id(),
        magnitude.fen_byte_range(),
    ) {
        (Some(left_source), Some((start, _)), Some(right_source), Some((_, end)))
            if left_source == right_source =>
        {
            PhysicalOriginV2::fen_bytes(left_source, start, end)
        }
        _ => minus,
    }
}
