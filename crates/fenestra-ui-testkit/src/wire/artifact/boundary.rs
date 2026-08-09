use super::super::scan::ScannedLine;

#[derive(Clone, Copy)]
pub(super) struct SectionRangeV1 {
    begin: usize,
    end: usize,
}

impl SectionRangeV1 {
    pub(super) const fn new(begin: usize, end: usize) -> Self {
        Self { begin, end }
    }
}

#[derive(Clone, Copy)]
pub(super) struct EnvelopeLayoutV1 {
    original: SectionRangeV1,
    minimized: SectionRangeV1,
    trace: SectionRangeV1,
}

impl EnvelopeLayoutV1 {
    pub(super) const fn new(
        original: SectionRangeV1,
        minimized: SectionRangeV1,
        trace: SectionRangeV1,
    ) -> Self {
        Self {
            original,
            minimized,
            trace,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SectionBoundaryV1<'scan, 'source> {
    lines: &'scan [ScannedLine<'source>],
    range: SectionRangeV1,
}

impl<'scan, 'source> SectionBoundaryV1<'scan, 'source> {
    const fn new(lines: &'scan [ScannedLine<'source>], range: SectionRangeV1) -> Self {
        Self { lines, range }
    }

    pub(super) const fn begin(self) -> &'scan ScannedLine<'source> {
        &self.lines[self.range.begin]
    }

    pub(super) fn records(self) -> &'scan [ScannedLine<'source>] {
        &self.lines[(self.range.begin + 1)..self.range.end]
    }

    pub(super) const fn end(self) -> &'scan ScannedLine<'source> {
        &self.lines[self.range.end]
    }
}

pub(in crate::wire) struct EnvelopeBoundariesV1<'source> {
    pub(super) lines: Vec<ScannedLine<'source>>,
    layout: EnvelopeLayoutV1,
}

impl<'source> EnvelopeBoundariesV1<'source> {
    pub(super) const fn new(lines: Vec<ScannedLine<'source>>, layout: EnvelopeLayoutV1) -> Self {
        Self { lines, layout }
    }

    pub(super) fn original(&self) -> SectionBoundaryV1<'_, 'source> {
        SectionBoundaryV1::new(&self.lines, self.layout.original)
    }

    pub(super) fn minimized(&self) -> SectionBoundaryV1<'_, 'source> {
        SectionBoundaryV1::new(&self.lines, self.layout.minimized)
    }

    pub(super) fn trace(&self) -> SectionBoundaryV1<'_, 'source> {
        SectionBoundaryV1::new(&self.lines, self.layout.trace)
    }
}
