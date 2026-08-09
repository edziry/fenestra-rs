use super::super::error::{
    HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1,
    HeadlessArtifactSectionKindV1 as Section,
};
use super::grammar::{CapacityRowV1, RecordKindV1, classify_line_v1};
use super::scan::ScannedLineV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SectionRangeV1 {
    pub(super) declaration: usize,
    pub(super) records_start: usize,
    pub(super) records_end: usize,
    pub(super) terminal: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LayoutV1 {
    pub(super) headless: SectionRangeV1,
    pub(super) scheduler: SectionRangeV1,
    pub(super) projection_begin: usize,
    pub(super) computed: SectionRangeV1,
    pub(super) geometry: SectionRangeV1,
    pub(super) semantics: SectionRangeV1,
    pub(super) hits: SectionRangeV1,
    pub(super) scene: SectionRangeV1,
    pub(super) projection_end: usize,
    pub(super) result: usize,
    pub(super) end: usize,
    pub(super) trailing_start: usize,
}

pub(super) fn scan_layout_v1(
    lines: &[ScannedLineV1<'_>],
) -> Result<LayoutV1, HeadlessArtifactDecodeErrorV1> {
    StateV1::new(lines).scan()
}

struct StateV1<'a, 'source> {
    lines: &'a [ScannedLineV1<'source>],
    cursor: usize,
}

impl<'a, 'source> StateV1<'a, 'source> {
    const fn new(lines: &'a [ScannedLineV1<'source>]) -> Self {
        Self { lines, cursor: 0 }
    }

    fn scan(mut self) -> Result<LayoutV1, HeadlessArtifactDecodeErrorV1> {
        self.singleton(RecordKindV1::Header, Section::Header)?;
        self.singleton(RecordKindV1::Versions, Section::Versions)?;
        self.singleton(RecordKindV1::Fixture, Section::Fixture)?;
        self.singleton(RecordKindV1::Environment, Section::Environment)?;
        self.singleton(RecordKindV1::ProjectionChoices, Section::ProjectionChoices)?;
        self.capacities()?;
        let headless = self.body(
            RecordKindV1::HeadlessBegin,
            RecordKindV1::HeadlessEvent,
            RecordKindV1::HeadlessEnd,
            Section::HeadlessTrace,
        )?;
        let scheduler = self.body(
            RecordKindV1::SchedulerBegin,
            RecordKindV1::SchedulerEvent,
            RecordKindV1::SchedulerEnd,
            Section::SchedulerTrace,
        )?;
        let projection_begin =
            self.singleton(RecordKindV1::ProjectionBegin, Section::Projection)?;
        let computed = self.body(
            RecordKindV1::ComputedBegin,
            RecordKindV1::Computed,
            RecordKindV1::ComputedEnd,
            Section::ComputedStyles,
        )?;
        let geometry = self.body(
            RecordKindV1::GeometryBegin,
            RecordKindV1::Geometry,
            RecordKindV1::GeometryEnd,
            Section::Geometry,
        )?;
        let semantics = self.body(
            RecordKindV1::SemanticBegin,
            RecordKindV1::Semantic,
            RecordKindV1::SemanticEnd,
            Section::Semantics,
        )?;
        let hits = self.body(
            RecordKindV1::HitBegin,
            RecordKindV1::Hit,
            RecordKindV1::HitEnd,
            Section::HitRegions,
        )?;
        let scene = self.body(
            RecordKindV1::SceneBegin,
            RecordKindV1::Scene,
            RecordKindV1::SceneEnd,
            Section::SceneRectangles,
        )?;
        let projection_end = self.singleton(RecordKindV1::ProjectionEnd, Section::Projection)?;
        let result = self.singleton(RecordKindV1::Result, Section::Result)?;
        let end = self.singleton(RecordKindV1::End, Section::End)?;
        Ok(LayoutV1 {
            headless,
            scheduler,
            projection_begin,
            computed,
            geometry,
            semantics,
            hits,
            scene,
            projection_end,
            result,
            end,
            trailing_start: self.cursor,
        })
    }

    fn capacities(&mut self) -> Result<(), HeadlessArtifactDecodeErrorV1> {
        for row in [
            CapacityRowV1::Ir,
            CapacityRowV1::Style,
            CapacityRowV1::Runtime,
            CapacityRowV1::Projection,
            CapacityRowV1::Scheduler,
            CapacityRowV1::Renderer,
            CapacityRowV1::SchedulerTrace,
            CapacityRowV1::HeadlessTrace,
            CapacityRowV1::Artifact,
        ] {
            let line = self.required(Section::Capacities)?;
            match classify_line_v1(line)? {
                RecordKindV1::Capacity(observed) if observed == row => self.cursor += 1,
                RecordKindV1::Capacity(_) => return Err(ordering(line.number)),
                _ => return Err(missing(Section::Capacities, line.number)),
            }
        }
        Ok(())
    }

    fn singleton(
        &mut self,
        expected: RecordKindV1,
        section: Section,
    ) -> Result<usize, HeadlessArtifactDecodeErrorV1> {
        let line = self.required(section)?;
        let observed = classify_line_v1(line)?;
        if observed == expected {
            let index = self.cursor;
            self.cursor += 1;
            return Ok(index);
        }
        if is_body_record(observed) {
            return Err(ordering(line.number));
        }
        let observed_section = record_section(observed);
        match section_rank(observed_section).cmp(&section_rank(section)) {
            std::cmp::Ordering::Less => Err(duplicate(observed_section, line.number)),
            std::cmp::Ordering::Equal => Err(duplicate(section, line.number)),
            std::cmp::Ordering::Greater => Err(missing(section, line.number)),
        }
    }

    fn body(
        &mut self,
        begin: RecordKindV1,
        record: RecordKindV1,
        end: RecordKindV1,
        section: Section,
    ) -> Result<SectionRangeV1, HeadlessArtifactDecodeErrorV1> {
        let declaration = self.cursor;
        let line = self.required(section)?;
        let observed = classify_line_v1(line)?;
        if observed != begin {
            return if observed == record || observed == end {
                Err(missing(section, line.number))
            } else {
                Err(ordering(line.number))
            };
        }
        self.cursor += 1;
        let records_start = self.cursor;
        loop {
            let line = self.required(section)?;
            let observed = classify_line_v1(line)?;
            if observed == record {
                self.cursor += 1;
                continue;
            }
            if observed == end {
                let terminal = self.cursor;
                self.cursor += 1;
                return Ok(SectionRangeV1 {
                    declaration,
                    records_start,
                    records_end: terminal,
                    terminal,
                });
            }
            if observed == begin {
                return Err(duplicate(section, line.number));
            }
            return Err(ordering(line.number));
        }
    }

    fn required(
        &self,
        section: Section,
    ) -> Result<&'a ScannedLineV1<'source>, HeadlessArtifactDecodeErrorV1> {
        self.lines.get(self.cursor).ok_or_else(|| {
            HeadlessArtifactDecodeErrorV1::new(
                HeadlessArtifactDecodeErrorKindV1::MissingSection(section),
                None,
            )
        })
    }
}

fn is_body_record(kind: RecordKindV1) -> bool {
    matches!(
        kind,
        RecordKindV1::HeadlessEvent
            | RecordKindV1::SchedulerEvent
            | RecordKindV1::Computed
            | RecordKindV1::Geometry
            | RecordKindV1::Semantic
            | RecordKindV1::Hit
            | RecordKindV1::Scene
    )
}

const fn record_section(kind: RecordKindV1) -> Section {
    match kind {
        RecordKindV1::Header => Section::Header,
        RecordKindV1::Versions => Section::Versions,
        RecordKindV1::Fixture => Section::Fixture,
        RecordKindV1::Environment => Section::Environment,
        RecordKindV1::ProjectionChoices => Section::ProjectionChoices,
        RecordKindV1::Capacity(_) => Section::Capacities,
        RecordKindV1::HeadlessBegin | RecordKindV1::HeadlessEvent | RecordKindV1::HeadlessEnd => {
            Section::HeadlessTrace
        }
        RecordKindV1::SchedulerBegin
        | RecordKindV1::SchedulerEvent
        | RecordKindV1::SchedulerEnd => Section::SchedulerTrace,
        RecordKindV1::ProjectionBegin | RecordKindV1::ProjectionEnd => Section::Projection,
        RecordKindV1::ComputedBegin | RecordKindV1::Computed | RecordKindV1::ComputedEnd => {
            Section::ComputedStyles
        }
        RecordKindV1::GeometryBegin | RecordKindV1::Geometry | RecordKindV1::GeometryEnd => {
            Section::Geometry
        }
        RecordKindV1::SemanticBegin | RecordKindV1::Semantic | RecordKindV1::SemanticEnd => {
            Section::Semantics
        }
        RecordKindV1::HitBegin | RecordKindV1::Hit | RecordKindV1::HitEnd => Section::HitRegions,
        RecordKindV1::SceneBegin | RecordKindV1::Scene | RecordKindV1::SceneEnd => {
            Section::SceneRectangles
        }
        RecordKindV1::Result => Section::Result,
        RecordKindV1::End => Section::End,
    }
}

const fn section_rank(section: Section) -> u8 {
    match section {
        Section::Header => 0,
        Section::Versions => 1,
        Section::Fixture => 2,
        Section::Environment => 3,
        Section::ProjectionChoices => 4,
        Section::Capacities => 5,
        Section::HeadlessTrace => 6,
        Section::SchedulerTrace => 7,
        Section::Projection => 8,
        Section::ComputedStyles => 9,
        Section::Geometry => 10,
        Section::Semantics => 11,
        Section::HitRegions => 12,
        Section::SceneRectangles => 13,
        Section::Result => 14,
        Section::End => 15,
    }
}

fn missing(section: Section, line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(
        HeadlessArtifactDecodeErrorKindV1::MissingSection(section),
        line,
    )
}

fn duplicate(section: Section, line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(
        HeadlessArtifactDecodeErrorKindV1::DuplicateSection(section),
        line,
    )
}

fn ordering(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(HeadlessArtifactDecodeErrorKindV1::OrderingViolation, line)
}
