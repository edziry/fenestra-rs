use crate::model::SpatialPointV2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FlattenedSubpathK2 {
    point_start: usize,
    point_length: usize,
    explicitly_closed: bool,
}

impl FlattenedSubpathK2 {
    pub(super) const fn new(
        point_start: usize,
        point_length: usize,
        explicitly_closed: bool,
    ) -> Self {
        Self {
            point_start,
            point_length,
            explicitly_closed,
        }
    }

    pub(crate) const fn point_start(self) -> usize {
        self.point_start
    }

    pub(crate) const fn point_length(self) -> usize {
        self.point_length
    }

    pub(crate) const fn is_explicitly_closed(self) -> bool {
        self.explicitly_closed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FlattenedPathK2 {
    points: Vec<SpatialPointV2>,
    subpaths: Vec<FlattenedSubpathK2>,
    segment_count: usize,
}

impl FlattenedPathK2 {
    pub(super) const fn new(
        points: Vec<SpatialPointV2>,
        subpaths: Vec<FlattenedSubpathK2>,
        segment_count: usize,
    ) -> Self {
        Self {
            points,
            subpaths,
            segment_count,
        }
    }

    pub(crate) fn points(&self) -> &[SpatialPointV2] {
        &self.points
    }

    pub(crate) fn subpaths(&self) -> &[FlattenedSubpathK2] {
        &self.subpaths
    }

    pub(crate) const fn segment_count(&self) -> usize {
        self.segment_count
    }
}
