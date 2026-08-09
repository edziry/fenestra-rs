use super::support::{CANONICAL_TRACE_EVENT, canonical_structural_artifact, replace_once};

pub(super) const TRACE_BYTES_LIMIT: usize = 65_536;
pub(super) const TRACE_EVENTS_LIMIT: usize = 64;
const LINE_BYTES_LIMIT: usize = 1_024;

pub(super) fn with_trace(events: &[String]) -> Vec<u8> {
    let mut body = String::new();
    for event in events {
        body.push_str(event);
        body.push('\n');
    }
    let before = format!("trace-begin|1|59\n{CANONICAL_TRACE_EVENT}\ntrace-end");
    let after = format!(
        "trace-begin|{}|{}\n{body}trace-end",
        events.len(),
        body.len()
    );
    replace_once(canonical_structural_artifact(), &before, &after)
}

pub(super) fn trace_body_over_bytes_limit() -> Vec<String> {
    let mut events = vec![padded_event(LINE_BYTES_LIMIT); TRACE_EVENTS_LIMIT - 1];
    let final_line = TRACE_BYTES_LIMIT + 1 - (TRACE_EVENTS_LIMIT - 1) * (LINE_BYTES_LIMIT + 1) - 1;
    events.push(padded_event(final_line));
    events
}

pub(super) fn trace_bytes(events: &[String]) -> usize {
    events.iter().map(|event| event.len() + 1).sum()
}

fn padded_event(line_bytes: usize) -> String {
    const PREFIX: &str = "event|0|2|";
    const SUFFIX: &str = "|0|1|commit|1|-|mismatch";
    let operation_bytes = line_bytes - PREFIX.len() - SUFFIX.len();
    let mut operations = if operation_bytes.is_multiple_of(2) {
        String::from("10")
    } else {
        String::from("0")
    };
    while operations.len() < operation_bytes {
        operations.push_str(",0");
    }
    assert_eq!(operations.len(), operation_bytes);
    format!("{PREFIX}{operations}{SUFFIX}")
}
