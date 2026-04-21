//! Integration tests for `false` — must always exit 1 (UTIL-09, D-22).
//!
//! Covers Dimension 1 (GNU exit codes), 2 (UTF-8 argv), 4 (no stdout/stderr noise).

use assert_cmd::Command;

fn false_cmd() -> Command {
    Command::cargo_bin("false")
        .expect("false binary not found — run `cargo build -p gow-false` first")
}

#[test]
fn test_exit_one_no_args() {
    false_cmd().assert().failure().code(1);
}

#[test]
fn test_exit_one_with_junk_args() {
    // GNU false ignores all args and still returns 1 (D-22).
    false_cmd()
        .args(["--bogus", "ignore", "me"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_no_stdout_no_stderr() {
    false_cmd().assert().failure().stdout("").stderr("");
}

#[test]
fn test_exit_one_with_double_dash() {
    false_cmd()
        .args(["--", "whatever"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_exit_one_with_utf8_args() {
    // Dimension 2 coverage — passing non-ASCII argv must not crash.
    false_cmd().arg("안녕").assert().failure().code(1);
}
