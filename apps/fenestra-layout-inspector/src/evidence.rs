use crate::InspectorFrame;

mod format;
mod replay;

use format::{flag, keys, viewport};

const HEADER: &str = "fenestra-layout-inspector|artifact=1|wu=0015";

/// Ordered milestones required by the native inspector operator sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceMilestone {
    /// The first frame completed native presentation.
    InitialPresent,
    /// A pointer moved over a hit target.
    PointerMove,
    /// A pointer press committed a selection.
    PointerPress,
    /// A keyed tile was committed.
    KeyedInsert,
    /// The keyed mutation completed native presentation.
    MutationPresent,
    /// A nonzero viewport change was committed.
    Resize,
    /// The resized frame completed native presentation.
    ResizePresent,
    /// The native window closed normally.
    Close,
}

impl EvidenceMilestone {
    /// Complete native milestone order.
    pub const ALL: [Self; 8] = [
        Self::InitialPresent,
        Self::PointerMove,
        Self::PointerPress,
        Self::KeyedInsert,
        Self::MutationPresent,
        Self::Resize,
        Self::ResizePresent,
        Self::Close,
    ];
}

/// Terminal classification of one inspector artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceResult {
    /// All native milestones completed in order.
    Pass,
    /// The window closed before all milestones completed.
    Stop,
}

/// Failures from building or independently verifying inspector evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceError {
    /// A record, line, or artifact byte bound was exceeded.
    Bounds,
    /// The artifact was not printable ASCII with one final LF.
    Encoding,
    /// A record shape or value was malformed.
    Grammar,
    /// An event did not match the next required milestone.
    Order,
    /// A frame value contradicted the preceding observation.
    Coherence,
    /// The sequence ended before the pass prefix was complete.
    Incomplete,
    /// An event followed a terminal close.
    Terminal,
}

/// Inclusive bounds for one layout inspector artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvidenceLimits {
    records: usize,
    line_bytes: usize,
    artifact_bytes: usize,
}

impl EvidenceLimits {
    /// Creates bounded artifact limits.
    #[must_use]
    pub const fn new(records: usize, line_bytes: usize, artifact_bytes: usize) -> Self {
        Self {
            records,
            line_bytes,
            artifact_bytes,
        }
    }

    /// Returns the maximum number of LF-terminated records.
    #[must_use]
    pub const fn records(self) -> usize {
        self.records
    }

    /// Returns the maximum bytes in one line excluding LF.
    #[must_use]
    pub const fn line_bytes(self) -> usize {
        self.line_bytes
    }

    /// Returns the maximum total artifact bytes.
    #[must_use]
    pub const fn artifact_bytes(self) -> usize {
        self.artifact_bytes
    }
}

/// Registered bounds for the WU-0015 native artifact.
pub const ARTIFACT_LIMITS: EvidenceLimits = EvidenceLimits::new(32, 256, 4_096);

/// Summary returned after independent artifact verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedEvidence {
    result: EvidenceResult,
    record_count: usize,
    byte_count: usize,
    final_generation: Option<u64>,
}

impl VerifiedEvidence {
    /// Returns the verified terminal classification.
    #[must_use]
    pub const fn result(self) -> EvidenceResult {
        self.result
    }

    /// Returns the number of LF-terminated records.
    #[must_use]
    pub const fn record_count(self) -> usize {
        self.record_count
    }

    /// Returns the total artifact byte count.
    #[must_use]
    pub const fn byte_count(self) -> usize {
        self.byte_count
    }

    /// Returns the last presented runtime generation.
    #[must_use]
    pub const fn final_generation(self) -> Option<u64> {
        self.final_generation
    }

    /// Returns the complete accepted milestone order.
    #[must_use]
    pub const fn milestones(self) -> &'static [EvidenceMilestone] {
        EvidenceMilestone::ALL.as_slice()
    }
}

/// Typed builder for the native inspector evidence artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutInspectorEvidence {
    encoded: String,
    milestones: Vec<EvidenceMilestone>,
    last_generation: Option<u64>,
    last_viewport: Option<(i32, i32)>,
    closed: bool,
}

impl LayoutInspectorEvidence {
    /// Starts an empty native inspector evidence sequence.
    #[must_use]
    pub fn new() -> Self {
        Self {
            encoded: format!("{HEADER}\n"),
            milestones: Vec::new(),
            last_generation: None,
            last_viewport: None,
            closed: false,
        }
    }

    /// Returns the next milestone required by the sequence.
    #[must_use]
    pub fn next_required(&self) -> Option<EvidenceMilestone> {
        EvidenceMilestone::ALL.get(self.milestones.len()).copied()
    }

    /// Records the first successfully presented frame.
    pub fn record_initial(&mut self, frame: &InspectorFrame) -> Result<(), EvidenceError> {
        self.require(EvidenceMilestone::InitialPresent)?;
        let (width, height) = viewport(frame);
        let line = format!(
            "event|milestone=initial-present|generation={}|viewport={width}x{height}|nodes={}|keys={}|hover={}|selected={}|raster-bytes={}",
            frame.generation(),
            frame.node_count(),
            keys(frame.keyed_keys()),
            flag(frame.has_hover()),
            flag(frame.has_selection()),
            frame.raster_bytes()
        );
        if frame.has_hover() || frame.has_selection() {
            return Err(EvidenceError::Coherence);
        }
        self.commit(EvidenceMilestone::InitialPresent, &line, frame.generation());
        self.last_viewport = Some((width, height));
        Ok(())
    }

    /// Records a hit-producing native pointer move.
    pub fn record_pointer_move(
        &mut self,
        x: i32,
        y: i32,
        frame: &InspectorFrame,
    ) -> Result<(), EvidenceError> {
        self.require(EvidenceMilestone::PointerMove)?;
        if !frame.has_hover() || Some(frame.generation()) != self.last_generation {
            return Err(EvidenceError::Coherence);
        }
        let line = format!(
            "event|milestone=pointer-move|x={x}|y={y}|hit=1|generation={}",
            frame.generation()
        );
        self.commit(EvidenceMilestone::PointerMove, &line, frame.generation());
        Ok(())
    }

    /// Records a selection committed by the native primary-button press.
    pub fn record_pointer_press(&mut self, frame: &InspectorFrame) -> Result<(), EvidenceError> {
        self.require(EvidenceMilestone::PointerPress)?;
        if !frame.has_selection()
            || self
                .last_generation
                .is_none_or(|generation| frame.generation() <= generation)
        {
            return Err(EvidenceError::Coherence);
        }
        let line = format!(
            "event|milestone=pointer-press|generation={}|selected=1",
            frame.generation()
        );
        self.commit(EvidenceMilestone::PointerPress, &line, frame.generation());
        Ok(())
    }

    /// Records a keyed insertion committed by a native key event.
    pub fn record_keyed_insert(
        &mut self,
        key: u64,
        frame: &InspectorFrame,
    ) -> Result<(), EvidenceError> {
        self.require(EvidenceMilestone::KeyedInsert)?;
        if self
            .last_generation
            .is_none_or(|generation| frame.generation() <= generation)
            || frame.keyed_keys().last().copied() != Some(key)
        {
            return Err(EvidenceError::Coherence);
        }
        let line = format!(
            "event|milestone=keyed-insert|key={key}|generation={}|nodes={}|keys={}",
            frame.generation(),
            frame.node_count(),
            keys(frame.keyed_keys())
        );
        self.commit(EvidenceMilestone::KeyedInsert, &line, frame.generation());
        Ok(())
    }

    /// Records the frame after keyed mutation presentation.
    pub fn record_mutation_present(&mut self, frame: &InspectorFrame) -> Result<(), EvidenceError> {
        self.require(EvidenceMilestone::MutationPresent)?;
        self.record_present("mutation-present", frame, self.last_generation)
    }

    /// Records a committed nonzero viewport resize.
    pub fn record_resize(&mut self, frame: &InspectorFrame) -> Result<(), EvidenceError> {
        self.require(EvidenceMilestone::Resize)?;
        let viewport = viewport(frame);
        if viewport.0 <= 0 || viewport.1 <= 0 || self.last_viewport == Some(viewport) {
            return Err(EvidenceError::Coherence);
        }
        let line = format!(
            "event|milestone=resize|viewport={}x{}",
            viewport.0, viewport.1
        );
        self.commit(EvidenceMilestone::Resize, &line, frame.generation());
        self.last_viewport = Some(viewport);
        Ok(())
    }

    /// Records the frame after resize presentation.
    pub fn record_resize_present(&mut self, frame: &InspectorFrame) -> Result<(), EvidenceError> {
        self.require(EvidenceMilestone::ResizePresent)?;
        self.record_present("resize-present", frame, self.last_generation)
    }

    /// Records normal native window closure.
    pub fn record_close(&mut self) -> Result<(), EvidenceError> {
        self.require(EvidenceMilestone::Close)?;
        self.push_line("event|milestone=close")?;
        self.milestones.push(EvidenceMilestone::Close);
        self.closed = true;
        Ok(())
    }

    /// Finishes the sequence and independently verifies its bytes.
    pub fn finish(mut self) -> Result<Vec<u8>, EvidenceError> {
        if !self.closed || self.milestones.as_slice() != EvidenceMilestone::ALL {
            return Err(EvidenceError::Incomplete);
        }
        self.push_line("result|kind=pass|reason=complete")?;
        let bytes = self.encoded.into_bytes();
        let verified = verify_artifact(&bytes)?;
        if verified.result != EvidenceResult::Pass {
            return Err(EvidenceError::Incomplete);
        }
        Ok(bytes)
    }

    fn record_present(
        &mut self,
        milestone: &str,
        frame: &InspectorFrame,
        expected_generation: Option<u64>,
    ) -> Result<(), EvidenceError> {
        if Some(frame.generation()) != expected_generation
            || Some(viewport(frame)) != self.last_viewport
        {
            return Err(EvidenceError::Coherence);
        }
        let (width, height) = viewport(frame);
        let line = format!(
            "event|milestone={milestone}|generation={}|viewport={width}x{height}|raster-bytes={}",
            frame.generation(),
            frame.raster_bytes()
        );
        self.commit(
            match milestone {
                "mutation-present" => EvidenceMilestone::MutationPresent,
                "resize-present" => EvidenceMilestone::ResizePresent,
                _ => unreachable!("present milestone is closed"),
            },
            &line,
            frame.generation(),
        );
        Ok(())
    }

    fn require(&self, milestone: EvidenceMilestone) -> Result<(), EvidenceError> {
        if self.closed {
            return Err(EvidenceError::Terminal);
        }
        if self.next_required() != Some(milestone) {
            return Err(EvidenceError::Order);
        }
        Ok(())
    }

    fn commit(&mut self, milestone: EvidenceMilestone, line: &str, generation: u64) {
        self.push_line(line)
            .expect("validated artifact line bounds");
        self.milestones.push(milestone);
        self.last_generation = Some(generation);
    }

    fn push_line(&mut self, line: &str) -> Result<(), EvidenceError> {
        if line.len() > ARTIFACT_LIMITS.line_bytes()
            || self.milestones.len() + 1 >= ARTIFACT_LIMITS.records()
            || self
                .encoded
                .len()
                .checked_add(line.len())
                .and_then(|bytes| bytes.checked_add(1))
                .is_none_or(|bytes| bytes > ARTIFACT_LIMITS.artifact_bytes())
        {
            return Err(EvidenceError::Bounds);
        }
        self.encoded.push_str(line);
        self.encoded.push('\n');
        Ok(())
    }
}

impl Default for LayoutInspectorEvidence {
    fn default() -> Self {
        Self::new()
    }
}

/// Independently verifies one native inspector artifact.
pub fn verify_artifact(bytes: &[u8]) -> Result<VerifiedEvidence, EvidenceError> {
    replay::verify_artifact(bytes)
}
