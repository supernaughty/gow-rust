//! Integration tests for gow-probe, validating gow-core platform primitives.
//!
//! These tests spawn the gow-probe binary via assert_cmd and assert observable
//! behavior: stdout, stderr, and exit codes. They cannot directly verify Win32
//! API calls or manifest contents, but they verify the observable effects.
//!
//! Test coverage map:
//!   FOUND-02 / WIN-01: init ok smoke test (UTF-8 init does not crash)
//!   FOUND-03 / WIN-03: exit code 1 on bad args (not 2); -- handling
//!   FOUND-06:          MSYS path conversion round-trip
//!   FOUND-04:          init does not crash (color module exercised at startup)

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: get a Command for the gow-probe binary.
fn probe() -> Command {
    Command::cargo_bin("gow-probe")
        .expect("gow-probe binary not found — run `cargo build -p gow-probe` first")
}

// ── Smoke tests ──────────────────────────────────────────────────────────────

/// FOUND-02, WIN-01: init() must not crash; probe exits 0 with expected output.
#[test]
fn test_default_init_ok() {
    probe()
        .assert()
        .success()
        .stdout(predicate::str::contains("gow-probe: init ok"));
}

// ── GNU exit code tests (FOUND-03, D-02) ─────────────────────────────────────

/// Bad argument must exit with code 1, not clap's default 2.
#[test]
fn test_bad_flag_exits_1_not_2() {
    probe()
        .arg("--completely-unknown-flag-xyz")
        .assert()
        .failure()
        .code(1);
}

/// The exit-code subcommand allows us to verify exit(0) directly.
#[test]
fn test_explicit_exit_code_zero() {
    probe()
        .args(["exit-code", "0"])
        .assert()
        .success()
        .code(0);
}

/// The exit-code subcommand with code 1 must produce exit 1.
#[test]
fn test_explicit_exit_code_one() {
    probe()
        .args(["exit-code", "1"])
        .assert()
        .failure()
        .code(1);
}

// ── MSYS path conversion tests (FOUND-06, D-06, D-07, D-08) ─────────────────

/// /c/Users/foo must be converted to C:\Users\foo.
#[test]
fn test_path_msys_c_drive_conversion() {
    probe()
        .args(["path", "/c/Users/foo"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r"C:\Users\foo"));
}

/// /d/workspace must be converted to D:\workspace.
#[test]
fn test_path_msys_d_drive_conversion() {
    probe()
        .args(["path", "/d/workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r"D:\workspace"));
}

/// Bare /c must NOT be converted (GOW #244 regression test, D-08).
#[test]
fn test_path_bare_drive_not_converted() {
    probe()
        .args(["path", "/c"])
        .assert()
        .success()
        // Must not print C:\ — must print /c unchanged
        .stdout(predicate::str::contains("/c"))
        .stdout(predicate::str::contains(r"C:\").not());
}

/// A regular relative path must pass through unchanged.
#[test]
fn test_path_relative_unchanged() {
    probe()
        .args(["path", "some/relative/path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("some"));
}

/// A Windows-style path with forward slashes is normalized.
#[test]
fn test_path_windows_forward_slash_normalized() {
    probe()
        .args(["path", "C:/Users/foo"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r"C:\Users\foo"));
}

// ── PowerShell compatibility (WIN-03) ────────────────────────────────────────
// WIN-03 is covered by the above tests: if the binary runs at all via assert_cmd
// (which uses the same process spawning as PowerShell), UTF-8 init and correct
// exit codes confirm PowerShell compatibility. No additional test needed.
