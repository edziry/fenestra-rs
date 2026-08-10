const GOLDEN: &[u8] = include_bytes!("artifacts/fedora-wayland-v1.txt");

#[test]
fn fedora_wayland_artifact_is_bounded_canonical_and_terminal() {
    assert_eq!(GOLDEN.len(), 12_975);
    assert_eq!(GOLDEN.last(), Some(&b'\n'));
    assert!(
        GOLDEN
            .iter()
            .all(|byte| *byte == b'\n' || (0x20..=0x7e).contains(byte))
    );

    let text = std::str::from_utf8(GOLDEN).expect("artifact must be printable ASCII");
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines.len(), 37);
    assert_eq!(lines.iter().map(|line| line.len()).max(), Some(389));
    assert_eq!(lines[0], "fenestra-native-artifact|1");
    assert_eq!(
        lines[1],
        "manifest|os=linux|target=x86_64-unknown-linux-gnu|window=wayland|winit=0.30.13|winit_features=rwh_06,wayland,wayland-dlopen|softbuffer=0.4.8|softbuffer_features=wayland,wayland-dlopen|physical=400x300|logical=320x240|scale_micros=1250000|requested=31|detected=31|effective=31"
    );

    let events = &lines[2..36];
    assert_eq!(events.len(), 34);
    for (sequence, event) in events.iter().enumerate() {
        assert!(event.starts_with(&format!("event|sequence={sequence}|")));
        assert!(event.ends_with("|accounted_bytes=192"));
    }

    let digests: Vec<_> = events
        .iter()
        .filter(|event| !event.contains("|digest=-|"))
        .copied()
        .collect();
    assert_eq!(digests.len(), 2);
    assert!(digests[0].contains("|sequence=11|"));
    assert!(digests[0].contains("|outcome=accepted|"));
    assert!(digests[0].contains("|frame=0|submission=0:0|"));
    assert!(digests[0].contains("|digest=99e3b9d20c7b7cbd|"));
    assert!(digests[1].contains("|sequence=25|"));
    assert!(digests[1].contains("|outcome=accepted|"));
    assert!(digests[1].contains("|frame=1|submission=0:1|"));
    assert!(digests[1].contains("|digest=0471ed0561dc931d|"));

    assert_eq!(
        lines[36],
        "terminal|result=pass|generation=2|scheduler=stopped|deferred=0:0|controls=0:0|visual=0:0|in_flight=0:0|redraw=0|pending=0:0:0"
    );
    for forbidden in ["/home/", "\\\\", "hostname=", "username=", "display="] {
        assert!(!text.to_ascii_lowercase().contains(forbidden));
    }
}
