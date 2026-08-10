use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutConstraintFieldV1, LayoutDimensionV1, LayoutExtentV1, LayoutNodeKeyV1,
    LayoutNodeV1, LayoutOutputFieldV1, LayoutPaddingSideV1, LayoutPaddingV1, LayoutRecordV1,
    LayoutRectV1, LayoutStyleV1, LayoutViewportV1,
};

use super::{DirectArtifactV1, DirectClassificationV1, DirectOutputV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DirectLaneV1 {
    Oracle,
    Reference,
    Candidate,
}

impl DirectLaneV1 {
    const ALL: [Self; 3] = [Self::Oracle, Self::Reference, Self::Candidate];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DirectInputFieldV1 {
    Key,
    Parent,
    Axis,
    Constraint(LayoutExtentV1, LayoutConstraintFieldV1),
    Padding(LayoutPaddingSideV1),
    Gap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DirectOutputFieldV1 {
    Key,
    Scalar(LayoutOutputFieldV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DirectFaultV1 {
    CaseName,
    Viewport(LayoutExtentV1),
    InputOrder,
    Input(DirectInputFieldV1),
    OutputOrder(DirectLaneV1),
    Output(DirectLaneV1, DirectOutputFieldV1),
    Classification,
}

pub(super) fn direct_faults() -> Vec<DirectFaultV1> {
    let mut faults = vec![DirectFaultV1::CaseName];
    faults.extend(LayoutExtentV1::ALL.map(DirectFaultV1::Viewport));
    faults.push(DirectFaultV1::InputOrder);
    faults.extend(
        [
            DirectInputFieldV1::Key,
            DirectInputFieldV1::Parent,
            DirectInputFieldV1::Axis,
        ]
        .map(DirectFaultV1::Input),
    );
    for extent in LayoutExtentV1::ALL {
        faults.extend(
            LayoutConstraintFieldV1::ALL
                .map(|field| DirectFaultV1::Input(DirectInputFieldV1::Constraint(extent, field))),
        );
    }
    faults.extend(
        LayoutPaddingSideV1::ALL
            .map(|side| DirectFaultV1::Input(DirectInputFieldV1::Padding(side))),
    );
    faults.push(DirectFaultV1::Input(DirectInputFieldV1::Gap));
    for lane in DirectLaneV1::ALL {
        faults.push(DirectFaultV1::OutputOrder(lane));
        faults.push(DirectFaultV1::Output(lane, DirectOutputFieldV1::Key));
        faults.extend(
            LayoutOutputFieldV1::ALL
                .map(|field| DirectFaultV1::Output(lane, DirectOutputFieldV1::Scalar(field))),
        );
    }
    faults.push(DirectFaultV1::Classification);
    assert_eq!(faults.len(), 37);
    faults
}

pub(super) fn apply_fault(model: &mut DirectArtifactV1, fault: DirectFaultV1) {
    let case = &mut model.cases[1];
    match fault {
        DirectFaultV1::CaseName => case.name.push_str("-fault"),
        DirectFaultV1::Viewport(extent) => case.viewport = mutate_viewport(case.viewport, extent),
        DirectFaultV1::InputOrder => case.input.swap(1, 2),
        DirectFaultV1::Input(field) => case.input[1] = mutate_node(case.input[1], field),
        DirectFaultV1::OutputOrder(lane) => swap_lane(&mut case.outputs, lane),
        DirectFaultV1::Output(lane, field) => {
            let record = case.outputs[1].record(lane);
            case.outputs[1].set_record(lane, mutate_record(record, field));
        }
        DirectFaultV1::Classification => case.classification = DirectClassificationV1::Adapt,
    }
}

impl DirectOutputV1 {
    fn record(self, lane: DirectLaneV1) -> LayoutRecordV1 {
        match lane {
            DirectLaneV1::Oracle => self.oracle,
            DirectLaneV1::Reference => self.reference,
            DirectLaneV1::Candidate => self.candidate,
        }
    }

    fn set_record(&mut self, lane: DirectLaneV1, record: LayoutRecordV1) {
        match lane {
            DirectLaneV1::Oracle => self.oracle = record,
            DirectLaneV1::Reference => self.reference = record,
            DirectLaneV1::Candidate => self.candidate = record,
        }
    }
}

fn swap_lane(outputs: &mut [DirectOutputV1], lane: DirectLaneV1) {
    let first = outputs[1].record(lane);
    let second = outputs[2].record(lane);
    outputs[1].set_record(lane, second);
    outputs[2].set_record(lane, first);
}

fn mutate_viewport(viewport: LayoutViewportV1, extent: LayoutExtentV1) -> LayoutViewportV1 {
    match extent {
        LayoutExtentV1::Width => LayoutViewportV1::new(viewport.width() + 1, viewport.height()),
        LayoutExtentV1::Height => LayoutViewportV1::new(viewport.width(), viewport.height() + 1),
    }
}

fn mutate_node(node: LayoutNodeV1, field: DirectInputFieldV1) -> LayoutNodeV1 {
    match field {
        DirectInputFieldV1::Key => LayoutNodeV1::new(
            LayoutNodeKeyV1::new(node.key().get() + 100),
            node.parent(),
            node.style(),
        ),
        DirectInputFieldV1::Parent => LayoutNodeV1::new(node.key(), None, node.style()),
        DirectInputFieldV1::Axis => with_style(
            node,
            LayoutStyleV1::new(
                toggle_axis(node.style().axis()),
                node.style().width(),
                node.style().height(),
                node.style().padding(),
                node.style().gap(),
            ),
        ),
        DirectInputFieldV1::Constraint(extent, constraint) => {
            let style = node.style();
            let changed = mutate_dimension(
                match extent {
                    LayoutExtentV1::Width => style.width(),
                    LayoutExtentV1::Height => style.height(),
                },
                constraint,
            );
            with_style(
                node,
                LayoutStyleV1::new(
                    style.axis(),
                    if extent == LayoutExtentV1::Width {
                        changed
                    } else {
                        style.width()
                    },
                    if extent == LayoutExtentV1::Height {
                        changed
                    } else {
                        style.height()
                    },
                    style.padding(),
                    style.gap(),
                ),
            )
        }
        DirectInputFieldV1::Padding(side) => {
            let style = node.style();
            with_style(
                node,
                LayoutStyleV1::new(
                    style.axis(),
                    style.width(),
                    style.height(),
                    mutate_padding(style.padding(), side),
                    style.gap(),
                ),
            )
        }
        DirectInputFieldV1::Gap => {
            let style = node.style();
            with_style(
                node,
                LayoutStyleV1::new(
                    style.axis(),
                    style.width(),
                    style.height(),
                    style.padding(),
                    style.gap() + 1,
                ),
            )
        }
    }
}

fn with_style(node: LayoutNodeV1, style: LayoutStyleV1) -> LayoutNodeV1 {
    LayoutNodeV1::new(node.key(), node.parent(), style)
}

fn toggle_axis(axis: LayoutAxisV1) -> LayoutAxisV1 {
    match axis {
        LayoutAxisV1::Row => LayoutAxisV1::Column,
        LayoutAxisV1::Column => LayoutAxisV1::Row,
    }
}

fn mutate_dimension(value: LayoutDimensionV1, field: LayoutConstraintFieldV1) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(
        value.minimum() + i32::from(field == LayoutConstraintFieldV1::Minimum),
        value.preferred() + i32::from(field == LayoutConstraintFieldV1::Preferred),
        value.maximum() + i32::from(field == LayoutConstraintFieldV1::Maximum),
    )
}

fn mutate_padding(value: LayoutPaddingV1, side: LayoutPaddingSideV1) -> LayoutPaddingV1 {
    LayoutPaddingV1::new(
        value.left() + i32::from(side == LayoutPaddingSideV1::Left),
        value.right() + i32::from(side == LayoutPaddingSideV1::Right),
        value.top() + i32::from(side == LayoutPaddingSideV1::Top),
        value.bottom() + i32::from(side == LayoutPaddingSideV1::Bottom),
    )
}

fn mutate_record(record: LayoutRecordV1, field: DirectOutputFieldV1) -> LayoutRecordV1 {
    if field == DirectOutputFieldV1::Key {
        return LayoutRecordV1::new(
            LayoutNodeKeyV1::new(record.key().get() + 100),
            record.bounds(),
        );
    }
    let bounds = record.bounds();
    let DirectOutputFieldV1::Scalar(scalar) = field else {
        unreachable!();
    };
    LayoutRecordV1::new(
        record.key(),
        LayoutRectV1::new(
            bounds.x() + i32::from(scalar == LayoutOutputFieldV1::X),
            bounds.y() + i32::from(scalar == LayoutOutputFieldV1::Y),
            bounds.width() + i32::from(scalar == LayoutOutputFieldV1::Width),
            bounds.height() + i32::from(scalar == LayoutOutputFieldV1::Height),
        ),
    )
}
