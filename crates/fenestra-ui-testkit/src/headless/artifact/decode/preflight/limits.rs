use super::super::super::error::{
    HeadlessArtifactCountKindV1 as CountKind, HeadlessArtifactDecodeErrorKindV1 as ErrorKind,
    HeadlessArtifactDecodeErrorV1, HeadlessArtifactLimitKindV1 as LimitKind,
};
use super::super::scan::ScannedArtifactV1;
use super::super::state::LayoutV1;
use super::canonical::InspectedV1;

const ARTIFACT_LIMITS: [usize; 3] = [65_536, 1_024, 512];
const HEADLESS_LIMITS: [usize; 2] = [128, 20_480];
const SCHEDULER_LIMITS: [usize; 2] = [256, 24_576];
const PROJECTION_LIMITS: [usize; 5] = [8, 8, 1, 8, 8];
const PATH_DEPTH_LIMIT: usize = 3;

pub(super) fn validate_limits_and_counts_v1(
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
    inspected: &InspectedV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    validate_artifact_limits(scanned, inspected)?;
    let lines = scanned.lines();
    let headless_line = lines[layout.headless.declaration].number;
    let scheduler_line = lines[layout.scheduler.declaration].number;
    validate_limit(
        inspected.capacities.headless_trace[0],
        HEADLESS_LIMITS[0],
        inspected.declared_headless[0],
        inspected.actual.headless_events,
        13,
        headless_line,
        LimitKind::HeadlessEvents,
    )?;
    validate_limit(
        inspected.capacities.headless_trace[1],
        HEADLESS_LIMITS[1],
        inspected.declared_headless[1],
        inspected.actual.headless_bytes,
        13,
        headless_line,
        LimitKind::HeadlessTraceBytes,
    )?;
    validate_limit(
        inspected.capacities.scheduler_trace[0],
        SCHEDULER_LIMITS[0],
        inspected.declared_scheduler[0],
        inspected.actual.scheduler_events,
        12,
        scheduler_line,
        LimitKind::SchedulerEvents,
    )?;
    validate_limit(
        inspected.capacities.scheduler_trace[1],
        SCHEDULER_LIMITS[1],
        inspected.declared_scheduler[1],
        inspected.actual.scheduler_bytes,
        12,
        scheduler_line,
        LimitKind::SchedulerTraceBytes,
    )?;
    let projection_line = lines[layout.projection_begin].number;
    for (index, hard) in PROJECTION_LIMITS.iter().copied().enumerate() {
        validate_limit(
            inspected.capacities.projection[index],
            hard,
            inspected.declared_projection[index],
            inspected.actual.projection[index],
            9,
            projection_line,
            projection_limit(index),
        )?;
    }
    validate_path_limit(inspected)?;
    validate_counts(inspected, headless_line, scheduler_line, projection_line)
}

fn validate_artifact_limits(
    scanned: &ScannedArtifactV1<'_>,
    inspected: &InspectedV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let actual = [
        scanned.artifact_bytes(),
        scanned.max_line_bytes(),
        scanned.lines().len(),
    ];
    for index in 0..ARTIFACT_LIMITS.len() {
        let configured = inspected.capacities.artifact[index];
        if configured > ARTIFACT_LIMITS[index] || configured < actual[index] {
            return Err(limit_error(artifact_limit(index), 14));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_limit(
    configured: usize,
    hard: usize,
    declared: usize,
    actual: usize,
    configured_line: u32,
    declaration_line: u32,
    kind: LimitKind,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    if actual > hard {
        return Err(limit_error(kind, declaration_line));
    }
    if configured > hard || configured < actual {
        return Err(limit_error(kind, configured_line));
    }
    if declared > hard {
        return Err(limit_error(kind, declaration_line));
    }
    Ok(())
}

fn validate_path_limit(inspected: &InspectedV1) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    if inspected.actual.max_path_depth > PATH_DEPTH_LIMIT {
        return Err(limit_error(
            LimitKind::PathDepth,
            inspected.actual.max_path_line.unwrap_or(6),
        ));
    }
    let configured = inspected.capacities.ir[7];
    if configured > PATH_DEPTH_LIMIT || configured < inspected.actual.max_path_depth {
        return Err(limit_error(LimitKind::PathDepth, 6));
    }
    Ok(())
}

fn validate_counts(
    inspected: &InspectedV1,
    headless_line: u32,
    scheduler_line: u32,
    projection_line: u32,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    for (declared, actual, kind) in [
        (
            inspected.declared_headless[0],
            inspected.actual.headless_events,
            CountKind::HeadlessEvents,
        ),
        (
            inspected.declared_headless[1],
            inspected.actual.headless_bytes,
            CountKind::HeadlessTraceBytes,
        ),
    ] {
        if declared != actual {
            return Err(count_error(kind, headless_line));
        }
    }
    for (declared, actual, kind) in [
        (
            inspected.declared_scheduler[0],
            inspected.actual.scheduler_events,
            CountKind::SchedulerEvents,
        ),
        (
            inspected.declared_scheduler[1],
            inspected.actual.scheduler_bytes,
            CountKind::SchedulerTraceBytes,
        ),
    ] {
        if declared != actual {
            return Err(count_error(kind, scheduler_line));
        }
    }
    for index in 0..inspected.actual.projection.len() {
        if inspected.declared_projection[index] != inspected.actual.projection[index] {
            return Err(count_error(projection_count(index), projection_line));
        }
    }
    Ok(())
}

const fn artifact_limit(index: usize) -> LimitKind {
    [
        LimitKind::ArtifactBytes,
        LimitKind::LineBytes,
        LimitKind::Lines,
    ][index]
}

const fn projection_limit(index: usize) -> LimitKind {
    [
        LimitKind::ComputedStyles,
        LimitKind::Geometry,
        LimitKind::Semantics,
        LimitKind::HitRegions,
        LimitKind::SceneRectangles,
    ][index]
}

const fn projection_count(index: usize) -> CountKind {
    [
        CountKind::ComputedStyles,
        CountKind::Geometry,
        CountKind::Semantics,
        CountKind::HitRegions,
        CountKind::SceneRectangles,
    ][index]
}

fn limit_error(kind: LimitKind, line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(ErrorKind::LimitExceeded(kind), line)
}

fn count_error(kind: CountKind, line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(ErrorKind::CountMismatch(kind), line)
}
