use crate::model::SpatialPointV2;
use crate::path::SpatialPathVerbV2;

use super::path::ValidatedPathK1;

mod curve;
mod error;
mod model;

pub(crate) use error::GeometryK2Error;
pub(crate) use error::{GeometryK2ErrorKind, GeometryK2LimitKind};
pub(crate) use model::FlattenedPathK2;

use error::GeometryK2LimitKind::{FlattenedSegmentsPerPath, FlattenedSegmentsTotal};
use model::FlattenedSubpathK2;

struct SegmentEmitter {
    points: Vec<SpatialPointV2>,
    path_segments: usize,
    total_segments: u128,
    maximum_per_path: u128,
    maximum_total: u128,
}

impl SegmentEmitter {
    fn new(accepted_total: usize, maximum_per_path: usize, maximum_total: usize) -> Self {
        Self {
            points: Vec::new(),
            path_segments: 0,
            total_segments: accepted_total as u128,
            maximum_per_path: maximum_per_path as u128,
            maximum_total: maximum_total as u128,
        }
    }

    fn begin_subpath(&mut self, point: SpatialPointV2) -> usize {
        let point_start = self.points.len();
        self.points.push(point);
        point_start
    }

    fn emit(
        &mut self,
        endpoint: SpatialPointV2,
        path: u32,
        source_verb: u32,
    ) -> Result<(), GeometryK2Error> {
        let observed_path = self.path_segments as u128 + 1;
        if observed_path > self.maximum_per_path {
            return Err(GeometryK2Error::limit(
                FlattenedSegmentsPerPath,
                path,
                source_verb,
                observed_path,
                self.maximum_per_path,
            ));
        }

        let observed_total = self.total_segments + 1;
        if observed_total > self.maximum_total {
            return Err(GeometryK2Error::limit(
                FlattenedSegmentsTotal,
                path,
                source_verb,
                observed_total,
                self.maximum_total,
            ));
        }

        self.path_segments = self
            .path_segments
            .checked_add(1)
            .expect("an accepted path segment fits its owned point table");
        self.total_segments = observed_total;
        self.points.push(endpoint);
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ActiveSubpath {
    point_start: usize,
    first_point: SpatialPointV2,
}

pub(crate) fn flatten_path_k2(
    path: u32,
    validated: ValidatedPathK1<'_>,
    accepted_total: usize,
    maximum_per_path: usize,
    maximum_total: usize,
) -> Result<FlattenedPathK2, GeometryK2Error> {
    let mut emitter = SegmentEmitter::new(accepted_total, maximum_per_path, maximum_total);
    let mut subpaths = Vec::with_capacity(validated.subpath_count());
    let mut active = None;
    let mut current = None;

    for (ordinal, verb) in validated.verbs().iter().copied().enumerate() {
        let source_verb = ordinal as u32;
        match verb {
            SpatialPathVerbV2::MoveTo { to } => {
                finish_active_subpath(&emitter, &mut subpaths, active.take(), false);
                active = Some(ActiveSubpath {
                    point_start: emitter.begin_subpath(to),
                    first_point: to,
                });
                current = Some(to);
            }
            SpatialPathVerbV2::LineTo { to } => {
                emitter.emit(to, path, source_verb)?;
                current = Some(to);
            }
            SpatialPathVerbV2::QuadraticTo { control, to } => {
                curve::flatten_quadratic(
                    path,
                    source_verb,
                    current.expect("K1 proof guarantees an active subpath"),
                    control,
                    to,
                    &mut emitter,
                )?;
                current = Some(to);
            }
            SpatialPathVerbV2::CubicTo {
                control1,
                control2,
                to,
            } => {
                curve::flatten_cubic(
                    path,
                    source_verb,
                    current.expect("K1 proof guarantees an active subpath"),
                    control1,
                    control2,
                    to,
                    &mut emitter,
                )?;
                current = Some(to);
            }
            SpatialPathVerbV2::Close => {
                let subpath = active.expect("K1 proof guarantees an active subpath");
                emitter.emit(subpath.first_point, path, source_verb)?;
                finish_active_subpath(&emitter, &mut subpaths, active.take(), true);
                current = None;
            }
        }
    }
    finish_active_subpath(&emitter, &mut subpaths, active, false);

    Ok(FlattenedPathK2::new(
        emitter.points,
        subpaths,
        emitter.path_segments,
    ))
}

fn finish_active_subpath(
    emitter: &SegmentEmitter,
    subpaths: &mut Vec<FlattenedSubpathK2>,
    active: Option<ActiveSubpath>,
    explicitly_closed: bool,
) {
    if let Some(active) = active {
        subpaths.push(FlattenedSubpathK2::new(
            active.point_start,
            emitter.points.len() - active.point_start,
            explicitly_closed,
        ));
    }
}
