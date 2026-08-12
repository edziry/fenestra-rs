use super::numeric::{Point, SCALE, contains, round_ratio};
use super::scene::{Coverage, Rule, Scene, Shape};
use super::types::Aabb;

const FLATNESS: i128 = 256;

pub fn coverage_contains(scene: &Scene, coverage: Coverage, bounds: Aabb, point: Point) -> bool {
    match coverage {
        Coverage::Fill { shape, rule } => shape_contains(scene, shape, rule, bounds, point),
        Coverage::Stroke { shape, width } => stroke_contains(scene, shape, width, bounds, point),
    }
}

pub fn shape_contains(scene: &Scene, shape: u32, rule: Rule, bounds: Aabb, point: Point) -> bool {
    if !contains(bounds, point) {
        return false;
    }
    match &scene.shapes[usize::try_from(shape).expect("shape key should fit")] {
        Shape::Rect {
            origin,
            width,
            height,
        } => {
            point[0] >= origin[0]
                && point[0] < origin[0] + width
                && point[1] >= origin[1]
                && point[1] < origin[1] + height
        }
        Shape::Circle { center, radius } => {
            let dx = i128::from(point[0] - center[0]);
            let dy = i128::from(point[1] - center[1]);
            let radius = i128::from(*radius);
            radius != 0 && dx * dx + dy * dy <= radius * radius
        }
        Shape::Polygon(points) => polygon_contains(points, rule, point),
        Shape::Path => polygon_contains(&scene.path, rule, point),
    }
}

fn stroke_contains(scene: &Scene, shape: u32, width: i64, bounds: Aabb, point: Point) -> bool {
    if !contains(bounds, point) {
        return false;
    }
    match &scene.shapes[usize::try_from(shape).expect("shape key should fit")] {
        Shape::Circle { center, radius } => {
            let dx = i128::from(point[0] - center[0]);
            let dy = i128::from(point[1] - center[1]);
            let distance = 4 * (dx * dx + dy * dy);
            let diameter = 2 * i128::from(*radius);
            let width = i128::from(width);
            let inner = (diameter - width).max(0);
            let outer = diameter + width;
            inner * inner <= distance && distance <= outer * outer
        }
        Shape::Path => scene
            .path
            .windows(2)
            .any(|pair| segment_stroke(pair[0], pair[1], width, point)),
        Shape::Rect {
            origin,
            width: rect_width,
            height,
        } => {
            let corners = [
                *origin,
                [origin[0] + rect_width, origin[1]],
                [origin[0] + rect_width, origin[1] + height],
                [origin[0], origin[1] + height],
                *origin,
            ];
            corners
                .windows(2)
                .any(|pair| segment_stroke(pair[0], pair[1], width, point))
        }
        Shape::Polygon(points) => points
            .iter()
            .copied()
            .zip(points.iter().copied().cycle().skip(1))
            .take(points.len())
            .any(|(start, end)| segment_stroke(start, end, width, point)),
    }
}

fn polygon_contains(points: &[Point], rule: Rule, query: Point) -> bool {
    let mut winding = 0_i64;
    for (start, end) in points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
    {
        let start_x = i128::from(start[0]);
        let start_y = i128::from(start[1]);
        let end_x = i128::from(end[0]);
        let end_y = i128::from(end[1]);
        let query_x = i128::from(query[0]);
        let query_y = i128::from(query[1]);
        let cross =
            (end_x - start_x) * (query_y - start_y) - (end_y - start_y) * (query_x - start_x);
        if cross == 0 && inclusive(query_x, start_x, end_x) && inclusive(query_y, start_y, end_y) {
            return true;
        }
        if start_y <= query_y && query_y < end_y && cross > 0 {
            winding += 1;
        } else if end_y <= query_y && query_y < start_y && cross < 0 {
            winding -= 1;
        }
    }
    match rule {
        Rule::NonZero => winding != 0,
        Rule::EvenOdd => winding % 2 != 0,
    }
}

fn inclusive(value: i128, first: i128, second: i128) -> bool {
    value >= first.min(second) && value <= first.max(second)
}

fn segment_stroke(start: Point, end: Point, width: i64, query: Point) -> bool {
    let start_x = i128::from(start[0]);
    let start_y = i128::from(start[1]);
    let dx = i128::from(end[0]) - start_x;
    let dy = i128::from(end[1]) - start_y;
    if dx == 0 && dy == 0 {
        return disk_contains(start_x, start_y, i128::from(width), query);
    }
    let query_dx = i128::from(query[0]) - start_x;
    let query_dy = i128::from(query[1]) - start_y;
    let dot = query_dx * dx + query_dy * dy;
    let length_squared = dx * dx + dy * dy;
    let parameter =
        round_ratio(dot * i128::from(SCALE), length_squared).clamp(0, i128::from(SCALE));
    let closest_x = start_x + round_ratio(dx * parameter, i128::from(SCALE));
    let closest_y = start_y + round_ratio(dy * parameter, i128::from(SCALE));
    disk_contains(closest_x, closest_y, i128::from(width), query)
}

fn disk_contains(center_x: i128, center_y: i128, width: i128, query: Point) -> bool {
    let dx = i128::from(query[0]) - center_x;
    let dy = i128::from(query[1]) - center_y;
    4 * (dx * dx + dy * dy) <= width * width
}

pub fn flatten_fixture_path() -> Vec<Point> {
    let mut points = vec![[0, 0], [10 * SCALE, 0]];
    flatten_quadratic(
        [10 * SCALE, 0],
        [15 * SCALE, 5 * SCALE],
        [10 * SCALE, 10 * SCALE],
        &mut points,
    );
    flatten_cubic(
        [10 * SCALE, 10 * SCALE],
        [5 * SCALE, 15 * SCALE],
        [0, 15 * SCALE],
        [0, 10 * SCALE],
        &mut points,
    );
    points.push([0, 0]);
    points
}

fn flatten_quadratic(start: Point, control: Point, end: Point, output: &mut Vec<Point>) {
    if flat(start, end, &[control]) {
        output.push(end);
        return;
    }
    let first = midpoint(start, control);
    let second = midpoint(control, end);
    let split = midpoint(first, second);
    flatten_quadratic(start, first, split, output);
    flatten_quadratic(split, second, end, output);
}

fn flatten_cubic(start: Point, first: Point, second: Point, end: Point, output: &mut Vec<Point>) {
    if flat(start, end, &[first, second]) {
        output.push(end);
        return;
    }
    let start_first = midpoint(start, first);
    let first_second = midpoint(first, second);
    let second_end = midpoint(second, end);
    let left_second = midpoint(start_first, first_second);
    let right_first = midpoint(first_second, second_end);
    let split = midpoint(left_second, right_first);
    flatten_cubic(start, start_first, left_second, split, output);
    flatten_cubic(split, right_first, second_end, end, output);
}

fn flat(start: Point, end: Point, controls: &[Point]) -> bool {
    controls.iter().all(|control| {
        let chord_x = i128::from(end[0] - start[0]);
        let chord_y = i128::from(end[1] - start[1]);
        let control_x = i128::from(control[0] - start[0]);
        let control_y = i128::from(control[1] - start[1]);
        let cross = (control_x * chord_y - control_y * chord_x).abs();
        let extent = chord_x.abs().max(chord_y.abs());
        cross <= FLATNESS * extent
            && expanded(control[0], start[0], end[0])
            && expanded(control[1], start[1], end[1])
    })
}

fn expanded(value: i64, start: i64, end: i64) -> bool {
    i128::from(value) >= i128::from(start.min(end)) - FLATNESS
        && i128::from(value) <= i128::from(start.max(end)) + FLATNESS
}

fn midpoint(left: Point, right: Point) -> Point {
    [
        i64::try_from(round_ratio(i128::from(left[0]) + i128::from(right[0]), 2))
            .expect("midpoint should fit"),
        i64::try_from(round_ratio(i128::from(left[1]) + i128::from(right[1]), 2))
            .expect("midpoint should fit"),
    ]
}
