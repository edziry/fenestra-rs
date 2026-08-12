use crate::baseline::literal_types::{CoverageInputV2, FillRuleV2, PaintContentInputV2, PointV2};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Aabb {
    pub(super) empty: bool,
    pub(super) edges: [i64; 4],
}

impl Aabb {
    pub(super) const EMPTY: Self = Self {
        empty: true,
        edges: [0; 4],
    };

    pub(super) const fn closed(edges: [i64; 4]) -> Self {
        Self {
            empty: false,
            edges,
        }
    }

    pub(super) fn intersection(self, other: Self) -> Self {
        if self.empty || other.empty {
            return Self::EMPTY;
        }
        let edges = [
            self.edges[0].max(other.edges[0]),
            self.edges[1].max(other.edges[1]),
            self.edges[2].min(other.edges[2]),
            self.edges[3].min(other.edges[3]),
        ];
        if edges[0] > edges[2] || edges[1] > edges[3] {
            Self::EMPTY
        } else {
            Self::closed(edges)
        }
    }

    pub(super) const fn contains(self, point: PointV2) -> bool {
        !self.empty
            && point.x >= self.edges[0]
            && point.x <= self.edges[2]
            && point.y >= self.edges[1]
            && point.y <= self.edges[3]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Base {
    pub(super) x: i64,
    pub(super) y: i64,
    pub(super) width: i32,
    pub(super) height: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Affine {
    pub(super) values: [i64; 6],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FlatPath {
    pub(super) points: Vec<PointV2>,
    pub(super) subpaths: Vec<Subpath>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Subpath {
    pub(super) start: usize,
    pub(super) length: usize,
    pub(super) closed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ShapePlan {
    pub(super) base: Aabb,
    pub(super) fill: Aabb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ScenePlan<'a> {
    pub(super) scene: &'a crate::baseline::literal_types::SceneInputV2,
    pub(super) bases: Vec<Base>,
    pub(super) worlds: Vec<Affine>,
    pub(super) paths: Vec<FlatPath>,
    pub(super) shapes: Vec<ShapePlan>,
    pub(super) clips: Vec<ResolvedClip>,
    pub(super) paints: Vec<ResolvedPaint<'a>>,
    pub(super) hits: Vec<ResolvedHit>,
    pub(super) semantics: Vec<ResolvedSemantic>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ResolvedClip {
    pub(super) primitive: Aabb,
    pub(super) effective: Aabb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ResolvedPaint<'a> {
    pub(super) input: &'a PaintContentInputV2,
    pub(super) local_bounds: Aabb,
    pub(super) world_bounds: Aabb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ResolvedHit {
    pub(super) coverage: CoverageInputV2,
    pub(super) local_bounds: Aabb,
    pub(super) world_bounds: Aabb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ResolvedSemantic {
    pub(super) shape: u32,
    pub(super) rule: FillRuleV2,
    pub(super) world_bounds: Aabb,
}
