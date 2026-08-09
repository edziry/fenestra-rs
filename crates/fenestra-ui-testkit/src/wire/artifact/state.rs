use super::super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind, SectionKind};
use super::super::scan::ScannedLine;
use super::boundary::{EnvelopeLayoutV1, SectionRangeV1};
use super::grammar::{MarkerRoleV1, RecordKindV1, classify_line_v1};

pub(super) fn scan_boundaries_v1(
    lines: &[ScannedLine<'_>],
) -> Result<EnvelopeLayoutV1, ArtifactDecodeError> {
    StateV1::new(lines).scan()
}

struct StateV1<'scan, 'source> {
    lines: &'scan [ScannedLine<'source>],
    cursor: usize,
}

impl<'scan, 'source> StateV1<'scan, 'source> {
    const fn new(lines: &'scan [ScannedLine<'source>]) -> Self {
        Self { lines, cursor: 0 }
    }

    fn scan(mut self) -> Result<EnvelopeLayoutV1, ArtifactDecodeError> {
        self.consume_singleton(SectionKind::Header)?;
        self.consume_singleton(SectionKind::Versions)?;
        self.consume_singleton(SectionKind::Fixture)?;
        self.consume_singleton(SectionKind::Replay)?;
        self.consume_singleton(SectionKind::Generator)?;
        self.consume_singleton(SectionKind::Seed)?;
        let original = self.consume_section(SectionKind::Original, BodyKindV1::Case)?;
        self.consume_singleton(SectionKind::Fault)?;
        self.consume_singleton(SectionKind::OriginalFailure)?;
        self.consume_singleton(SectionKind::Reducer)?;
        let minimized = self.consume_section(SectionKind::Minimized, BodyKindV1::Case)?;
        self.consume_singleton(SectionKind::MinimizedFailure)?;
        let trace = self.consume_section(SectionKind::Trace, BodyKindV1::Trace)?;
        self.consume_singleton(SectionKind::End)?;
        Ok(EnvelopeLayoutV1::new(original, minimized, trace))
    }

    fn consume_singleton(&mut self, expected: SectionKind) -> Result<(), ArtifactDecodeError> {
        let line = self.next_required(expected)?;
        match classify_line_v1(line)? {
            RecordKindV1::Marker {
                section,
                role: MarkerRoleV1::Singleton,
            } if section == expected => Ok(()),
            RecordKindV1::Marker { section, .. } => {
                Err(marker_error(expected, section, line.number))
            }
            RecordKindV1::Case(_) | RecordKindV1::Trace => Err(ordering(line.number)),
        }
    }

    fn consume_section(
        &mut self,
        section: SectionKind,
        body: BodyKindV1,
    ) -> Result<SectionRangeV1, ArtifactDecodeError> {
        let begin = self.cursor;
        let begin_line = self.next_required(section)?;
        match classify_line_v1(begin_line)? {
            RecordKindV1::Marker {
                section: observed,
                role: MarkerRoleV1::Begin,
            } if observed == section => {}
            RecordKindV1::Marker {
                section: observed, ..
            } if observed == section => {
                return Err(ArtifactDecodeError::at(
                    ArtifactDecodeErrorKind::MissingSection(section),
                    begin_line.number,
                ));
            }
            RecordKindV1::Marker {
                section: observed, ..
            } => return Err(marker_error(section, observed, begin_line.number)),
            RecordKindV1::Case(_) | RecordKindV1::Trace => {
                return Err(ordering(begin_line.number));
            }
        }

        loop {
            let index = self.cursor;
            let line = self.next_required(section)?;
            match classify_line_v1(line)? {
                RecordKindV1::Marker {
                    section: observed,
                    role: MarkerRoleV1::End,
                } if observed == section => return Ok(SectionRangeV1::new(begin, index)),
                RecordKindV1::Marker {
                    section: observed, ..
                } if observed == section => {
                    return Err(ArtifactDecodeError::at(
                        ArtifactDecodeErrorKind::DuplicateSection(section),
                        line.number,
                    ));
                }
                RecordKindV1::Marker {
                    section: observed, ..
                } => return Err(marker_error(section, observed, line.number)),
                RecordKindV1::Case(_) if body == BodyKindV1::Case => {}
                RecordKindV1::Trace if body == BodyKindV1::Trace => {}
                RecordKindV1::Case(_) | RecordKindV1::Trace => {
                    return Err(ordering(line.number));
                }
            }
        }
    }

    fn next_required(
        &mut self,
        expected: SectionKind,
    ) -> Result<&'scan ScannedLine<'source>, ArtifactDecodeError> {
        let line = self.lines.get(self.cursor).ok_or_else(|| {
            ArtifactDecodeError::new(ArtifactDecodeErrorKind::MissingSection(expected), None)
        })?;
        self.cursor += 1;
        Ok(line)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum BodyKindV1 {
    Case,
    Trace,
}

fn marker_error(expected: SectionKind, observed: SectionKind, line: u32) -> ArtifactDecodeError {
    let kind = if section_rank(observed) < section_rank(expected) {
        ArtifactDecodeErrorKind::DuplicateSection(observed)
    } else if section_rank(observed) > section_rank(expected) {
        ArtifactDecodeErrorKind::MissingSection(expected)
    } else {
        ArtifactDecodeErrorKind::OrderingViolation
    };
    ArtifactDecodeError::at(kind, line)
}

fn ordering(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::OrderingViolation, line)
}

const fn section_rank(section: SectionKind) -> u8 {
    match section {
        SectionKind::Header => 0,
        SectionKind::Versions => 1,
        SectionKind::Fixture => 2,
        SectionKind::Replay => 3,
        SectionKind::Generator => 4,
        SectionKind::Seed => 5,
        SectionKind::Original => 6,
        SectionKind::Fault => 7,
        SectionKind::OriginalFailure => 8,
        SectionKind::Reducer => 9,
        SectionKind::Minimized => 10,
        SectionKind::MinimizedFailure => 11,
        SectionKind::Trace => 12,
        SectionKind::End => 13,
    }
}
