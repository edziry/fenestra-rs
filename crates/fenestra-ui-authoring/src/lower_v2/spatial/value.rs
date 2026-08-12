use fenestra_ui_ir::prototype::{PropertyId, SpatialBindingV2, SpatialFieldV2, ValueType};

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::AuthoringLimitKindV2;
use crate::parsed_v2::{ParsedBindingV2, ParsedFieldV2, ParsedLiteralV2, ParsedNameFieldV2};
use crate::resolved::logical_span;
use crate::source_v2::{DiagnosticLocationV2, PhysicalOriginV2};

use super::SpatialLowerer;

impl SpatialLowerer<'_> {
    pub(super) fn error(
        &self,
        kind: AuthoringDiagnosticKindV2,
        anchor: u32,
    ) -> AuthoringDiagnosticV2 {
        let parsed_anchor = &self.parsed.anchors[anchor as usize];
        self.error_at(kind, anchor, parsed_anchor.physical)
    }

    pub(super) fn error_at(
        &self,
        kind: AuthoringDiagnosticKindV2,
        anchor: u32,
        physical: PhysicalOriginV2,
    ) -> AuthoringDiagnosticV2 {
        let parsed_anchor = &self.parsed.anchors[anchor as usize];
        AuthoringDiagnosticV2::new(
            self.parsed.frontend,
            kind,
            DiagnosticLocationV2::Anchored {
                logical: logical_span(anchor),
                anchor_kind: parsed_anchor.kind,
                physical,
            },
        )
    }

    pub(super) fn field_value<T>(&self, anchor: u32, value: T) -> SpatialFieldV2<T> {
        self.emitted_fields.set(
            self.emitted_fields
                .get()
                .checked_add(1)
                .expect("spatial field count must remain representable"),
        );
        SpatialFieldV2::new(value, logical_span(anchor))
    }

    pub(super) fn literal_field<T: Copy>(
        &self,
        field: &ParsedFieldV2<ParsedLiteralV2<T>>,
    ) -> Result<SpatialFieldV2<T>, AuthoringDiagnosticV2> {
        let value = self.literal(&field.value, field.anchor)?;
        Ok(self.field_value(field.anchor, value))
    }

    pub(super) fn literal<T: Copy>(
        &self,
        literal: &ParsedLiteralV2<T>,
        anchor: u32,
    ) -> Result<T, AuthoringDiagnosticV2> {
        literal.value.map_err(|physical| {
            self.error_at(AuthoringDiagnosticKindV2::InvalidLiteral, anchor, physical)
        })
    }

    pub(super) fn binding_field<T: Copy>(
        &self,
        field: &ParsedFieldV2<ParsedBindingV2<T>>,
        component: u32,
        expected: ValueType,
    ) -> Result<SpatialFieldV2<SpatialBindingV2<T>>, AuthoringDiagnosticV2> {
        let binding = match &field.value {
            ParsedBindingV2::Literal(literal) => {
                SpatialBindingV2::Literal(self.literal(literal, field.anchor)?)
            }
            ParsedBindingV2::Property(name) => SpatialBindingV2::Property(self.property(
                component,
                name,
                expected,
                field.anchor,
            )?),
        };
        Ok(self.field_value(field.anchor, binding))
    }

    pub(super) fn resolve_node(
        &self,
        field: &ParsedNameFieldV2,
    ) -> Result<SpatialFieldV2<fenestra_ui_ir::prototype::SpatialNodeSymbolV2>, AuthoringDiagnosticV2>
    {
        let index = self.node_symbols.get(&field.value).ok_or_else(|| {
            self.error(
                AuthoringDiagnosticKindV2::UnknownSpatialNodeName,
                field.anchor,
            )
        })?;
        self.node_symbol_field(field, index)
    }

    pub(super) fn resolve_image(
        &self,
        field: &ParsedNameFieldV2,
    ) -> Result<
        SpatialFieldV2<fenestra_ui_ir::prototype::SpatialImageSymbolV2>,
        AuthoringDiagnosticV2,
    > {
        let index = self.image_symbols.get(&field.value).ok_or_else(|| {
            self.error(
                AuthoringDiagnosticKindV2::UnknownSpatialImageName,
                field.anchor,
            )
        })?;
        self.image_symbol_field(field, index)
    }

    pub(super) fn dense_symbol(
        &self,
        index: usize,
        limit: AuthoringLimitKindV2,
        anchor: u32,
    ) -> Result<u32, AuthoringDiagnosticV2> {
        u32::try_from(index)
            .map_err(|_| self.error(AuthoringDiagnosticKindV2::LimitExceeded(limit), anchor))
    }

    fn property(
        &self,
        component: u32,
        name: &str,
        expected: ValueType,
        anchor: u32,
    ) -> Result<PropertyId, AuthoringDiagnosticV2> {
        let component = self
            .core
            .schema
            .components
            .iter()
            .find(|candidate| candidate.id == component)
            .expect("resolved templates must reference a resolved component");
        let property = component
            .properties
            .iter()
            .find(|property| property.name.as_ref() == name)
            .ok_or_else(|| self.error(AuthoringDiagnosticKindV2::UnknownPropertyName, anchor))?;
        if property.value_type != expected {
            return Err(self.error(AuthoringDiagnosticKindV2::ValueTypeMismatch, anchor));
        }
        Ok(PropertyId::new(property.id))
    }
}
