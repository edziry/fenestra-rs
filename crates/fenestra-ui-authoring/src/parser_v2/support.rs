use fenestra_ui_ir::prototype::{SourceId, SourceSpan};

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::AuthoringLimitKindV2;
use crate::parsed_v2::{ParsedAnchorV2, ParsedFieldV2, SpannedV2};
use crate::source_v2::{DiagnosticLocationV2, PhysicalOriginV2};
use crate::token::{AbstractToken, AbstractTokenKind, Punctuation};
use crate::vocabulary_v2::AnchorKindV2;

use super::{ParserV2, RecordCountV2, SpelledTokenV2};

impl ParserV2 {
    pub(super) fn parse_name(&mut self) -> Result<SpelledTokenV2, AuthoringDiagnosticV2> {
        let token = self.take()?;
        let AbstractTokenKind::Identifier(text) = &token.kind else {
            return Err(self.failure_at(AuthoringDiagnosticKindV2::UnexpectedToken, &token));
        };
        if is_reserved(text) {
            return Err(self.failure_at(AuthoringDiagnosticKindV2::InvalidIdentifier, &token));
        }
        Ok(SpelledTokenV2 {
            text: text.clone(),
            physical: token.physical,
        })
    }

    pub(super) fn expect_keyword(
        &mut self,
        expected: &str,
    ) -> Result<AbstractToken<PhysicalOriginV2>, AuthoringDiagnosticV2> {
        let token = self.take()?;
        if matches!(&token.kind, AbstractTokenKind::Identifier(actual) if &**actual == expected) {
            Ok(token)
        } else {
            Err(self.failure_at(AuthoringDiagnosticKindV2::UnexpectedToken, &token))
        }
    }

    pub(super) fn expect_punctuation(
        &mut self,
        expected: Punctuation,
    ) -> Result<AbstractToken<PhysicalOriginV2>, AuthoringDiagnosticV2> {
        let token = self.take()?;
        if token.kind == AbstractTokenKind::Punctuation(expected) {
            Ok(token)
        } else {
            Err(self.failure_at(AuthoringDiagnosticKindV2::UnexpectedToken, &token))
        }
    }

    pub(super) fn matches_keyword(&self, expected: &str) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(AbstractTokenKind::Identifier(actual)) if &**actual == expected
        )
    }

    pub(super) fn matches_punctuation(&self, expected: Punctuation) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(AbstractTokenKind::Punctuation(actual)) if *actual == expected
        )
    }

    pub(super) fn take_unsigned(&mut self) -> Result<SpelledTokenV2, AuthoringDiagnosticV2> {
        let token = self.take()?;
        let AbstractTokenKind::UnsignedDecimal(text) = &token.kind else {
            return Err(self.failure_at(AuthoringDiagnosticKindV2::UnexpectedToken, &token));
        };
        Ok(SpelledTokenV2 {
            text: text.clone(),
            physical: token.physical,
        })
    }

    pub(super) fn push_anchor(
        &mut self,
        kind: AnchorKindV2,
        token: &AbstractToken<PhysicalOriginV2>,
    ) -> Result<u32, AuthoringDiagnosticV2> {
        self.push_anchor_parts(kind, token.label(), token.physical)
    }

    pub(super) fn push_spelled_anchor(
        &mut self,
        kind: AnchorKindV2,
        token: &SpelledTokenV2,
    ) -> Result<u32, AuthoringDiagnosticV2> {
        self.push_anchor_parts(kind, &token.text, token.physical)
    }

    pub(super) fn push_field_spelled<T>(
        &mut self,
        value: T,
        token: &SpelledTokenV2,
    ) -> Result<ParsedFieldV2<T>, AuthoringDiagnosticV2> {
        self.push_field_parts(value, &token.text, token.physical)
    }

    pub(super) fn push_field_parts<T>(
        &mut self,
        value: T,
        label: &str,
        physical: PhysicalOriginV2,
    ) -> Result<ParsedFieldV2<T>, AuthoringDiagnosticV2> {
        self.claim_record(RecordCountV2::SpatialFields, physical)?;
        let anchor = self.push_anchor_parts(AnchorKindV2::SpatialField, label, physical)?;
        Ok(ParsedFieldV2 { value, anchor })
    }

    pub(super) fn spanned_name(&self, token: SpelledTokenV2) -> SpannedV2<Box<str>> {
        SpannedV2 {
            physical: token.physical,
            value: token.text,
        }
    }

    fn push_anchor_parts(
        &mut self,
        kind: AnchorKindV2,
        label: &str,
        physical: PhysicalOriginV2,
    ) -> Result<u32, AuthoringDiagnosticV2> {
        let maximum = self
            .limits
            .limit(AuthoringLimitKindV2::SourceAnchors)
            .min(u32::MAX as usize);
        if self.anchors.len() >= maximum {
            return Err(self.physical_failure(
                AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::SourceAnchors),
                physical,
            ));
        }
        let ordinal = u32::try_from(self.anchors.len()).map_err(|_| {
            self.physical_failure(
                AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::SourceAnchors),
                physical,
            )
        })?;
        self.anchors.push(ParsedAnchorV2 {
            kind,
            label: label.into(),
            physical,
        });
        Ok(ordinal)
    }

    pub(super) fn claim_record(
        &mut self,
        record: RecordCountV2,
        physical: PhysicalOriginV2,
    ) -> Result<(), AuthoringDiagnosticV2> {
        let kind = record.limit_kind();
        let index = record as usize;
        let next = self.record_counts[index].checked_add(1).ok_or_else(|| {
            self.physical_failure(AuthoringDiagnosticKindV2::LimitExceeded(kind), physical)
        })?;
        if next > self.limits.limit(kind) {
            return Err(
                self.physical_failure(AuthoringDiagnosticKindV2::LimitExceeded(kind), physical)
            );
        }
        self.record_counts[index] = next;
        Ok(())
    }

    pub(super) fn claim_image_byte(
        &mut self,
        image_anchor: u32,
        physical: PhysicalOriginV2,
    ) -> Result<(), AuthoringDiagnosticV2> {
        let record = RecordCountV2::ImageBytes;
        let kind = record.limit_kind();
        let index = record as usize;
        let next = self.record_counts[index].checked_add(1).ok_or_else(|| {
            self.anchored_failure_at(
                AuthoringDiagnosticKindV2::LimitExceeded(kind),
                image_anchor,
                physical,
            )
        })?;
        if next > self.limits.limit(kind) {
            return Err(self.anchored_failure_at(
                AuthoringDiagnosticKindV2::LimitExceeded(kind),
                image_anchor,
                physical,
            ));
        }
        self.record_counts[index] = next;
        Ok(())
    }

    pub(super) fn record_count(&self, record: RecordCountV2) -> usize {
        self.record_counts[record as usize]
    }

    pub(super) fn anchored_failure(
        &self,
        kind: AuthoringDiagnosticKindV2,
        ordinal: u32,
    ) -> AuthoringDiagnosticV2 {
        self.anchored_failure_at(kind, ordinal, self.anchors[ordinal as usize].physical)
    }

    pub(super) fn anchored_failure_at(
        &self,
        kind: AuthoringDiagnosticKindV2,
        ordinal: u32,
        physical: PhysicalOriginV2,
    ) -> AuthoringDiagnosticV2 {
        let anchor = &self.anchors[ordinal as usize];
        AuthoringDiagnosticV2::new(
            self.frontend,
            kind,
            DiagnosticLocationV2::Anchored {
                logical: SourceSpan::bytes(SourceId::new(0), ordinal, ordinal + 1),
                anchor_kind: anchor.kind,
                physical,
            },
        )
    }

    pub(super) fn failure_at(
        &self,
        kind: AuthoringDiagnosticKindV2,
        token: &AbstractToken<PhysicalOriginV2>,
    ) -> AuthoringDiagnosticV2 {
        self.physical_failure(kind, token.physical)
    }

    pub(super) fn unexpected(&self) -> AuthoringDiagnosticV2 {
        self.peek().map_or_else(
            || self.physical_failure(AuthoringDiagnosticKindV2::UnexpectedEof, self.eof),
            |token| self.failure_at(AuthoringDiagnosticKindV2::UnexpectedToken, token),
        )
    }

    pub(super) fn physical_failure(
        &self,
        kind: AuthoringDiagnosticKindV2,
        physical: PhysicalOriginV2,
    ) -> AuthoringDiagnosticV2 {
        AuthoringDiagnosticV2::new(
            self.frontend,
            kind,
            DiagnosticLocationV2::Physical(physical),
        )
    }

    pub(super) fn expect_eof(&self) -> Result<(), AuthoringDiagnosticV2> {
        self.peek().map_or(Ok(()), |_| Err(self.unexpected()))
    }

    fn peek(&self) -> Option<&AbstractToken<PhysicalOriginV2>> {
        self.tokens.get(self.next)
    }

    fn take(&mut self) -> Result<AbstractToken<PhysicalOriginV2>, AuthoringDiagnosticV2> {
        let token = self.peek().cloned().ok_or_else(|| self.unexpected())?;
        self.next += 1;
        Ok(token)
    }
}

fn is_reserved(identifier: &str) -> bool {
    V2_RESERVED_WORDS.contains(&identifier)
}

const V2_RESERVED_WORDS: &[&str] = &[
    "format",
    "schema",
    "namespace",
    "revision",
    "component",
    "property",
    "invalidates",
    "construction",
    "template",
    "set",
    "child",
    "region",
    "owner",
    "repeat",
    "keys",
    "style",
    "bool",
    "scalar_i32",
    "rgba8",
    "input_policy",
    "true",
    "false",
    "accept",
    "ignore",
    "structure",
    "style_match",
    "intrinsic",
    "layout",
    "semantics",
    "hit_test",
    "paint",
    "composition",
    "surface",
    "spatial",
    "viewport",
    "container",
    "row",
    "column",
    "padding",
    "gap",
    "resources",
    "image",
    "width",
    "height",
    "stride",
    "bytes",
    "node",
    "placement",
    "free",
    "dimension",
    "self_anchor",
    "anchor",
    "start",
    "center",
    "end",
    "target",
    "parent",
    "target_anchor",
    "offset",
    "transform",
    "identity",
    "translate",
    "scale",
    "quarter_turn",
    "affine",
    "origin",
    "point",
    "fixed",
    "shape",
    "rect",
    "circle",
    "polygon",
    "path",
    "radius",
    "move_to",
    "line_to",
    "quadratic_to",
    "cubic_to",
    "close",
    "brush",
    "solid",
    "color",
    "linear_gradient",
    "stop",
    "clip",
    "none",
    "fill_rule",
    "non_zero",
    "even_odd",
    "coverage",
    "fill",
    "rule",
    "round_stroke",
    "opacity",
    "source",
    "destination",
    "hit",
    "input",
    "semantic",
];
