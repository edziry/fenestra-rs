use fenestra_ui_layout::prototype::LayoutAxisV1::{Column, Row};

use super::support::{
    ExpectedCaseV1, dimension, expected_case, fixed, fixed_node, node, padding, record, viewport,
};

pub fn cases() -> Vec<ExpectedCaseV1> {
    vec![
        expected_case(
            "single-fixed-root",
            viewport(7, 5),
            vec![fixed_node(0, None, Column, 31, 19)],
            vec![record(0, 0, 0, 31, 19)],
        ),
        expected_case(
            "column-two",
            viewport(100, 100),
            vec![
                fixed_node(0, None, Column, 80, 60),
                fixed_node(1, Some(0), Column, 20, 11),
                fixed_node(2, Some(0), Column, 30, 13),
            ],
            vec![
                record(0, 0, 0, 80, 60),
                record(1, 0, 0, 20, 11),
                record(2, 0, 11, 30, 13),
            ],
        ),
        expected_case(
            "row-two",
            viewport(100, 100),
            vec![
                fixed_node(0, None, Row, 70, 50),
                fixed_node(1, Some(0), Column, 17, 21),
                fixed_node(2, Some(0), Column, 13, 9),
            ],
            vec![
                record(0, 0, 0, 70, 50),
                record(1, 0, 0, 17, 21),
                record(2, 17, 0, 13, 9),
            ],
        ),
        expected_case(
            "nested-row-in-column",
            viewport(100, 100),
            vec![
                fixed_node(0, None, Column, 80, 70),
                fixed_node(1, Some(0), Column, 15, 7),
                fixed_node(2, Some(0), Row, 50, 30),
                fixed_node(3, Some(2), Column, 11, 5),
                fixed_node(4, Some(2), Column, 13, 9),
                fixed_node(5, Some(0), Column, 8, 6),
            ],
            vec![
                record(0, 0, 0, 80, 70),
                record(1, 0, 0, 15, 7),
                record(2, 0, 7, 50, 30),
                record(3, 0, 7, 11, 5),
                record(4, 11, 7, 13, 9),
                record(5, 0, 37, 8, 6),
            ],
        ),
        expected_case(
            "asymmetric-padding",
            viewport(100, 100),
            vec![
                node(
                    0,
                    None,
                    Column,
                    fixed(100),
                    fixed(70),
                    padding(7, 11, 5, 13),
                    0,
                ),
                fixed_node(1, Some(0), Column, 20, 10),
                fixed_node(2, Some(0), Column, 30, 15),
            ],
            vec![
                record(0, 0, 0, 100, 70),
                record(1, 7, 5, 20, 10),
                record(2, 7, 15, 30, 15),
            ],
        ),
        expected_case(
            "column-gap-three",
            viewport(100, 100),
            vec![
                node(
                    0,
                    None,
                    Column,
                    fixed(80),
                    fixed(60),
                    padding(0, 0, 0, 0),
                    4,
                ),
                fixed_node(1, Some(0), Column, 12, 9),
                fixed_node(2, Some(0), Column, 13, 5),
                fixed_node(3, Some(0), Column, 14, 7),
            ],
            vec![
                record(0, 0, 0, 80, 60),
                record(1, 0, 0, 12, 9),
                record(2, 0, 13, 13, 5),
                record(3, 0, 22, 14, 7),
            ],
        ),
        expected_case(
            "row-gap-three",
            viewport(100, 100),
            vec![
                node(0, None, Row, fixed(80), fixed(30), padding(0, 0, 0, 0), 4),
                fixed_node(1, Some(0), Column, 9, 12),
                fixed_node(2, Some(0), Column, 5, 13),
                fixed_node(3, Some(0), Column, 7, 14),
            ],
            vec![
                record(0, 0, 0, 80, 30),
                record(1, 0, 0, 9, 12),
                record(2, 13, 0, 5, 13),
                record(3, 22, 0, 7, 14),
            ],
        ),
        expected_case(
            "clamp-below",
            viewport(100, 100),
            vec![
                fixed_node(0, None, Row, 80, 40),
                node(
                    1,
                    Some(0),
                    Column,
                    dimension(10, 3, 20),
                    dimension(7, 2, 15),
                    padding(0, 0, 0, 0),
                    0,
                ),
                fixed_node(2, Some(0), Column, 4, 5),
            ],
            vec![
                record(0, 0, 0, 80, 40),
                record(1, 0, 0, 10, 7),
                record(2, 10, 0, 4, 5),
            ],
        ),
        expected_case(
            "clamp-above",
            viewport(100, 100),
            vec![
                fixed_node(0, None, Column, 50, 100),
                node(
                    1,
                    Some(0),
                    Column,
                    dimension(10, 30, 20),
                    dimension(7, 25, 15),
                    padding(0, 0, 0, 0),
                    0,
                ),
                fixed_node(2, Some(0), Column, 5, 4),
            ],
            vec![
                record(0, 0, 0, 50, 100),
                record(1, 0, 0, 20, 15),
                record(2, 0, 15, 5, 4),
            ],
        ),
        mixed_constraints_case(),
        expected_case(
            "main-axis-overflow",
            viewport(20, 20),
            vec![
                fixed_node(0, None, Column, 20, 20),
                fixed_node(1, Some(0), Column, 8, 25),
                fixed_node(2, Some(0), Column, 5, 4),
            ],
            vec![
                record(0, 0, 0, 20, 20),
                record(1, 0, 0, 8, 25),
                record(2, 0, 25, 5, 4),
            ],
        ),
        cross_axis_overflow_case(),
    ]
}

fn mixed_constraints_case() -> ExpectedCaseV1 {
    expected_case(
        "mixed-constraints-padding",
        viewport(100, 100),
        vec![
            node(
                0,
                None,
                Column,
                dimension(50, 60, 70),
                dimension(40, 50, 60),
                padding(6, 7, 4, 5),
                3,
            ),
            node(
                1,
                Some(0),
                Column,
                dimension(12, 8, 20),
                dimension(5, 14, 10),
                padding(0, 0, 0, 0),
                0,
            ),
            node(
                2,
                Some(0),
                Column,
                dimension(5, 18, 16),
                dimension(6, 3, 12),
                padding(0, 0, 0, 0),
                0,
            ),
        ],
        vec![
            record(0, 0, 0, 60, 50),
            record(1, 6, 4, 12, 10),
            record(2, 6, 17, 16, 6),
        ],
    )
}

fn cross_axis_overflow_case() -> ExpectedCaseV1 {
    expected_case(
        "cross-axis-overflow",
        viewport(20, 30),
        vec![
            node(
                0,
                None,
                Column,
                fixed(20),
                fixed(30),
                padding(3, 4, 2, 1),
                0,
            ),
            fixed_node(1, Some(0), Column, 25, 6),
            fixed_node(2, Some(0), Column, 10, 5),
        ],
        vec![
            record(0, 0, 0, 20, 30),
            record(1, 3, 2, 25, 6),
            record(2, 3, 8, 10, 5),
        ],
    )
}
