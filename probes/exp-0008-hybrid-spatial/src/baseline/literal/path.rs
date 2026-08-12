use crate::baseline::literal_types::{PathInputV2, PathVerbInputV2, PointV2};

use super::numeric::round_ratio;
use super::types::{FlatPath, Subpath};

const FLATNESS: i128 = 256;

pub(super) fn flatten(value: &PathInputV2) -> FlatPath {
    let mut output = FlatPath {
        points: Vec::new(),
        subpaths: Vec::new(),
    };
    let mut active = None;
    let mut current = None;
    for verb in &value.verbs {
        match *verb {
            PathVerbInputV2::Move(to) => {
                finish(&mut output, active.take(), false);
                active = Some(output.points.len());
                output.points.push(to);
                current = Some(to);
            }
            PathVerbInputV2::Line(to) => {
                output.points.push(to);
                current = Some(to);
            }
            PathVerbInputV2::Quadratic(control, to) => {
                quadratic(
                    current.expect("registered path has an active subpath"),
                    control,
                    to,
                    0,
                    &mut output.points,
                );
                current = Some(to);
            }
            PathVerbInputV2::Cubic(first, second, to) => {
                cubic(
                    current.expect("registered path has an active subpath"),
                    first,
                    second,
                    to,
                    0,
                    &mut output.points,
                );
                current = Some(to);
            }
            PathVerbInputV2::Close => {
                let start = active.expect("registered path has an active subpath");
                output.points.push(output.points[start]);
                finish(&mut output, active.take(), true);
                current = None;
            }
        }
    }
    finish(&mut output, active, false);
    output
}

fn finish(path: &mut FlatPath, start: Option<usize>, closed: bool) {
    if let Some(start) = start {
        path.subpaths.push(Subpath {
            start,
            length: path.points.len() - start,
            closed,
        });
    }
}

fn quadratic(start: PointV2, control: PointV2, end: PointV2, depth: u8, out: &mut Vec<PointV2>) {
    if flat(start, end, &[control]) {
        out.push(end);
        return;
    }
    assert!(depth < 16, "registered quadratic should flatten");
    let first = midpoint(start, control);
    let second = midpoint(control, end);
    let split = midpoint(first, second);
    quadratic(start, first, split, depth + 1, out);
    quadratic(split, second, end, depth + 1, out);
}

fn cubic(
    start: PointV2,
    first: PointV2,
    second: PointV2,
    end: PointV2,
    depth: u8,
    out: &mut Vec<PointV2>,
) {
    if flat(start, end, &[first, second]) {
        out.push(end);
        return;
    }
    assert!(depth < 16, "registered cubic should flatten");
    let start_first = midpoint(start, first);
    let first_second = midpoint(first, second);
    let second_end = midpoint(second, end);
    let left_second = midpoint(start_first, first_second);
    let right_first = midpoint(first_second, second_end);
    let split = midpoint(left_second, right_first);
    cubic(start, start_first, left_second, split, depth + 1, out);
    cubic(split, right_first, second_end, end, depth + 1, out);
}

fn flat(start: PointV2, end: PointV2, controls: &[PointV2]) -> bool {
    controls.iter().copied().all(|control| {
        let chord_x = end.x as i128 - start.x as i128;
        let chord_y = end.y as i128 - start.y as i128;
        let control_x = control.x as i128 - start.x as i128;
        let control_y = control.y as i128 - start.y as i128;
        let cross = (control_x * chord_y - control_y * chord_x).abs();
        let extent = chord_x.abs().max(chord_y.abs());
        cross <= FLATNESS * extent
            && in_range(control.x, start.x, end.x)
            && in_range(control.y, start.y, end.y)
    })
}

fn in_range(value: i64, start: i64, end: i64) -> bool {
    value as i128 >= start.min(end) as i128 - FLATNESS
        && value as i128 <= start.max(end) as i128 + FLATNESS
}

fn midpoint(left: PointV2, right: PointV2) -> PointV2 {
    PointV2 {
        x: round_ratio(left.x as i128 + right.x as i128, 2) as i64,
        y: round_ratio(left.y as i128 + right.y as i128, 2) as i64,
    }
}
