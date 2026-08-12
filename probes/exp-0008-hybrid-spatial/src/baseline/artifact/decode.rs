use super::super::EvidenceSectionV2;
use super::GrammarValueKindV2 as Grammar;
use super::{
    ARTIFACT_LIMITS_V2, ArtifactCaseV2, ArtifactControlV2, ArtifactErrorKindV2, ArtifactErrorV2,
    ArtifactKindV2, ArtifactLimitKindV2, ArtifactObservationV2, ArtifactSectionV2, CASE_NAMES,
    CONTROL_FAMILIES, HEADER, LIMITS, OBSERVATION_COUNTS, PACKAGES, PROFILE,
    SpatialEvidenceArtifactV2, grammar_value_accepts_v2, validate_record_grammar,
    verify_spatial_evidence_artifact_v2,
};

pub(crate) fn decode_spatial_evidence_artifact_v2(
    bytes: &[u8],
) -> Result<SpatialEvidenceArtifactV2, ArtifactErrorV2> {
    let lines = preflight(bytes)?;
    let mut parser = Parser { lines, next: 0 };
    parser.header()?;
    parser.exact(PACKAGES)?;
    parser.exact(PROFILE)?;
    parser.exact(LIMITS)?;
    let mut cases = Vec::with_capacity(CASE_NAMES.len());
    for ordinal in 0..CASE_NAMES.len() {
        cases.push(parser.case(ordinal)?);
    }
    let mut controls = Vec::with_capacity(CONTROL_FAMILIES.len());
    for family in CONTROL_FAMILIES {
        controls.push(parser.control(family)?);
    }
    parser.exact("result|literal=pass|reference=pass|candidate-count=0")?;
    parser.exact("end|spatial-v2")?;
    if parser.next != parser.lines.len() {
        return Err(ArtifactErrorV2::at(
            ArtifactErrorKindV2::InvalidOrder,
            parser.next,
        ));
    }
    let artifact = SpatialEvidenceArtifactV2 {
        kind: ArtifactKindV2::Baseline,
        candidate_count: 0,
        cases,
        controls,
    };
    verify_spatial_evidence_artifact_v2(&artifact)?;
    Ok(artifact)
}

fn preflight(bytes: &[u8]) -> Result<Vec<&str>, ArtifactErrorV2> {
    if bytes.len() > ARTIFACT_LIMITS_V2.artifact_bytes {
        return Err(ArtifactErrorV2::limit(
            ArtifactLimitKindV2::ArtifactBytes,
            bytes.len(),
            ARTIFACT_LIMITS_V2.artifact_bytes,
            None,
        ));
    }
    if !bytes.is_ascii()
        || !bytes.ends_with(b"\n")
        || bytes.ends_with(b"\n\n")
        || bytes.contains(&b'\r')
    {
        return Err(ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidGrammar));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidGrammar))?;
    let lines = text
        .strip_suffix('\n')
        .expect("final LF was checked")
        .split('\n')
        .collect::<Vec<_>>();
    if lines.len() > ARTIFACT_LIMITS_V2.records {
        return Err(ArtifactErrorV2::limit(
            ArtifactLimitKindV2::Records,
            lines.len(),
            ARTIFACT_LIMITS_V2.records,
            None,
        ));
    }
    for (record, line) in lines.iter().enumerate() {
        validate_record_grammar(line, record)?;
        if line.len() > ARTIFACT_LIMITS_V2.line_bytes {
            return Err(ArtifactErrorV2::limit(
                ArtifactLimitKindV2::LineBytes,
                line.len(),
                ARTIFACT_LIMITS_V2.line_bytes,
                Some(record),
            ));
        }
    }
    Ok(lines)
}

struct Parser<'a> {
    lines: Vec<&'a str>,
    next: usize,
}

impl<'a> Parser<'a> {
    fn header(&mut self) -> Result<(), ArtifactErrorV2> {
        let line = self.take()?;
        if line == HEADER {
            return Ok(());
        }
        if line.starts_with("spatial-v2|") {
            return Err(ArtifactErrorV2::at(
                ArtifactErrorKindV2::InvalidVersion,
                self.next - 1,
            ));
        }
        Err(ArtifactErrorV2::at(
            ArtifactErrorKindV2::InvalidOrder,
            self.next - 1,
        ))
    }

    fn exact(&mut self, expected: &str) -> Result<(), ArtifactErrorV2> {
        let line = self.take()?;
        if line == expected {
            Ok(())
        } else {
            Err(ArtifactErrorV2::at(
                ArtifactErrorKindV2::InvalidOrder,
                self.next - 1,
            ))
        }
    }

    fn case(&mut self, ordinal: usize) -> Result<ArtifactCaseV2, ArtifactErrorV2> {
        let fields = self.fields("case", &["ordinal", "name", "observations"])?;
        if parse_unsigned(fields[0])? != ordinal as u64 {
            return self.error(ArtifactErrorKindV2::InvalidReference);
        }
        if fields[1] != CASE_NAMES[ordinal] {
            return self.error(ArtifactErrorKindV2::InvalidOrder);
        }
        if parse_unsigned(fields[2])? != OBSERVATION_COUNTS[ordinal] as u64 {
            return self.error(ArtifactErrorKindV2::InvalidCount);
        }
        let mut observations = Vec::with_capacity(OBSERVATION_COUNTS[ordinal]);
        for step in 0..OBSERVATION_COUNTS[ordinal] {
            observations.push(self.observation(ordinal, step)?);
        }
        self.exact(&format!(
            "case-result|case={ordinal}|literal=match|reference=match|repeat=match"
        ))?;
        Ok(ArtifactCaseV2 {
            ordinal: ordinal as u8,
            name: CASE_NAMES[ordinal].to_owned(),
            observations,
            literal_match: true,
            reference_match: true,
            repeat_match: true,
        })
    }

    fn observation(
        &mut self,
        case: usize,
        step: usize,
    ) -> Result<ArtifactObservationV2, ArtifactErrorV2> {
        let fields = self.fields("observation", &["case", "step", "generation", "viewport"])?;
        if parse_unsigned(fields[0])? != case as u64 || parse_unsigned(fields[1])? != step as u64 {
            return self.error(ArtifactErrorKindV2::InvalidReference);
        }
        let generation = parse_optional(fields[2])?;
        let viewport = parse_viewport(fields[3])?;
        let mut sections = Vec::with_capacity(EvidenceSectionV2::ALL.len());
        for section in EvidenceSectionV2::ALL {
            sections.push(self.section(case, step, section)?);
        }
        Ok(ArtifactObservationV2 {
            case: case as u8,
            step: step as u8,
            generation,
            viewport,
            sections,
        })
    }

    fn section(
        &mut self,
        case: usize,
        step: usize,
        section: EvidenceSectionV2,
    ) -> Result<ArtifactSectionV2, ArtifactErrorV2> {
        let fields = self.fields(
            "section",
            &["case", "step", "name", "records", "bytes", "digest"],
        )?;
        if parse_unsigned(fields[0])? != case as u64 || parse_unsigned(fields[1])? != step as u64 {
            return self.error(ArtifactErrorKindV2::InvalidReference);
        }
        if fields[2] != section.token() {
            return self.error(ArtifactErrorKindV2::InvalidOrder);
        }
        Ok(ArtifactSectionV2 {
            name: section,
            records: parse_unsigned(fields[3])?,
            bytes: parse_unsigned(fields[4])?,
            digest: parse_hex16(fields[5])?,
        })
    }

    fn control(&mut self, family: &str) -> Result<ArtifactControlV2, ArtifactErrorV2> {
        let fields = self.fields("control", &["family", "registered", "detected"])?;
        if fields[0] != family {
            return self.error(ArtifactErrorKindV2::InvalidOrder);
        }
        let registered = parse_unsigned(fields[1])?;
        let detected = parse_unsigned(fields[2])?;
        if registered != detected {
            return self.error(ArtifactErrorKindV2::InvalidCount);
        }
        Ok(ArtifactControlV2 {
            family: family.to_owned(),
            registered,
            detected,
        })
    }

    fn fields(&mut self, record: &str, names: &[&str]) -> Result<Vec<&'a str>, ArtifactErrorV2> {
        let line = self.take()?;
        let mut parts = line.split('|');
        if parts.next() != Some(record) {
            return self.error(ArtifactErrorKindV2::InvalidOrder);
        }
        let mut values = Vec::with_capacity(names.len());
        for name in names {
            let Some(field) = parts.next() else {
                return self.error(ArtifactErrorKindV2::InvalidOrder);
            };
            let Some((actual, value)) = field.split_once('=') else {
                return self.error(ArtifactErrorKindV2::InvalidGrammar);
            };
            if actual != *name {
                return self.error(ArtifactErrorKindV2::InvalidOrder);
            }
            values.push(value);
        }
        if parts.next().is_some() {
            return self.error(ArtifactErrorKindV2::InvalidOrder);
        }
        Ok(values)
    }

    fn take(&mut self) -> Result<&'a str, ArtifactErrorV2> {
        let line = self
            .lines
            .get(self.next)
            .copied()
            .ok_or_else(|| ArtifactErrorV2::at(ArtifactErrorKindV2::InvalidCount, self.next))?;
        self.next += 1;
        Ok(line)
    }

    fn error<T>(&self, kind: ArtifactErrorKindV2) -> Result<T, ArtifactErrorV2> {
        Err(ArtifactErrorV2::at(kind, self.next.saturating_sub(1)))
    }
}

fn parse_unsigned(value: &str) -> Result<u64, ArtifactErrorV2> {
    if !grammar_value_accepts_v2(Grammar::Unsigned, value) {
        return Err(ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidGrammar));
    }
    value
        .parse()
        .map_err(|_| ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidGrammar))
}

fn parse_optional(value: &str) -> Result<Option<u64>, ArtifactErrorV2> {
    if grammar_value_accepts_v2(Grammar::Absent, value) {
        Ok(None)
    } else {
        parse_unsigned(value).map(Some)
    }
}

fn parse_viewport(value: &str) -> Result<(u32, u32), ArtifactErrorV2> {
    let (width, height) = value
        .split_once('x')
        .ok_or_else(|| ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidGrammar))?;
    let width = u32::try_from(parse_unsigned(width)?)
        .map_err(|_| ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidGrammar))?;
    let height = u32::try_from(parse_unsigned(height)?)
        .map_err(|_| ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidGrammar))?;
    Ok((width, height))
}

fn parse_hex16(value: &str) -> Result<u64, ArtifactErrorV2> {
    if !grammar_value_accepts_v2(Grammar::Hex16, value) {
        return Err(ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidGrammar));
    }
    u64::from_str_radix(value, 16)
        .map_err(|_| ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidGrammar))
}
