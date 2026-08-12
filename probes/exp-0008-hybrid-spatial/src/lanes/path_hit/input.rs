use super::types::{
    FillRuleV2, PATH_SCALE_V2 as S, PathCoverageV2, PathHitCaseV2, PathHitObligationV2 as O,
    PathLayerV2, PathQueryV2, PathVerbV2 as V,
};

pub(crate) fn path_hit_cases_v2() -> Vec<PathHitCaseV2> {
    vec![
        case(
            0,
            "convex",
            vec![fill(square(0, 0, 10, 10), FillRuleV2::NonZero)],
            None,
            vec![query(5, 5, false), query(12, 5, false)],
            vec![O::Convex, O::Fill, O::NonZero],
        ),
        case(
            1,
            "concave",
            vec![fill(
                polygon(&[(0, 0), (10, 0), (10, 3), (3, 3), (3, 10), (0, 10)]),
                FillRuleV2::NonZero,
            )],
            None,
            vec![query(1, 8, false), query(7, 7, true)],
            vec![O::Concave, O::AabbMiss],
        ),
        case(
            2,
            "hole-evenodd",
            vec![fill(hole(), FillRuleV2::EvenOdd)],
            None,
            vec![query(1, 1, false), query(5, 5, false)],
            vec![O::Holes, O::EvenOdd],
        ),
        case(
            3,
            "hole-nonzero",
            vec![fill(hole(), FillRuleV2::NonZero)],
            None,
            vec![query(1, 1, false), query(5, 5, false)],
            vec![O::Holes, O::NonZero],
        ),
        case(
            4,
            "self-intersection",
            vec![fill(
                polygon(&[(0, 0), (10, 10), (0, 10), (10, 0)]),
                FillRuleV2::EvenOdd,
            )],
            None,
            vec![query(2, 1, false), query(2, 9, false), query(5, 5, false)],
            vec![O::SelfIntersection],
        ),
        case(
            5,
            "degenerate",
            vec![fill(
                polygon(&[(0, 0), (10, 0), (10, 0), (10, 10), (0, 10)]),
                FillRuleV2::NonZero,
            )],
            None,
            vec![query(5, 5, false), query(12, 5, false)],
            vec![O::Degenerate],
        ),
        case(
            6,
            "quadratic",
            vec![fill(
                vec![
                    V::Move(point(0, 0)),
                    V::Quadratic(point(5, 10), point(10, 0)),
                    V::Line(point(0, 0)),
                    V::Close,
                ],
                FillRuleV2::NonZero,
            )],
            None,
            vec![query(5, 2, false), query(5, 7, true)],
            vec![O::Quadratic],
        ),
        case(
            7,
            "cubic",
            vec![fill(
                vec![
                    V::Move(point(0, 0)),
                    V::Cubic(point(0, 10), point(10, 10), point(10, 0)),
                    V::Line(point(0, 0)),
                    V::Close,
                ],
                FillRuleV2::NonZero,
            )],
            None,
            vec![query(5, 3, false), query(5, 9, true)],
            vec![O::Cubic],
        ),
        case(
            8,
            "round-stroke",
            vec![PathLayerV2 {
                verbs: vec![V::Move(point(0, 5)), V::Line(point(10, 5))],
                coverage: PathCoverageV2::RoundStroke { width: 2 * S },
            }],
            None,
            vec![query(5, 5, false), query(0, 5, false), query(5, 8, false)],
            vec![O::RoundStroke],
        ),
        case(
            9,
            "explicit-clip",
            vec![fill(square(0, 0, 10, 10), FillRuleV2::NonZero)],
            Some(fill(
                polygon(&[(0, 0), (10, 0), (0, 10)]),
                FillRuleV2::NonZero,
            )),
            vec![query(2, 2, false), query(8, 8, false)],
            vec![O::Clip],
        ),
        case(
            10,
            "reverse-painter",
            vec![
                fill(square(0, 0, 10, 10), FillRuleV2::NonZero),
                fill(square(5, 0, 15, 10), FillRuleV2::NonZero),
            ],
            None,
            vec![query(2, 5, false), query(7, 5, false), query(13, 5, false)],
            vec![O::ReversePainter],
        ),
    ]
}

fn case(
    ordinal: u8,
    name: &'static str,
    layers: Vec<PathLayerV2>,
    clip: Option<PathLayerV2>,
    queries: Vec<PathQueryV2>,
    obligations: Vec<O>,
) -> PathHitCaseV2 {
    PathHitCaseV2 {
        ordinal,
        name,
        layers,
        clip,
        queries,
        obligations,
    }
}

fn fill(verbs: Vec<V>, rule: FillRuleV2) -> PathLayerV2 {
    PathLayerV2 {
        verbs,
        coverage: PathCoverageV2::Fill(rule),
    }
}

fn square(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<V> {
    polygon(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
}

fn hole() -> Vec<V> {
    let mut verbs = square(0, 0, 10, 10);
    verbs.extend(square(3, 3, 7, 7));
    verbs
}

fn polygon(points: &[(i32, i32)]) -> Vec<V> {
    let mut verbs = vec![V::Move(point(points[0].0, points[0].1))];
    verbs.extend(points[1..].iter().map(|&(x, y)| V::Line(point(x, y))));
    verbs.push(V::Close);
    verbs
}

const fn point(x: i32, y: i32) -> [i32; 2] {
    [x * S, y * S]
}

const fn query(x: i32, y: i32, nonrectangular_aabb_miss: bool) -> PathQueryV2 {
    PathQueryV2 {
        point: point(x, y),
        nonrectangular_aabb_miss,
    }
}
