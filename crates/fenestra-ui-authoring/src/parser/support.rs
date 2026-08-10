use fenestra_ui_ir::prototype::{SourceId, SourceSpan};

use crate::diagnostic::{AuthoringDiagnosticKindV1, AuthoringDiagnosticV1};
use crate::limits::AuthoringLimitKindV1;
use crate::parsed::{ParsedAnchorV1, SpannedV1};
use crate::source::{DiagnosticLocationV1, PhysicalOriginV1};
use crate::token::{AbstractTokenKindV1, AbstractTokenV1, PunctuationV1};
use crate::vocabulary::AnchorKindV1;

use super::{ParserV1, RecordCountV1, SpelledTokenV1};

impl ParserV1 {
    pub(super) fn parse_name(&mut self) -> Result<SpelledTokenV1, AuthoringDiagnosticV1> {
        let token = self.take()?;
        let AbstractTokenKindV1::Identifier(text) = &token.kind else {
            return Err(self.failure_at(AuthoringDiagnosticKindV1::UnexpectedToken, &token));
        };
        if is_reserved(text) {
            return Err(self.failure_at(AuthoringDiagnosticKindV1::InvalidIdentifier, &token));
        }
        Ok(SpelledTokenV1 {
            text: text.clone(),
            physical: token.physical,
        })
    }

    pub(super) fn expect_keyword(
        &mut self,
        expected: &str,
    ) -> Result<AbstractTokenV1, AuthoringDiagnosticV1> {
        let token = self.take()?;
        if matches!(&token.kind, AbstractTokenKindV1::Identifier(actual) if &**actual == expected) {
            Ok(token)
        } else {
            Err(self.failure_at(AuthoringDiagnosticKindV1::UnexpectedToken, &token))
        }
    }

    pub(super) fn expect_punctuation(
        &mut self,
        expected: PunctuationV1,
    ) -> Result<AbstractTokenV1, AuthoringDiagnosticV1> {
        let token = self.take()?;
        if token.kind == AbstractTokenKindV1::Punctuation(expected) {
            Ok(token)
        } else {
            Err(self.failure_at(AuthoringDiagnosticKindV1::UnexpectedToken, &token))
        }
    }

    pub(super) fn matches_keyword(&self, expected: &str) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(AbstractTokenKindV1::Identifier(actual)) if &**actual == expected
        )
    }

    pub(super) fn matches_punctuation(&self, expected: PunctuationV1) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(AbstractTokenKindV1::Punctuation(actual)) if *actual == expected
        )
    }

    pub(super) fn take_unsigned(&mut self) -> Result<SpelledTokenV1, AuthoringDiagnosticV1> {
        let token = self.take()?;
        let AbstractTokenKindV1::UnsignedDecimal(text) = &token.kind else {
            return Err(self.failure_at(AuthoringDiagnosticKindV1::UnexpectedToken, &token));
        };
        Ok(SpelledTokenV1 {
            text: text.clone(),
            physical: token.physical,
        })
    }

    pub(super) fn push_anchor(
        &mut self,
        kind: AnchorKindV1,
        token: &AbstractTokenV1,
    ) -> Result<u32, AuthoringDiagnosticV1> {
        self.push_anchor_parts(kind, token.label(), token.physical)
    }

    pub(super) fn push_spelled_anchor(
        &mut self,
        kind: AnchorKindV1,
        token: &SpelledTokenV1,
    ) -> Result<u32, AuthoringDiagnosticV1> {
        self.push_anchor_parts(kind, &token.text, token.physical)
    }

    pub(super) fn spanned_name(&self, token: SpelledTokenV1) -> SpannedV1<Box<str>> {
        SpannedV1 {
            physical: token.physical,
            value: token.text,
        }
    }

    fn push_anchor_parts(
        &mut self,
        kind: AnchorKindV1,
        label: &str,
        physical: PhysicalOriginV1,
    ) -> Result<u32, AuthoringDiagnosticV1> {
        if self.anchors.len() >= self.limits.limit(AuthoringLimitKindV1::SourceAnchors) {
            return Err(self.physical_failure(
                AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::SourceAnchors),
                physical,
            ));
        }
        let ordinal = u32::try_from(self.anchors.len()).map_err(|_| {
            self.physical_failure(
                AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::SourceAnchors),
                physical,
            )
        })?;
        self.anchors.push(ParsedAnchorV1 {
            kind,
            label: label.into(),
            physical,
        });
        Ok(ordinal)
    }

    pub(super) fn claim_record(
        &mut self,
        record: RecordCountV1,
        opening: &AbstractTokenV1,
    ) -> Result<(), AuthoringDiagnosticV1> {
        self.claim_record_origin(record, opening.physical)
    }

    pub(super) fn claim_spelled_record(
        &mut self,
        record: RecordCountV1,
        opening: &SpelledTokenV1,
    ) -> Result<(), AuthoringDiagnosticV1> {
        self.claim_record_origin(record, opening.physical)
    }

    fn claim_record_origin(
        &mut self,
        record: RecordCountV1,
        physical: PhysicalOriginV1,
    ) -> Result<(), AuthoringDiagnosticV1> {
        let kind = record.limit_kind();
        let index = record as usize;
        if self.record_counts[index] >= self.limits.limit(kind) {
            return Err(
                self.physical_failure(AuthoringDiagnosticKindV1::LimitExceeded(kind), physical)
            );
        }
        self.record_counts[index] += 1;
        Ok(())
    }

    pub(super) fn anchored_failure(
        &self,
        kind: AuthoringDiagnosticKindV1,
        ordinal: u32,
    ) -> AuthoringDiagnosticV1 {
        let anchor = &self.anchors[ordinal as usize];
        self.anchored_failure_at(kind, ordinal, anchor.physical)
    }

    pub(super) fn anchored_failure_at(
        &self,
        kind: AuthoringDiagnosticKindV1,
        ordinal: u32,
        physical: PhysicalOriginV1,
    ) -> AuthoringDiagnosticV1 {
        let anchor = &self.anchors[ordinal as usize];
        AuthoringDiagnosticV1::new(
            self.frontend,
            kind,
            DiagnosticLocationV1::Anchored {
                logical: SourceSpan::bytes(SourceId::new(0), ordinal, ordinal + 1),
                anchor_kind: anchor.kind,
                physical,
            },
        )
    }

    pub(super) fn failure_at(
        &self,
        kind: AuthoringDiagnosticKindV1,
        token: &AbstractTokenV1,
    ) -> AuthoringDiagnosticV1 {
        self.physical_failure(kind, token.physical)
    }

    pub(super) fn unexpected(&self) -> AuthoringDiagnosticV1 {
        self.peek().map_or_else(
            || self.physical_failure(AuthoringDiagnosticKindV1::UnexpectedEof, self.eof),
            |token| self.failure_at(AuthoringDiagnosticKindV1::UnexpectedToken, token),
        )
    }

    fn physical_failure(
        &self,
        kind: AuthoringDiagnosticKindV1,
        physical: PhysicalOriginV1,
    ) -> AuthoringDiagnosticV1 {
        AuthoringDiagnosticV1::new(
            self.frontend,
            kind,
            DiagnosticLocationV1::Physical(physical),
        )
    }

    pub(super) fn expect_eof(&self) -> Result<(), AuthoringDiagnosticV1> {
        self.peek().map_or(Ok(()), |_| Err(self.unexpected()))
    }

    fn peek(&self) -> Option<&AbstractTokenV1> {
        self.tokens.get(self.next)
    }

    fn take(&mut self) -> Result<AbstractTokenV1, AuthoringDiagnosticV1> {
        let token = self.peek().cloned().ok_or_else(|| self.unexpected())?;
        self.next += 1;
        Ok(token)
    }
}

fn is_reserved(identifier: &str) -> bool {
    matches!(
        identifier,
        "format"
            | "schema"
            | "namespace"
            | "revision"
            | "component"
            | "property"
            | "invalidates"
            | "construction"
            | "template"
            | "set"
            | "child"
            | "region"
            | "owner"
            | "repeat"
            | "keys"
            | "style"
            | "bool"
            | "scalar_i32"
            | "rgba8"
            | "input_policy"
            | "true"
            | "false"
            | "accept"
            | "ignore"
            | "structure"
            | "style_match"
            | "intrinsic"
            | "layout"
            | "semantics"
            | "hit_test"
            | "paint"
            | "composition"
            | "surface"
    )
}
