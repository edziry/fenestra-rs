use crate::aabb::SpatialAabbV2;
use crate::coverage::SpatialFillRuleV2;
use crate::model::SpatialPointV2;

pub(super) const fn bounds_contains(bounds: SpatialAabbV2, query: SpatialPointV2) -> bool {
    !bounds.is_empty()
        && query.x().raw() >= bounds.min_x().raw()
        && query.y().raw() >= bounds.min_y().raw()
        && query.x().raw() <= bounds.max_x().raw()
        && query.y().raw() <= bounds.max_y().raw()
}

pub(super) struct FillAccumulator {
    query: SpatialPointV2,
    winding: i64,
}

impl FillAccumulator {
    pub(super) const fn new(query: SpatialPointV2) -> Self {
        Self { query, winding: 0 }
    }

    pub(super) fn add_segment(&mut self, start: SpatialPointV2, end: SpatialPointV2) -> bool {
        if start == end {
            return false;
        }

        let start_x = i128::from(start.x().raw());
        let start_y = i128::from(start.y().raw());
        let end_x = i128::from(end.x().raw());
        let end_y = i128::from(end.y().raw());
        let query_x = i128::from(self.query.x().raw());
        let query_y = i128::from(self.query.y().raw());
        let cross =
            (end_x - start_x) * (query_y - start_y) - (end_y - start_y) * (query_x - start_x);

        if cross == 0
            && inside_inclusive(query_x, start_x, end_x)
            && inside_inclusive(query_y, start_y, end_y)
        {
            return true;
        }

        if start_y <= query_y && query_y < end_y && cross > 0 {
            self.winding += 1;
        } else if end_y <= query_y && query_y < start_y && cross < 0 {
            self.winding -= 1;
        }
        false
    }

    pub(super) const fn matches_rule(self, rule: SpatialFillRuleV2) -> bool {
        match rule {
            SpatialFillRuleV2::NonZero => self.winding != 0,
            SpatialFillRuleV2::EvenOdd => self.winding % 2 != 0,
        }
    }
}

const fn inside_inclusive(value: i128, first: i128, second: i128) -> bool {
    if first <= second {
        value >= first && value <= second
    } else {
        value >= second && value <= first
    }
}
