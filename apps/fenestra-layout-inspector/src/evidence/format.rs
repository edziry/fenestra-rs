use std::fmt::Write as _;

use crate::InspectorFrame;

pub(super) fn viewport(frame: &InspectorFrame) -> (i32, i32) {
    (frame.viewport().width(), frame.viewport().height())
}

pub(super) fn flag(value: bool) -> u8 {
    u8::from(value)
}

pub(super) fn keys(values: &[u64]) -> String {
    let mut output = String::new();
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        write!(output, "{value}").expect("writing to a string cannot fail");
    }
    output
}
