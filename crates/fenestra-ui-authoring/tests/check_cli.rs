use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use fenestra_ui_authoring::prototype::{
    FenSourceV2, REFERENCE_AUTHORING_LIMITS_V2, canonical_rust_v2, compile_fen_v2,
};
use fenestra_ui_ir::prototype::SourceId;

const FIXTURE: &str = "../../probes/exp-0007-typed-authoring/fixtures/hybrid-spatial-v2.fen";
static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(bytes: &[u8]) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "fenestra-check-{}-{}",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("source with spaces.fen");
        fs::write(&path, bytes).unwrap();
        Self(path)
    }
}

impl Drop for SourceFile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.0.parent().unwrap());
    }
}

fn check(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fenestra-check"))
        .arg(path)
        .output()
        .unwrap()
}

#[test]
fn valid_source_checks_and_emits_the_existing_canonical_rust() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let checked = check(&path);
    assert!(checked.status.success());
    assert!(String::from_utf8_lossy(&checked.stdout).contains("ok|format=2|"));
    let emitted = Command::new(env!("CARGO_BIN_EXE_fenestra-check"))
        .arg("--emit-rust")
        .arg(&path)
        .output()
        .unwrap();
    assert!(emitted.status.success());
    let bytes = fs::read(path).unwrap();
    let compiled = compile_fen_v2(
        FenSourceV2::new(SourceId::new(1), &bytes),
        REFERENCE_AUTHORING_LIMITS_V2,
    )
    .unwrap();
    let expected = canonical_rust_v2(&compiled, REFERENCE_AUTHORING_LIMITS_V2).unwrap();
    assert_eq!(emitted.stdout, expected.as_str().as_bytes());
}

#[test]
fn syntax_errors_report_crlf_line_and_byte_column() {
    let source = SourceFile::new(b"format 2;\r\nschema nope");
    let output = check(&source.0);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("source with spaces.fen:2:8: error: unexpected-token"),
        "{error}"
    );
    assert!(error.contains("bytes=18..22"), "{error}");
}

#[test]
fn incomplete_and_invalid_utf8_sources_have_physical_locations() {
    for (bytes, expected) in [
        (&b"format 2;\n"[..], ":2:1: error: unexpected-eof"),
        (&b"format 2;\n\xff"[..], ":2:1: error: invalid-utf8"),
    ] {
        let source = SourceFile::new(bytes);
        let output = check(&source.0);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn oversized_source_reports_the_registered_bound() {
    let source = SourceFile::new(&vec![b' '; 8193]);
    let output = check(&source.0);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("limit-exceeded(fen-source-bytes)"),
        "{error}"
    );
    assert!(error.contains("bytes=8192..8193"), "{error}");
}

#[test]
fn source_limit_reports_byte_locations_even_when_reading_splits_utf8() {
    let mut bytes = vec![b' '; 8191];
    bytes[4095] = b'\n';
    bytes.extend_from_slice("\u{20ac}".as_bytes());
    let source = SourceFile::new(&bytes);

    let output = check(&source.0);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains(":2:4097: error: limit-exceeded(fen-source-bytes)"),
        "{error}"
    );
    assert!(error.contains("bytes=8192..8193"), "{error}");
    assert!(!error.contains("invalid-utf8"), "{error}");
}

#[test]
fn unknown_component_reports_the_reference_location_and_kind() {
    let text = "format 2;
schema namespace 1 revision 1 {
  component c = 0 {
    property p = 0: scalar_i32 = 0 invalidates [layout];
  }
}
construction {
  template root = 0: missing { child region rows; }
  region rows = 0 owner root repeat root keys [] invalidates [structure];
}
style {}
spatial format 2 {
  viewport container row padding (0, 0, 0, 0) gap 0;
  resources {}
}
";
    let source = SourceFile::new(text.as_bytes());

    let output = check(&source.0);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains(":8:22: error: unknown-component-name"),
        "{error}"
    );
    let start = text.find("missing").unwrap();
    assert!(
        error.contains(&format!("bytes={start}..{}", start + "missing".len())),
        "{error}"
    );
    assert!(error.contains("kind=UnknownComponentName"), "{error}");
}

#[test]
fn expanded_instance_failures_retain_the_underlying_ir_limit() {
    let text = "format 2;
schema namespace 1 revision 1 {
  component c = 0 {
    property p = 0: scalar_i32 = 0 invalidates [layout];
  }
}
construction {
  template root = 0: c { child region rows; }
  template cell = 1: c {
    child template first;
    child template second;
    child template third;
  }
  template first = 2: c {}
  template second = 3: c {}
  template third = 4: c {}
  region rows = 0 owner root repeat cell keys [10, 20] invalidates [structure];
}
style {}
spatial format 2 {
  viewport container row padding (0, 0, 0, 0) gap 0;
  resources {}
}
";
    let source = SourceFile::new(text.as_bytes());

    let output = check(&source.0);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("error: ir-validation"), "{error}");
    assert!(
        error.contains("kind=IrValidation(LimitExceeded(InitialInstances))"),
        "{error}"
    );
}

#[test]
fn option_terminator_allows_a_source_name_beginning_with_a_dash() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let source = SourceFile::new(&fs::read(fixture).unwrap());
    let directory = source.0.parent().unwrap();
    let name = "--emit-rust";
    fs::rename(&source.0, directory.join(name)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fenestra-check"))
        .current_dir(directory)
        .args(["--", name])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("ok|format=2|"));
}

#[test]
fn help_and_argument_errors_do_not_attempt_compilation() {
    let help = Command::new(env!("CARGO_BIN_EXE_fenestra-check"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("8192"));
    for arguments in [vec![], vec!["--unknown"], vec!["a.fen", "b.fen"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_fenestra-check"))
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    }
}

#[test]
fn missing_source_reports_a_read_error() {
    let source = SourceFile::new(b"");
    fs::remove_file(&source.0).unwrap();
    let output = check(&source.0);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read source"));
}

#[test]
fn format_three_checks_and_emits_public_facade_constructors() {
    let source =
        SourceFile::new(b"/* an application */ format 3; view hello { rect panel { width: 20; } }");
    let checked = check(&source.0);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert!(String::from_utf8_lossy(&checked.stdout).contains("ok|format=3|"));
    let emitted = Command::new(env!("CARGO_BIN_EXE_fenestra-check"))
        .arg("--emit-rust")
        .arg(&source.0)
        .output()
        .unwrap();
    assert!(emitted.status.success());
    let generated = String::from_utf8(emitted.stdout).unwrap();
    assert!(generated.contains("fenestra_ui :: View"), "{generated}");
    assert!(!generated.contains("prototype"));
}

#[test]
fn format_three_syntax_failures_keep_the_correct_source_language() {
    let source =
        SourceFile::new(b"// panel\r\nformat 3;\r\nview hello { rect panel { width: nope; } }");
    let output = check(&source.0);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains(":3:"), "{error}");
    assert!(!error.contains("unsupported-authoring-format"), "{error}");
    assert!(!error.contains("unsupported-token"), "{error}");
}
