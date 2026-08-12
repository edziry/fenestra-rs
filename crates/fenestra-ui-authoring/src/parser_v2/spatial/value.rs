use fenestra_ui_ir::prototype::InputPolicy;

use crate::diagnostic_v2::AuthoringDiagnosticV2;
use crate::parsed_v2::{
    ParsedBindingV2, ParsedColorBindingFieldV2, ParsedFixedBindingFieldV2, ParsedI32BindingFieldV2,
    ParsedI32FieldV2, ParsedInputBindingFieldV2, ParsedLiteralV2, ParsedNameFieldV2,
    ParsedU8FieldV2, ParsedU16FieldV2, ParsedU32FieldV2,
};
use crate::source_v2::PhysicalOriginV2;
use crate::token::Punctuation;

use super::super::ParserV2;

impl ParserV2 {
    pub(super) fn parse_name_field(&mut self) -> Result<ParsedNameFieldV2, AuthoringDiagnosticV2> {
        let name = self.parse_name()?;
        self.push_field_spelled(name.text.clone(), &name)
    }

    pub(super) fn parse_i32_field(&mut self) -> Result<ParsedI32FieldV2, AuthoringDiagnosticV2> {
        let parsed = self.parse_signed_i32_spelling()?;
        let physical = parsed.literal.physical;
        self.push_field_parts(parsed.literal, &parsed.label, physical)
    }

    pub(super) fn parse_u8_field(&mut self) -> Result<ParsedU8FieldV2, AuthoringDiagnosticV2> {
        let parsed = self.parse_unsigned_spelling::<u8>()?;
        let physical = parsed.literal.physical;
        self.push_field_parts(parsed.literal, &parsed.label, physical)
    }

    pub(super) fn parse_u16_field(&mut self) -> Result<ParsedU16FieldV2, AuthoringDiagnosticV2> {
        let parsed = self.parse_unsigned_spelling::<u16>()?;
        let physical = parsed.literal.physical;
        self.push_field_parts(parsed.literal, &parsed.label, physical)
    }

    pub(super) fn parse_u32_field(&mut self) -> Result<ParsedU32FieldV2, AuthoringDiagnosticV2> {
        let parsed = self.parse_unsigned_spelling::<u32>()?;
        let physical = parsed.literal.physical;
        self.push_field_parts(parsed.literal, &parsed.label, physical)
    }

    pub(super) fn parse_i32_binding_field(
        &mut self,
    ) -> Result<ParsedI32BindingFieldV2, AuthoringDiagnosticV2> {
        if self.matches_keyword("property") {
            self.expect_keyword("property")?;
            let property = self.parse_name()?;
            return self
                .push_field_spelled(ParsedBindingV2::Property(property.text.clone()), &property);
        }
        let parsed = self.parse_signed_i32_spelling()?;
        let physical = parsed.literal.physical;
        self.push_field_parts(
            ParsedBindingV2::Literal(parsed.literal),
            &parsed.label,
            physical,
        )
    }

    pub(super) fn parse_fixed_binding_field(
        &mut self,
    ) -> Result<ParsedFixedBindingFieldV2, AuthoringDiagnosticV2> {
        if self.matches_keyword("property") {
            self.expect_keyword("property")?;
            let property = self.parse_name()?;
            return self
                .push_field_spelled(ParsedBindingV2::Property(property.text.clone()), &property);
        }
        self.expect_keyword("fixed")?;
        self.expect_punctuation(Punctuation::OpenParenthesis)?;
        let parsed = self.parse_signed_i64_spelling()?;
        self.expect_punctuation(Punctuation::CloseParenthesis)?;
        let physical = parsed.literal.physical;
        self.push_field_parts(
            ParsedBindingV2::Literal(parsed.literal),
            &parsed.label,
            physical,
        )
    }

    pub(super) fn parse_color_binding_field(
        &mut self,
    ) -> Result<ParsedColorBindingFieldV2, AuthoringDiagnosticV2> {
        if self.matches_keyword("property") {
            self.expect_keyword("property")?;
            let property = self.parse_name()?;
            return self
                .push_field_spelled(ParsedBindingV2::Property(property.text.clone()), &property);
        }
        let parsed = self.parse_rgba8_spelling()?;
        let physical = parsed.literal.physical;
        self.push_field_parts(
            ParsedBindingV2::Literal(parsed.literal),
            &parsed.label,
            physical,
        )
    }

    pub(super) fn parse_input_binding_field(
        &mut self,
    ) -> Result<ParsedInputBindingFieldV2, AuthoringDiagnosticV2> {
        if self.matches_keyword("property") {
            self.expect_keyword("property")?;
            let property = self.parse_name()?;
            return self
                .push_field_spelled(ParsedBindingV2::Property(property.text.clone()), &property);
        }
        let (token, value) = if self.matches_keyword("accept") {
            (self.expect_keyword("accept")?, InputPolicy::Accept)
        } else if self.matches_keyword("ignore") {
            (self.expect_keyword("ignore")?, InputPolicy::Ignore)
        } else {
            return Err(self.unexpected());
        };
        self.push_field_parts(
            ParsedBindingV2::Literal(ParsedLiteralV2 {
                value: Ok(value),
                physical: token.physical,
            }),
            token.label(),
            token.physical,
        )
    }

    pub(super) fn constant_fixed_field(
        &mut self,
        value: i64,
        label: &str,
        physical: PhysicalOriginV2,
    ) -> Result<ParsedFixedBindingFieldV2, AuthoringDiagnosticV2> {
        self.push_field_parts(
            ParsedBindingV2::Literal(ParsedLiteralV2 {
                value: Ok(value),
                physical,
            }),
            label,
            physical,
        )
    }
}
