use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use fenestra_ui_authoring::view::{Limits, build_file, compile_fen};

const CHILD: &str = "FENESTRA_VIEW_BUILD_CHILD";
static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn build_helper_child() {
    if env::var_os(CHILD).is_none() {
        return;
    }
    match build_file("fixture.fen", "view.rs") {
        Ok(()) => println!("build-result:success"),
        Err(error) => println!("build-result:error:{error}"),
    }
}

#[test]
fn build_helper_generates_the_same_expression_and_reports_changes() {
    let source = b"format 3; view hello { column root { rect child {} } }";
    let directory = fixture("success");
    fs::write(directory.join("fixture.fen"), source).unwrap();
    let output = run(&directory, true);
    assert!(output.contains("cargo::rerun-if-changed="), "{output}");
    assert!(output.contains("fixture.fen"), "{output}");
    assert!(output.contains("build-result:success"), "{output}");
    assert_eq!(
        fs::read_to_string(directory.join("view.rs")).unwrap(),
        compile_fen(source).unwrap().rust_source(),
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn build_helper_reports_file_line_byte_column_and_preserves_previous_output() {
    let directory = fixture("diagnostic");
    let source = "// caf\u{e9}\r\nformat 3;\r\nview hello { rect root {\r\n    width: -1;\r\n} }";
    fs::write(directory.join("fixture.fen"), source).unwrap();
    fs::write(directory.join("view.rs"), "old-generated-output").unwrap();
    let output = run(&directory, true);
    assert!(
        output.contains("fixture.fen:4:12: error: expected a nonnegative integer"),
        "{output}"
    );
    assert_eq!(
        fs::read_to_string(directory.join("view.rs")).unwrap(),
        "old-generated-output"
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn build_helper_bounds_reads_and_reports_missing_files_and_cargo_environment() {
    let directory = fixture("failures");
    let missing = run(&directory, true);
    assert!(
        missing.contains("fixture.fen: cannot read source:"),
        "{missing}"
    );
    fs::write(
        directory.join("fixture.fen"),
        vec![b' '; Limits::default().source_bytes() + 100],
    )
    .unwrap();
    let oversized = run(&directory, true);
    assert!(
        oversized.contains("authoring limit exceeded: source bytes"),
        "{oversized}"
    );
    let environment = run(&directory, false);
    assert!(
        environment.contains("missing Cargo environment: OUT_DIR"),
        "{environment}"
    );
    assert!(!directory.join("view.rs").exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn build_helper_rejects_output_paths_before_using_the_environment() {
    for name in [
        "",
        ".",
        "..",
        "../view.rs",
        "nested/view.rs",
        "nested\\view.rs",
        "/view.rs",
        "view.rs\n",
    ] {
        let error = build_file("fixture.fen", name).expect_err("one filename only");
        assert_eq!(error.to_string(), "output_name must be one file name");
    }
}

fn fixture(label: &str) -> std::path::PathBuf {
    let index = NEXT.fetch_add(1, Ordering::Relaxed);
    let directory = env::temp_dir().join(format!(
        "fenestra-view-build-{}-{label}-{index}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap();
    directory
}

fn run(directory: &Path, output_environment: bool) -> String {
    let mut command = Command::new(env::current_exe().unwrap());
    command
        .args(["--exact", "build_helper_child", "--nocapture"])
        .env(CHILD, "1")
        .env("CARGO_MANIFEST_DIR", directory);
    if output_environment {
        command.env("OUT_DIR", directory);
    } else {
        command.env_remove("OUT_DIR");
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
