use crate::{InteractiveEvidenceV1, InteractiveObservationV1, InteractiveResultV1};

use super::InteractiveArtifactErrorKindV1;
use super::syntax::{exact_keys, parse_digest, parse_extent, parse_u64};

pub(super) struct ArtifactReplay {
    evidence: InteractiveEvidenceV1,
    pub(super) last_generation: Option<u64>,
    last_frame: Option<u64>,
    last_submission: Option<u64>,
    physical: Option<(u32, u32)>,
    logical: Option<(u32, u32)>,
    raster: Option<u64>,
}

impl ArtifactReplay {
    pub(super) const fn new() -> Self {
        Self {
            evidence: InteractiveEvidenceV1::new(),
            last_generation: None,
            last_frame: None,
            last_submission: None,
            physical: None,
            logical: None,
            raster: None,
        }
    }

    pub(super) fn event(
        &mut self,
        event: &[(&str, &str)],
    ) -> Result<(), InteractiveArtifactErrorKindV1> {
        let milestone = event
            .first()
            .filter(|field| field.0 == "milestone")
            .map(|field| field.1)
            .ok_or(InteractiveArtifactErrorKindV1::Grammar)?;
        let observation = match milestone {
            "adapter" => {
                exact_keys(event, &["milestone"])?;
                InteractiveObservationV1::Adapter
            }
            "initial-present" => InteractiveObservationV1::InitialPresent {
                generation: self.present(event, milestone)?,
            },
            "pointer-move" => {
                exact_keys(event, &["milestone"])?;
                InteractiveObservationV1::PointerMove
            }
            "pointer-press" => {
                exact_keys(event, &["milestone"])?;
                InteractiveObservationV1::PointerPress
            }
            "mutation-present" => InteractiveObservationV1::MutationPresent {
                generation: self.present(event, milestone)?,
            },
            "resize" => {
                exact_keys(event, &["milestone", "physical", "logical"])?;
                let physical = parse_extent(event[1].1)?;
                let logical = parse_extent(event[2].1)?;
                if self.physical == Some(physical) || physical != logical {
                    return Err(InteractiveArtifactErrorKindV1::Protocol);
                }
                self.physical = Some(physical);
                self.logical = Some(logical);
                InteractiveObservationV1::Resize
            }
            "resize-present" => InteractiveObservationV1::ResizePresent {
                generation: self.present(event, milestone)?,
            },
            "suspend" => {
                exact_keys(event, &["milestone"])?;
                InteractiveObservationV1::Suspend
            }
            "restore" => {
                exact_keys(event, &["milestone"])?;
                InteractiveObservationV1::Restore
            }
            "restore-present" => InteractiveObservationV1::RestorePresent {
                generation: self.present(event, milestone)?,
            },
            "close" => {
                exact_keys(event, &["milestone"])?;
                InteractiveObservationV1::Close
            }
            _ => return Err(InteractiveArtifactErrorKindV1::Grammar),
        };
        self.evidence
            .observe(observation)
            .map_err(|_| InteractiveArtifactErrorKindV1::Protocol)
    }

    fn present(
        &mut self,
        event: &[(&str, &str)],
        milestone: &str,
    ) -> Result<u64, InteractiveArtifactErrorKindV1> {
        exact_keys(
            event,
            &[
                "milestone",
                "generation",
                "frame",
                "submission",
                "physical",
                "logical",
                "raster",
            ],
        )?;
        let generation = parse_u64(event[1].1)?;
        let frame = parse_u64(event[2].1)?;
        let submission = parse_u64(event[3].1)?;
        let physical = parse_extent(event[4].1)?;
        let logical = parse_extent(event[5].1)?;
        let raster = parse_digest(event[6].1)?;
        if self.last_frame.is_some_and(|last| frame <= last)
            || self.last_submission.is_some_and(|last| submission <= last)
            || physical != logical
            || (self.physical.is_some() && self.physical != Some(physical))
            || (milestone == "mutation-present" && self.raster == Some(raster))
            || (milestone == "restore-present" && self.raster != Some(raster))
        {
            return Err(InteractiveArtifactErrorKindV1::Protocol);
        }
        self.last_generation = Some(generation);
        self.last_frame = Some(frame);
        self.last_submission = Some(submission);
        self.physical = Some(physical);
        self.logical = Some(logical);
        self.raster = Some(raster);
        Ok(generation)
    }
}

pub(super) fn terminal_result(
    result: &[(&str, &str)],
    replay: &ArtifactReplay,
    has_adapter: bool,
    has_surface: bool,
) -> Result<InteractiveResultV1, InteractiveArtifactErrorKindV1> {
    match (result[0].1, result[1].1) {
        ("pass", "complete")
            if has_adapter
                && has_surface
                && replay.evidence.result() == Some(InteractiveResultV1::Pass) =>
        {
            Ok(InteractiveResultV1::Pass)
        }
        ("stop", "operator-close")
            if has_adapter
                && has_surface
                && replay.evidence.result() == Some(InteractiveResultV1::Stop) =>
        {
            Ok(InteractiveResultV1::Stop)
        }
        ("adapt", reason)
            if replay.evidence.result().is_none()
                && matches!(
                    reason,
                    "adapter-unavailable"
                        | "backend"
                        | "device-type"
                        | "identity"
                        | "surface-format"
                        | "device-request"
                        | "renderer"
                        | "surface"
                        | "out-of-memory"
                        | "timeout"
                ) =>
        {
            Ok(InteractiveResultV1::Adapt)
        }
        _ => Err(InteractiveArtifactErrorKindV1::Terminal),
    }
}
