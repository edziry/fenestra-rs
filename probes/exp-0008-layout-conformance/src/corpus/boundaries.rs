use fenestra_ui_layout::prototype::LayoutAxisV1::{Column, Row};

use super::RegisteredLayoutCaseV1;
use super::support::{fixed, fixed_node, node, padding, record, viewport};

pub(super) fn cases() -> Vec<RegisteredLayoutCaseV1> {
    vec![
        RegisteredLayoutCaseV1::new(
            "padding-equal-box",
            viewport(10, 8),
            vec![
                node(0, None, Column, fixed(10), fixed(8), padding(4, 6, 8, 0), 0),
                fixed_node(1, Some(0), Column, 3, 2),
            ],
            vec![record(0, 0, 0, 10, 8), record(1, 4, 8, 3, 2)],
        ),
        RegisteredLayoutCaseV1::new(
            "zero-width-child",
            viewport(30, 20),
            vec![
                fixed_node(0, None, Row, 30, 20),
                fixed_node(1, Some(0), Column, 7, 5),
                fixed_node(2, Some(0), Column, 0, 6),
                fixed_node(3, Some(0), Column, 5, 4),
            ],
            vec![
                record(0, 0, 0, 30, 20),
                record(1, 0, 0, 7, 5),
                record(2, 7, 0, 0, 6),
                record(3, 7, 0, 5, 4),
            ],
        ),
        RegisteredLayoutCaseV1::new(
            "zero-height-gap",
            viewport(30, 30),
            vec![
                node(
                    0,
                    None,
                    Column,
                    fixed(30),
                    fixed(30),
                    padding(0, 0, 0, 0),
                    3,
                ),
                fixed_node(1, Some(0), Column, 7, 5),
                fixed_node(2, Some(0), Column, 8, 0),
                fixed_node(3, Some(0), Column, 9, 4),
            ],
            vec![
                record(0, 0, 0, 30, 30),
                record(1, 0, 0, 7, 5),
                record(2, 0, 8, 8, 0),
                record(3, 0, 11, 9, 4),
            ],
        ),
        RegisteredLayoutCaseV1::new(
            "zero-width-viewport",
            viewport(0, 5),
            vec![
                fixed_node(0, None, Column, 12, 9),
                fixed_node(1, Some(0), Column, 4, 3),
            ],
            vec![record(0, 0, 0, 12, 9), record(1, 0, 0, 4, 3)],
        ),
        RegisteredLayoutCaseV1::new(
            "zero-height-viewport",
            viewport(7, 0),
            vec![
                fixed_node(0, None, Column, 12, 9),
                fixed_node(1, Some(0), Column, 4, 3),
            ],
            vec![record(0, 0, 0, 12, 9), record(1, 0, 0, 4, 3)],
        ),
        RegisteredLayoutCaseV1::new(
            "zero-by-zero-viewport",
            viewport(0, 0),
            vec![
                fixed_node(0, None, Column, 12, 9),
                fixed_node(1, Some(0), Column, 4, 3),
            ],
            vec![record(0, 0, 0, 12, 9), record(1, 0, 0, 4, 3)],
        ),
        RegisteredLayoutCaseV1::new(
            "large-integer-padding-gap",
            viewport(4096, 4096),
            vec![
                node(
                    0,
                    None,
                    Column,
                    fixed(4096),
                    fixed(4096),
                    padding(4096, 0, 4096, 0),
                    4096,
                ),
                fixed_node(1, Some(0), Column, 0, 0),
                fixed_node(2, Some(0), Column, 0, 0),
            ],
            vec![
                record(0, 0, 0, 4096, 4096),
                record(1, 4096, 4096, 0, 0),
                record(2, 4096, 8192, 0, 0),
            ],
        ),
    ]
}
