use crate::baseline::literal_types::{
    CoverageInputV2, FIXED_ONE_V2, FillRuleV2, PointV2, ShapeGeometryInputV2,
};

use super::numeric::{inverse_point, round_ratio};
use super::types::{Aabb, FlatPath, ScenePlan};

pub(super) fn hit_test(
    plan: &ScenePlan<'_>,
    scene_point: PointV2,
) -> Option<(u32, u32, u32, PointV2)> {
    for (index, (input, resolved)) in plan.scene.hits.iter().zip(&plan.hits).enumerate().rev() {
        if !input.accepts
            || !clip_allows(plan, input.clip, scene_point)
            || !resolved.world_bounds.contains(scene_point)
        {
            continue;
        }
        let local = inverse_point(plan.worlds[input.owner as usize], scene_point)?;
        if coverage_contains(plan, input.coverage, resolved.local_bounds, local) {
            return Some((index as u32, input.owner, input.item, local));
        }
    }
    None
}

pub(super) fn clip_allows(
    plan: &ScenePlan<'_>,
    terminal: Option<u32>,
    scene_point: PointV2,
) -> bool {
    let Some(terminal) = terminal else {
        return true;
    };
    if !plan.clips[terminal as usize]
        .effective
        .contains(scene_point)
    {
        return false;
    }
    let mut current = Some(terminal);
    while let Some(key) = current {
        let input = plan.scene.clips[key as usize];
        if !plan.clips[key as usize].primitive.contains(scene_point) {
            return false;
        }
        let Some(local) = inverse_point(plan.worlds[input.owner as usize], scene_point) else {
            return false;
        };
        if !shape_fill_contains(plan, input.shape, input.rule, local) {
            return false;
        }
        current = input.parent;
    }
    true
}

pub(super) fn coverage_contains(
    plan: &ScenePlan<'_>,
    coverage: CoverageInputV2,
    bounds: Aabb,
    query: PointV2,
) -> bool {
    match coverage {
        CoverageInputV2::Fill { shape, rule } => shape_fill_contains(plan, shape, rule, query),
        CoverageInputV2::RoundStroke { shape, width } => {
            shape_stroke_contains(plan, shape, bounds, width as i128, query)
        }
    }
}

fn shape_fill_contains(plan: &ScenePlan<'_>, shape: u32, rule: FillRuleV2, query: PointV2) -> bool {
    let input = &plan.scene.shapes[shape as usize];
    let bounds = plan.shapes[shape as usize].fill;
    if !bounds.contains(query) {
        return false;
    }
    match &input.geometry {
        ShapeGeometryInputV2::Rect(value) => {
            query.x >= value.x
                && query.x < value.x + value.width
                && query.y >= value.y
                && query.y < value.y + value.height
        }
        ShapeGeometryInputV2::Circle { center, radius } => {
            let dx = query.x as i128 - center.x as i128;
            let dy = query.y as i128 - center.y as i128;
            dx * dx + dy * dy <= *radius as i128 * *radius as i128
        }
        ShapeGeometryInputV2::Polygon { points } => fill_segments(points, true, rule, query),
        ShapeGeometryInputV2::Path { path } => path_fill(&plan.paths[*path as usize], rule, query),
    }
}

fn shape_stroke_contains(
    plan: &ScenePlan<'_>,
    shape: u32,
    bounds: Aabb,
    width: i128,
    query: PointV2,
) -> bool {
    if !bounds.contains(query) {
        return false;
    }
    match &plan.scene.shapes[shape as usize].geometry {
        ShapeGeometryInputV2::Rect(value) => {
            let points = [
                PointV2 {
                    x: value.x,
                    y: value.y,
                },
                PointV2 {
                    x: value.x + value.width,
                    y: value.y,
                },
                PointV2 {
                    x: value.x + value.width,
                    y: value.y + value.height,
                },
                PointV2 {
                    x: value.x,
                    y: value.y + value.height,
                },
            ];
            stroke_segments(&points, true, width, query)
        }
        ShapeGeometryInputV2::Circle { center, radius } => {
            let dx = query.x as i128 - center.x as i128;
            let dy = query.y as i128 - center.y as i128;
            let four_distance = 4 * (dx * dx + dy * dy);
            let diameter = 2 * *radius as i128;
            let inner = (diameter - width).max(0);
            let outer = diameter + width;
            inner * inner <= four_distance && four_distance <= outer * outer
        }
        ShapeGeometryInputV2::Polygon { points } => stroke_segments(points, true, width, query),
        ShapeGeometryInputV2::Path { path } => {
            let path = &plan.paths[*path as usize];
            path.subpaths.iter().any(|subpath| {
                let points = &path.points[subpath.start..subpath.start + subpath.length];
                stroke_segments(points, false, width, query)
            })
        }
    }
}

fn path_fill(path: &FlatPath, rule: FillRuleV2, query: PointV2) -> bool {
    let mut accumulator = FillAccumulator::new(query);
    for subpath in &path.subpaths {
        let points = &path.points[subpath.start..subpath.start + subpath.length];
        for pair in points.windows(2) {
            if accumulator.add(pair[0], pair[1]) {
                return true;
            }
        }
        if !subpath.closed
            && accumulator.add(
                *points.last().expect("registered subpath is nonempty"),
                points[0],
            )
        {
            return true;
        }
    }
    accumulator.matches(rule)
}

fn fill_segments(points: &[PointV2], close: bool, rule: FillRuleV2, query: PointV2) -> bool {
    let mut accumulator = FillAccumulator::new(query);
    for pair in points.windows(2) {
        if accumulator.add(pair[0], pair[1]) {
            return true;
        }
    }
    if close
        && accumulator.add(
            *points.last().expect("registered polygon is nonempty"),
            points[0],
        )
    {
        return true;
    }
    accumulator.matches(rule)
}

fn stroke_segments(points: &[PointV2], close: bool, width: i128, query: PointV2) -> bool {
    points
        .windows(2)
        .any(|pair| segment_stroke(pair[0], pair[1], width, query))
        || (close
            && segment_stroke(
                *points
                    .last()
                    .expect("registered stroke points are nonempty"),
                points[0],
                width,
                query,
            ))
}

fn segment_stroke(start: PointV2, end: PointV2, width: i128, query: PointV2) -> bool {
    let dx = end.x as i128 - start.x as i128;
    let dy = end.y as i128 - start.y as i128;
    if dx == 0 && dy == 0 {
        return disk(start.x as i128, start.y as i128, width, query);
    }
    let query_x = query.x as i128 - start.x as i128;
    let query_y = query.y as i128 - start.y as i128;
    let parameter = round_ratio(
        (query_x * dx + query_y * dy) * FIXED_ONE_V2 as i128,
        dx * dx + dy * dy,
    )
    .clamp(0, FIXED_ONE_V2 as i128);
    let closest_x = start.x as i128 + round_ratio(dx * parameter, FIXED_ONE_V2 as i128);
    let closest_y = start.y as i128 + round_ratio(dy * parameter, FIXED_ONE_V2 as i128);
    disk(closest_x, closest_y, width, query)
}

fn disk(x: i128, y: i128, width: i128, query: PointV2) -> bool {
    let dx = query.x as i128 - x;
    let dy = query.y as i128 - y;
    4 * (dx * dx + dy * dy) <= width * width
}

struct FillAccumulator {
    query: PointV2,
    winding: i64,
}

impl FillAccumulator {
    fn new(query: PointV2) -> Self {
        Self { query, winding: 0 }
    }

    fn add(&mut self, start: PointV2, end: PointV2) -> bool {
        if start == end {
            return false;
        }
        let cross = (end.x as i128 - start.x as i128) * (self.query.y as i128 - start.y as i128)
            - (end.y as i128 - start.y as i128) * (self.query.x as i128 - start.x as i128);
        if cross == 0
            && inside(self.query.x, start.x, end.x)
            && inside(self.query.y, start.y, end.y)
        {
            return true;
        }
        if start.y <= self.query.y && self.query.y < end.y && cross > 0 {
            self.winding += 1;
        } else if end.y <= self.query.y && self.query.y < start.y && cross < 0 {
            self.winding -= 1;
        }
        false
    }

    fn matches(self, rule: FillRuleV2) -> bool {
        match rule {
            FillRuleV2::NonZero => self.winding != 0,
            FillRuleV2::EvenOdd => self.winding % 2 != 0,
        }
    }
}

fn inside(value: i64, first: i64, second: i64) -> bool {
    value >= first.min(second) && value <= first.max(second)
}
