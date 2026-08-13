use fenestra_ui_exp_0014_windows_gpu::{
    ARTIFACT_LIMITS_V1, InteractiveArtifactErrorKindV1, InteractiveResultV1,
    verify_interactive_artifact_v1,
};

fn valid_pass() -> Vec<u8> {
    concat!(
        "fenestra-windows-gpu|artifact=1|probe=14\n",
        "run|target=windows-dx12|rust-target=x86_64-pc-windows-msvc|package=0.2.0|profile=release|os=windows|os-version-hex=31312e30\n",
        "adapter|backend=dx12|device-type=integrated|vendor=4098|device=5686|name-hex=414d44|driver-hex=616d64|info-hex=33312e30\n",
        "surface|format=bgra8unorm|present=fifo|alpha=opaque\n",
        "event|milestone=adapter\n",
        "event|milestone=initial-present|generation=0|frame=0|submission=0|physical=192x128|logical=192x128|raster=0123456789abcdef\n",
        "event|milestone=pointer-move\n",
        "event|milestone=pointer-press\n",
        "event|milestone=mutation-present|generation=1|frame=1|submission=1|physical=192x128|logical=192x128|raster=fedcba9876543210\n",
        "event|milestone=resize|physical=224x160|logical=224x160\n",
        "event|milestone=resize-present|generation=2|frame=2|submission=2|physical=224x160|logical=224x160|raster=0011223344556677\n",
        "event|milestone=suspend\n",
        "event|milestone=restore\n",
        "event|milestone=restore-present|generation=2|frame=3|submission=3|physical=224x160|logical=224x160|raster=0011223344556677\n",
        "event|milestone=close\n",
        "result|kind=pass|reason=complete\n",
    )
    .as_bytes()
    .to_vec()
}

#[test]
fn exact_complete_windows_artifact_verifies() {
    let bytes = valid_pass();
    let verified = verify_interactive_artifact_v1(&bytes).expect("valid pass artifact");

    assert_eq!(verified.result(), InteractiveResultV1::Pass);
    assert_eq!(verified.record_count(), 16);
    assert_eq!(verified.byte_count(), bytes.len());
    assert_eq!(verified.last_generation(), Some(2));
}

#[test]
fn artifact_bounds_are_exact_and_checked_before_grammar() {
    assert_eq!(ARTIFACT_LIMITS_V1.records(), 256);
    assert_eq!(ARTIFACT_LIMITS_V1.line_bytes(), 512);
    assert_eq!(ARTIFACT_LIMITS_V1.artifact_bytes(), 65_536);

    let mut too_many = Vec::new();
    for _ in 0..257 {
        too_many.extend_from_slice(b"x\n");
    }
    assert_eq!(
        verify_interactive_artifact_v1(&too_many).expect_err("record bound"),
        InteractiveArtifactErrorKindV1::Bounds
    );

    let mut long_line = vec![b'x'; 513];
    long_line.push(b'\n');
    assert_eq!(
        verify_interactive_artifact_v1(&long_line).expect_err("line bound"),
        InteractiveArtifactErrorKindV1::Bounds
    );
}

#[test]
fn encoding_and_privacy_failures_are_closed() {
    let mut non_ascii = valid_pass();
    non_ascii[0] = 0xff;
    assert_eq!(
        verify_interactive_artifact_v1(&non_ascii).expect_err("ASCII only"),
        InteractiveArtifactErrorKindV1::Encoding
    );

    let private = String::from_utf8(valid_pass())
        .expect("fixture ASCII")
        .replace("|profile=release", "|home=/users/alice|profile=release");
    assert_eq!(
        verify_interactive_artifact_v1(private.as_bytes()).expect_err("private key"),
        InteractiveArtifactErrorKindV1::Redaction
    );
}

#[test]
fn target_backend_coherence_is_verified() {
    let incoherent = String::from_utf8(valid_pass())
        .expect("fixture ASCII")
        .replace("adapter|backend=dx12", "adapter|backend=vulkan");
    assert_eq!(
        verify_interactive_artifact_v1(incoherent.as_bytes()).expect_err("backend mismatch"),
        InteractiveArtifactErrorKindV1::Coherence
    );
}

#[test]
fn artifact_replays_generation_and_terminal_rules() {
    let stale = String::from_utf8(valid_pass())
        .expect("fixture ASCII")
        .replace(
            "mutation-present|generation=1",
            "mutation-present|generation=0",
        );
    assert_eq!(
        verify_interactive_artifact_v1(stale.as_bytes()).expect_err("stale mutation"),
        InteractiveArtifactErrorKindV1::Protocol
    );

    let incomplete = String::from_utf8(valid_pass())
        .expect("fixture ASCII")
        .replace("event|milestone=close\n", "");
    assert_eq!(
        verify_interactive_artifact_v1(incomplete.as_bytes()).expect_err("incomplete pass"),
        InteractiveArtifactErrorKindV1::Terminal
    );
}
