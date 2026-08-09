#[path = "support/mod.rs"]
mod support;

use std::process::{Command, Output};

#[test]
fn probe_stdout_is_exactly_the_golden_without_host_or_log_output() {
    let first = run_probe("host-a", "trace");
    let second = run_probe("host-b", "off");
    for output in [&first, &second] {
        assert!(output.status.success());
        assert!(
            output.stdout == support::GOLDEN,
            "probe stdout differed: observed {} bytes",
            output.stdout.len()
        );
        assert!(output.stderr.is_empty());
    }
    assert_eq!(first.stdout, second.stdout);
}

fn run_probe(host: &str, log: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fenestra-ui-exp-0001-spine"))
        .env("FENESTRA_TEST_HOST_SENTINEL", host)
        .env("FENESTRA_TEST_LOG_SENTINEL", log)
        .env("HOSTNAME", host)
        .env("COMPUTERNAME", host)
        .env("USER", host)
        .env("USERNAME", host)
        .env("USERPROFILE", host)
        .env("FENESTRA_TEST_HOME_SENTINEL", host)
        .env("RUST_LOG", log)
        .output()
        .expect("the probe binary should launch")
}
