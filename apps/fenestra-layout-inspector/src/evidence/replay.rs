use super::{ARTIFACT_LIMITS, EvidenceError, EvidenceMilestone, EvidenceResult, VerifiedEvidence};

pub(super) fn verify_artifact(bytes: &[u8]) -> Result<VerifiedEvidence, EvidenceError> {
    let lines = bounded_lines(bytes)?;
    if lines.first().copied() != Some(super::HEADER) {
        return Err(EvidenceError::Grammar);
    }
    let mut replay = Replay::default();
    let mut cursor = 1;
    while cursor < lines.len() && lines[cursor].starts_with("event|") {
        replay.event(parse_fields(Some(lines[cursor]), "event")?)?;
        cursor += 1;
    }
    let result = parse_fields(lines.get(cursor).copied(), "result")?;
    exact_keys(&result, &["kind", "reason"])?;
    if cursor + 1 != lines.len() {
        return Err(EvidenceError::Terminal);
    }
    let declared = match (result[0].1, result[1].1) {
        ("pass", "complete") if replay.complete() => EvidenceResult::Pass,
        ("stop", "incomplete") if !replay.complete() => EvidenceResult::Stop,
        _ => return Err(EvidenceError::Terminal),
    };
    Ok(VerifiedEvidence {
        result: declared,
        record_count: lines.len(),
        byte_count: bytes.len(),
        final_generation: replay.last_generation,
    })
}

#[derive(Default)]
struct Replay {
    milestones: Vec<EvidenceMilestone>,
    last_generation: Option<u64>,
    viewport: Option<(i32, i32)>,
    nodes: Option<usize>,
    keys: Vec<u64>,
    close: bool,
}

impl Replay {
    fn event(&mut self, fields: Vec<(&str, &str)>) -> Result<(), EvidenceError> {
        let milestone = field(&fields, "milestone")?;
        let expected = EvidenceMilestone::ALL.get(self.milestones.len()).copied();
        let current = parse_milestone(milestone)?;
        if expected != Some(current) {
            return Err(EvidenceError::Order);
        }
        match current {
            EvidenceMilestone::InitialPresent => self.initial(&fields)?,
            EvidenceMilestone::PointerMove => self.pointer_move(&fields)?,
            EvidenceMilestone::PointerPress => self.pointer_press(&fields)?,
            EvidenceMilestone::KeyedInsert => self.keyed_insert(&fields)?,
            EvidenceMilestone::MutationPresent => self.present(&fields, "mutation-present")?,
            EvidenceMilestone::Resize => self.resize(&fields)?,
            EvidenceMilestone::ResizePresent => self.present(&fields, "resize-present")?,
            EvidenceMilestone::Close => {
                exact_keys(&fields, &["milestone"])?;
                self.close = true;
            }
        }
        self.milestones.push(current);
        Ok(())
    }

    fn initial(&mut self, fields: &[(&str, &str)]) -> Result<(), EvidenceError> {
        exact_keys(
            fields,
            &[
                "milestone",
                "generation",
                "viewport",
                "nodes",
                "keys",
                "hover",
                "selected",
                "raster-bytes",
            ],
        )?;
        let generation = parse_u64(fields[1].1)?;
        let viewport = parse_viewport(fields[2].1)?;
        let nodes = parse_usize(fields[3].1)?;
        let keys = parse_keys(fields[4].1)?;
        if nodes == 0 || keys.is_empty() || parse_flag(fields[5].1)? || parse_flag(fields[6].1)? {
            return Err(EvidenceError::Coherence);
        }
        verify_raster(viewport, fields[7].1)?;
        self.last_generation = Some(generation);
        self.viewport = Some(viewport);
        self.nodes = Some(nodes);
        self.keys = keys;
        Ok(())
    }

    fn pointer_move(&self, fields: &[(&str, &str)]) -> Result<(), EvidenceError> {
        exact_keys(fields, &["milestone", "x", "y", "hit", "generation"])?;
        let _ = parse_i32(fields[1].1)?;
        let _ = parse_i32(fields[2].1)?;
        if !parse_flag(fields[3].1)? || Some(parse_u64(fields[4].1)?) != self.last_generation {
            return Err(EvidenceError::Coherence);
        }
        Ok(())
    }

    fn pointer_press(&mut self, fields: &[(&str, &str)]) -> Result<(), EvidenceError> {
        exact_keys(fields, &["milestone", "generation", "selected"])?;
        let generation = parse_u64(fields[1].1)?;
        if !parse_flag(fields[2].1)?
            || self
                .last_generation
                .is_none_or(|previous| generation <= previous)
        {
            return Err(EvidenceError::Coherence);
        }
        self.last_generation = Some(generation);
        Ok(())
    }

    fn keyed_insert(&mut self, fields: &[(&str, &str)]) -> Result<(), EvidenceError> {
        exact_keys(fields, &["milestone", "key", "generation", "nodes", "keys"])?;
        let key = parse_u64(fields[1].1)?;
        let generation = parse_u64(fields[2].1)?;
        let nodes = parse_usize(fields[3].1)?;
        let keys = parse_keys(fields[4].1)?;
        let expected_nodes = self.nodes.ok_or(EvidenceError::Coherence)? + 1;
        let mut expected_keys = self.keys.clone();
        expected_keys.push(key);
        if nodes != expected_nodes
            || keys != expected_keys
            || self
                .last_generation
                .is_none_or(|previous| generation <= previous)
        {
            return Err(EvidenceError::Coherence);
        }
        self.last_generation = Some(generation);
        self.nodes = Some(nodes);
        self.keys = keys;
        Ok(())
    }

    fn present(&mut self, fields: &[(&str, &str)], milestone: &str) -> Result<(), EvidenceError> {
        exact_keys(
            fields,
            &["milestone", "generation", "viewport", "raster-bytes"],
        )?;
        let generation = parse_u64(fields[1].1)?;
        let generation_ok = match milestone {
            "mutation-present" => Some(generation) == self.last_generation,
            "resize-present" => self
                .last_generation
                .is_some_and(|previous| generation > previous),
            _ => false,
        };
        if fields[0].1 != milestone
            || !generation_ok
            || Some(parse_viewport(fields[2].1)?) != self.viewport
        {
            return Err(EvidenceError::Coherence);
        }
        verify_raster(self.viewport.ok_or(EvidenceError::Coherence)?, fields[3].1)?;
        self.last_generation = Some(generation);
        Ok(())
    }

    fn resize(&mut self, fields: &[(&str, &str)]) -> Result<(), EvidenceError> {
        exact_keys(fields, &["milestone", "viewport"])?;
        let viewport = parse_viewport(fields[1].1)?;
        if Some(viewport) == self.viewport {
            return Err(EvidenceError::Coherence);
        }
        self.viewport = Some(viewport);
        Ok(())
    }

    fn complete(&self) -> bool {
        self.close && self.milestones.as_slice() == EvidenceMilestone::ALL
    }
}

fn bounded_lines(bytes: &[u8]) -> Result<Vec<&str>, EvidenceError> {
    if bytes.len() > ARTIFACT_LIMITS.artifact_bytes()
        || bytes.iter().filter(|byte| **byte == b'\n').count() > ARTIFACT_LIMITS.records()
        || bytes
            .split(|byte| *byte == b'\n')
            .any(|line| line.len() > ARTIFACT_LIMITS.line_bytes())
        || bytes.is_empty()
        || !bytes.ends_with(b"\n")
        || bytes.ends_with(b"\n\n")
        || bytes
            .iter()
            .any(|byte| *byte != b'\n' && !(b' '..=b'~').contains(byte))
    {
        return Err(
            if bytes.len() > ARTIFACT_LIMITS.artifact_bytes()
                || bytes.iter().filter(|byte| **byte == b'\n').count() > ARTIFACT_LIMITS.records()
                || bytes
                    .split(|byte| *byte == b'\n')
                    .any(|line| line.len() > ARTIFACT_LIMITS.line_bytes())
            {
                EvidenceError::Bounds
            } else {
                EvidenceError::Encoding
            },
        );
    }
    let text = std::str::from_utf8(bytes).map_err(|_| EvidenceError::Encoding)?;
    Ok(text.lines().collect())
}

fn parse_fields<'a>(
    line: Option<&'a str>,
    record: &str,
) -> Result<Vec<(&'a str, &'a str)>, EvidenceError> {
    let mut parts = line.ok_or(EvidenceError::Grammar)?.split('|');
    if parts.next() != Some(record) {
        return Err(EvidenceError::Grammar);
    }
    let mut fields = Vec::new();
    for part in parts {
        let (key, value) = part.split_once('=').ok_or(EvidenceError::Grammar)?;
        if key.is_empty()
            || value.is_empty()
            || value.contains('=')
            || fields.iter().any(|field: &(&str, &str)| field.0 == key)
        {
            return Err(EvidenceError::Grammar);
        }
        fields.push((key, value));
    }
    Ok(fields)
}

fn exact_keys(fields: &[(&str, &str)], expected: &[&str]) -> Result<(), EvidenceError> {
    if fields.len() != expected.len()
        || fields
            .iter()
            .zip(expected)
            .any(|(field, expected)| field.0 != *expected)
    {
        return Err(EvidenceError::Grammar);
    }
    Ok(())
}

fn field<'a>(fields: &[(&'a str, &'a str)], name: &str) -> Result<&'a str, EvidenceError> {
    fields
        .iter()
        .find_map(|(key, value)| (*key == name).then_some(*value))
        .ok_or(EvidenceError::Grammar)
}

fn parse_milestone(value: &str) -> Result<EvidenceMilestone, EvidenceError> {
    match value {
        "initial-present" => Ok(EvidenceMilestone::InitialPresent),
        "pointer-move" => Ok(EvidenceMilestone::PointerMove),
        "pointer-press" => Ok(EvidenceMilestone::PointerPress),
        "keyed-insert" => Ok(EvidenceMilestone::KeyedInsert),
        "mutation-present" => Ok(EvidenceMilestone::MutationPresent),
        "resize" => Ok(EvidenceMilestone::Resize),
        "resize-present" => Ok(EvidenceMilestone::ResizePresent),
        "close" => Ok(EvidenceMilestone::Close),
        _ => Err(EvidenceError::Grammar),
    }
}

fn parse_u64(value: &str) -> Result<u64, EvidenceError> {
    value.parse().map_err(|_| EvidenceError::Grammar)
}

fn parse_usize(value: &str) -> Result<usize, EvidenceError> {
    value.parse().map_err(|_| EvidenceError::Grammar)
}

fn parse_i32(value: &str) -> Result<i32, EvidenceError> {
    value.parse().map_err(|_| EvidenceError::Grammar)
}

fn parse_flag(value: &str) -> Result<bool, EvidenceError> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(EvidenceError::Grammar),
    }
}

fn parse_viewport(value: &str) -> Result<(i32, i32), EvidenceError> {
    let (width, height) = value.split_once('x').ok_or(EvidenceError::Grammar)?;
    let viewport = (parse_i32(width)?, parse_i32(height)?);
    if viewport.0 <= 0 || viewport.1 <= 0 {
        return Err(EvidenceError::Coherence);
    }
    Ok(viewport)
}

fn parse_keys(value: &str) -> Result<Vec<u64>, EvidenceError> {
    value
        .split(',')
        .map(parse_u64)
        .collect::<Result<Vec<_>, _>>()
}

fn verify_raster(viewport: (i32, i32), value: &str) -> Result<(), EvidenceError> {
    let bytes = parse_usize(value)?;
    let expected = usize::try_from(viewport.0)
        .ok()
        .and_then(|width| {
            usize::try_from(viewport.1)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(EvidenceError::Coherence)?;
    (bytes == expected)
        .then_some(())
        .ok_or(EvidenceError::Coherence)
}
