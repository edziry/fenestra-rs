use fenestra_ui_ir::prototype::{SourceId, SourceSpan};

use crate::compiled_v2::{CompiledAuthoringV2, SourceMapEntryV2};
use crate::semantic::{InvalidRecord, validate_name};
use crate::vocabulary_v2::AnchorKindV2;

pub(super) struct SourceCatalog<'a> {
    entries: &'a [SourceMapEntryV2],
}

impl<'a> SourceCatalog<'a> {
    pub(super) fn new(
        compiled: &'a CompiledAuthoringV2,
        expected_count: usize,
    ) -> Result<Self, InvalidRecord> {
        let entries = compiled.source_map().entries();
        let source = compiled.logical_source_catalog();
        if entries.len() != expected_count
            || source.len() != expected_count
            || !source.iter().all(|byte| *byte == b'@')
        {
            return Err(InvalidRecord);
        }
        for (ordinal, entry) in entries.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).map_err(|_| InvalidRecord)?;
            let end = ordinal.checked_add(1).ok_or(InvalidRecord)?;
            if entry.logical_span() != SourceSpan::bytes(SourceId::new(0), ordinal, end)
                || entry.canonical_label().is_empty()
                || !entry.canonical_label().is_ascii()
            {
                return Err(InvalidRecord);
            }
        }
        Ok(Self { entries })
    }

    pub(super) fn anchor(
        &self,
        span: SourceSpan,
        kind: AnchorKindV2,
    ) -> Result<u32, InvalidRecord> {
        let anchor = span_anchor(span)?;
        let entry = self
            .entries
            .get(usize::try_from(anchor).map_err(|_| InvalidRecord)?)
            .ok_or(InvalidRecord)?;
        if entry.anchor_kind() != kind || entry.logical_span() != span {
            return Err(InvalidRecord);
        }
        Ok(anchor)
    }

    pub(super) fn named_anchor(
        &self,
        span: SourceSpan,
        kind: AnchorKindV2,
    ) -> Result<(u32, &'a str), InvalidRecord> {
        let anchor = self.anchor(span, kind)?;
        let name =
            self.entries[usize::try_from(anchor).map_err(|_| InvalidRecord)?].canonical_label();
        validate_name(name)?;
        Ok((anchor, name))
    }

    pub(super) fn nth_anchor(
        &self,
        kind: AnchorKindV2,
        ordinal: usize,
    ) -> Result<u32, InvalidRecord> {
        let entry = self
            .entries
            .iter()
            .filter(|entry| entry.anchor_kind() == kind)
            .nth(ordinal)
            .ok_or(InvalidRecord)?;
        span_anchor(entry.logical_span())
    }
}

fn span_anchor(span: SourceSpan) -> Result<u32, InvalidRecord> {
    let SourceSpan::Bytes { source, start, end } = span else {
        return Err(InvalidRecord);
    };
    if source != SourceId::new(0) || start.checked_add(1) != Some(end) {
        return Err(InvalidRecord);
    }
    Ok(start)
}
