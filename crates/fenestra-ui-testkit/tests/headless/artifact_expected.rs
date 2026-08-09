#[path = "artifact_expected/events.rs"]
mod events;
#[path = "artifact_expected/header.rs"]
mod header;
#[path = "artifact_expected/projection.rs"]
mod projection;

pub(super) fn assert_canonical_artifact(encoded: &[u8]) {
    assert_eq!(encoded.last(), Some(&b'\n'));
    assert!(encoded.len() <= 65_536);
    let text = std::str::from_utf8(encoded).expect("canonical ASCII should be UTF-8");
    let lines = text.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 144);
    assert!(lines.iter().all(|line| line.len() <= 1_024));

    header::assert_header(&lines[..14]);
    events::assert_trace_sections(&lines[14..114]);
    projection::assert_projection_and_result(&lines[114..]);
}
