//! Integration tests for `true` — must always exit 0 (UTIL-08, D-22).
//!
//! Covers Dimension 1 (GNU exit codes), 2 (UTF-8 argv), 4 (no stdout/stderr noise).

use assert_cmd::Command;

fn true_cmd() -> Command {
    Command::cargo_bin("true")
        .expect("true binary not found — run `cargo build -p gow-true` first")
}

#[test]
fn test_exit_zero_no_args() {
    true_cmd().assert().success().code(0);
}

#[test]
fn test_exit_zero_with_junk_args() {
    // GNU true ignores all args and still returns 0 (D-22).
    true_cmd()
        .args(["--bogus", "ignore", "me"])
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_no_stdout_no_stderr() {
    true_cmd().assert().success().stdout("").stderr("");
}

#[test]
fn test_exit_zero_with_double_dash() {
    true_cmd()
        .args(["--", "whatever"])
        .assert()
        .success()
        .code(0);
}

#[test]
fn test_exit_zero_with_utf8_args() {
    // Dimension 2 coverage — passing non-ASCII argv must not crash.
    true_cmd().arg("안녕").assert().success().code(0);
}
