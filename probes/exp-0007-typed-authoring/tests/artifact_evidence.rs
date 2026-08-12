use std::collections::BTreeSet;

const EVIDENCE: &str = include_str!("artifacts/layout-board-evidence-v1.txt");

struct FileEvidenceV1 {
    name: &'static str,
    path: &'static str,
    bytes: &'static [u8],
    sha256: &'static str,
}

const FILES: [FileEvidenceV1; 11] = [
    file(
        "fen-fixture",
        "fixtures/layout-board.fen",
        include_bytes!("../fixtures/layout-board.fen"),
        "8308bb19961812d8ec793469c3783c92e63489bbbcbc0e143d24dcac1c983ec6",
    ),
    file(
        "ui-fixture",
        "fixtures/layout-board.ui",
        include_bytes!("../fixtures/layout-board.ui"),
        "dd52b908919cb2eb5ddb61ecc10f43e5f43de744347652700700137f0ba3e2b8",
    ),
    file(
        "generated-rust",
        "tests/artifacts/layout-board-generated-v1.rs",
        include_bytes!("artifacts/layout-board-generated-v1.rs"),
        "b633d1f01c0da43827925ae245e095a02aa87a8229d870a1cbee9f2f5adf42de",
    ),
    file(
        "fen-map",
        "tests/artifacts/layout-board-fen-map-v1.txt",
        include_bytes!("artifacts/layout-board-fen-map-v1.txt"),
        "3f480eb8021dcca110e2b20bd715e7f2231cb9ea06f981b11859c5ab279a8323",
    ),
    file(
        "ui-map",
        "tests/artifacts/layout-board-ui-map-v1.txt",
        include_bytes!("artifacts/layout-board-ui-map-v1.txt"),
        "baba40d517ee79dcc102c6562bfb9d2053924feee4d815c9b1e8f8b2fd5fc224",
    ),
    file(
        "semantic",
        "tests/artifacts/layout-board-semantic-v1.txt",
        include_bytes!("artifacts/layout-board-semantic-v1.txt"),
        "013d2c66d9858db5a5bcfb0b62fb060b373a081b20f19e27ea433864445a3871",
    ),
    file(
        "runtime",
        "tests/artifacts/layout-board-runtime-v1.txt",
        include_bytes!("artifacts/layout-board-runtime-v1.txt"),
        "8e3dc45ff29ed49ee9426cb0b907e11665fb7dc3335bd25f8a0179ec5325d9fa",
    ),
    file(
        "trybuild-nesting",
        "crates/fenestra-ui-macros/tests/ui/nesting_depth.stderr",
        include_bytes!("../../../crates/fenestra-ui-macros/tests/ui/nesting_depth.stderr"),
        "218c18f4259f2a32754b3d673c66ab56f7db06508044cb4cb72874411fd8e24b",
    ),
    file(
        "trybuild-unknown-component",
        "crates/fenestra-ui-macros/tests/ui/unknown_component.stderr",
        include_bytes!("../../../crates/fenestra-ui-macros/tests/ui/unknown_component.stderr"),
        "8ba6efc1d086a98cff9775c2b51ac33e1f3280fee8fc69781623062f7a4f1b72",
    ),
    file(
        "trybuild-unsupported-token",
        "crates/fenestra-ui-macros/tests/ui/unsupported_token.stderr",
        include_bytes!("../../../crates/fenestra-ui-macros/tests/ui/unsupported_token.stderr"),
        "550f19751c832236b78259a01b86a9268b13742acb2031733a0d59f3051c53be",
    ),
    file(
        "cargo-lock",
        "Cargo.lock",
        include_bytes!("../../../Cargo.lock"),
        "343c31486110d3deda1bbfc335ef71f46ae7d4b775fe4c1face03b12ec4a027b",
    ),
];

const STATIC_ROWS: [&str; 23] = [
    "fenestra-authoring-evidence|1",
    "work-unit|WU-0010|experiment=EXP-0007|experiment-status=open",
    "package|version=0.1.0|edition=2024|publish=false",
    "formats|authoring=1|schema=1|construction=1|style=1|map=1|semantic=1|runtime=1",
    "authoring-limits|fen-source-bytes=8192|tokens=1024|identifier-bytes=32|nesting-depth=8|components=1|properties=5|templates=4|regions=1|child-slots=3|initial-properties=12|initial-keys=2|style-assignments=2|source-anchors=34|generated-rust-bytes=32768",
    "ir-limits|components=1|properties=5|templates=4|regions=1|child-slots=3|initial-properties=12|initial-keys=2|template-depth=3|initial-instances=5|style-assignments=2",
    "map-limits|artifact-bytes=4096|line-bytes=128|records=36|priority=records,line-bytes,artifact-bytes",
    "semantic-limits|artifact-bytes=8192|line-bytes=512|records=64|priority=records,line-bytes,artifact-bytes",
    "runtime-artifact-limits|artifact-bytes=32768|line-bytes=512|records=512|priority=records,line-bytes,artifact-bytes",
    "evidence-limits|artifact-bytes=8192|line-bytes=512|records=64",
    "observer-limits|transactions=16|operations-per-transaction=8|operations=128|live-memberships=5|path-depth=3|nodes=8|fragments=2|properties=40|actions=64|trace-bytes=20480",
    "semantic-counts|records=34|components=1|properties=5|templates=4|regions=1|child-slots=3|initial-properties=12|initial-keys=2|style-assignments=2",
    "runtime-counts|lanes=3|generations=6|receipts=6|mutations=5|manifests=2|nodes=33|properties=165|children=18|fragments=6|members=15|computed=33|geometry=33|semantics=6|hits=21|scene=33",
    "runtime-series|nodes=5,5,6,6,6,5|properties=25,25,30,30,30,25|members=2,2,3,3,3,2|final-keys=10,30",
    "dependency|proc-macro2|version=1.0.107|scope=host-normal,probe-dev|license=MIT-OR-Apache-2.0|rust=1.71|build-script=true",
    "dependency|quote|version=1.0.47|scope=host-normal|license=MIT-OR-Apache-2.0|rust=1.71|build-script=true",
    "dependency|unicode-ident|version=1.0.24|scope=host-transitive|license=(MIT-OR-Apache-2.0)-AND-Unicode-3.0|rust=1.71|build-script=false",
    "dependency|trybuild|version=1.0.120|scope=macro-dev-only|license=MIT-OR-Apache-2.0|rust=1.88|build-script=false",
    "graph|probe-target-no-proc-macro=fenestra-ui-ir|normal-syn=false|trybuild-dev-only=true|native-ffi=false",
    "build-script|input=fixtures/layout-board.fen|output=OUT_DIR/layout_board_fen_v1.rs|shell=false|network=false|source-tree-write=false",
    "environment|linux|os=fedora-43|kernel=7.1.5-101.fc43.x86_64|arch=x86_64|rust=1.97.1|cargo=1.97.1|llvm=22.1.6|time=gnu-1.9",
    "measurement|linux|profile=debug|case=clean|elapsed-seconds=1.27|peak-rss-kib=195320|rlib-bytes=285646",
    "measurement|linux|profile=debug|case=noop|elapsed-seconds=0.08|peak-rss-kib=48280|rlib-bytes=285646",
];

const EDIT_ROW: &str = "measurement|linux|profile=debug|case=one-source-edit|elapsed-seconds=0.11|peak-rss-kib=86264|rlib-bytes=285646|generated-rust-sha256=b633d1f01c0da43827925ae245e095a02aa87a8229d870a1cbee9f2f5adf42de";

#[test]
fn committed_evidence_summary_matches_every_versioned_input_and_artifact() {
    assert!(EVIDENCE.is_ascii());
    assert!(!EVIDENCE.contains('\r'));
    assert!(EVIDENCE.ends_with('\n'));
    assert!(!EVIDENCE.ends_with("\n\n"));
    assert!(EVIDENCE.len() <= 8_192);
    assert!(max_line_bytes(EVIDENCE.as_bytes()) <= 512);
    assert!(line_count(EVIDENCE.as_bytes()) <= 64);

    let lines = EVIDENCE.lines().collect::<Vec<_>>();
    let unique = lines.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), lines.len());
    assert_eq!(lines.len(), STATIC_ROWS.len() + FILES.len() + 2);
    for row in STATIC_ROWS {
        assert!(unique.contains(row), "missing `{row}`");
    }
    assert!(unique.contains(EDIT_ROW));
    assert_eq!(lines.last(), Some(&"end"));

    for file in &FILES {
        assert_file_bytes(file.bytes);
        assert_eq!(file.sha256.len(), 64);
        assert!(file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()));
        let row = format!(
            "file|{}|path={}|bytes={}|lines={}|max-line={}|sha256={}",
            file.name,
            file.path,
            file.bytes.len(),
            line_count(file.bytes),
            max_line_bytes(file.bytes),
            file.sha256,
        );
        assert!(unique.contains(row.as_str()), "missing `{row}`");
    }

    for forbidden in ["/home/", "C:\\", "username=", "hostname=", "timestamp="] {
        assert!(!EVIDENCE.contains(forbidden));
    }
}

const fn file(
    name: &'static str,
    path: &'static str,
    bytes: &'static [u8],
    sha256: &'static str,
) -> FileEvidenceV1 {
    FileEvidenceV1 {
        name,
        path,
        bytes,
        sha256,
    }
}

fn assert_file_bytes(bytes: &[u8]) {
    assert!(bytes.is_ascii());
    assert!(!bytes.contains(&b'\r'));
    assert_eq!(bytes.last(), Some(&b'\n'));
}

fn line_count(bytes: &[u8]) -> usize {
    bytes.iter().filter(|byte| **byte == b'\n').count()
}

fn max_line_bytes(bytes: &[u8]) -> usize {
    bytes
        .split(|byte| *byte == b'\n')
        .map(<[u8]>::len)
        .max()
        .unwrap_or(0)
}
