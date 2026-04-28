#![allow(non_snake_case)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::{Command as StdCommand, Stdio};
use tempfile::tempdir;

// On Windows, the canonical "echo" we can rely on is `cmd /C echo`.
// Helper: returns (program, leading_args) for a portable echo invocation.
fn echo_invocation() -> (&'static str, Vec<&'static str>) {
    if cfg!(windows) {
        ("cmd", vec!["/C", "echo"])
    } else {
        ("echo", vec![])
    }
}

#[test]
fn test_xargs_default_newline_mode_appends_args() {
    let (echo_bin, prefix) = echo_invocation();
    let mut c = Command::cargo_bin("xargs").unwrap();
    c.arg(echo_bin);
    for a in &prefix {
        c.arg(a);
    }
    c.write_stdin("foo\nbar\nbaz\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("foo"))
        .stdout(predicate::str::contains("bar"))
        .stdout(predicate::str::contains("baz"));
}

#[test]
fn test_xargs_null_mode_reads_nul_separated() {
    let (echo_bin, prefix) = echo_invocation();
    let mut c = Command::cargo_bin("xargs").unwrap();
    c.arg("-0").arg(echo_bin);
    for a in &prefix {
        c.arg(a);
    }
    // Embed NUL bytes between tokens; on Windows the binary will _setmode(0, _O_BINARY)
    // before reading, so 0x00 and 0x1A pass through unchanged.
    let stdin: Vec<u8> = b"alpha\0beta\0gamma\0".to_vec();
    c.write_stdin(stdin)
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"))
        .stdout(predicate::str::contains("gamma"));
}

#[test]
fn test_xargs_n_batches_args() {
    let (echo_bin, prefix) = echo_invocation();
    let mut c = Command::cargo_bin("xargs").unwrap();
    c.arg("-n").arg("2").arg(echo_bin);
    for a in &prefix {
        c.arg(a);
    }
    let output = c.write_stdin("a\nb\nc\nd\n").output().unwrap();
    assert!(output.status.success(), "xargs -n should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // -n 2 means each `echo` invocation receives at most 2 args, producing
    // 2 separate output lines: one for "a b" and one for "c d".
    let line_count = stdout.lines().count();
    assert_eq!(
        line_count, 2,
        "expected 2 echo invocations, got {} lines: {:?}",
        line_count, stdout
    );
}

#[test]
fn test_xargs_L_batches_lines() {
    let (echo_bin, prefix) = echo_invocation();
    let mut c = Command::cargo_bin("xargs").unwrap();
    c.arg("-L").arg("1").arg(echo_bin);
    for a in &prefix {
        c.arg(a);
    }
    let output = c.write_stdin("first\nsecond\nthird\n").output().unwrap();
    assert!(output.status.success(), "xargs -L should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().count(),
        3,
        "expected 3 invocations (one per input line), got: {:?}",
        stdout
    );
}

#[test]
fn test_xargs_replace_braces_substring() {
    let (echo_bin, prefix) = echo_invocation();
    let mut c = Command::cargo_bin("xargs").unwrap();
    c.arg("-I").arg(echo_bin);
    for a in &prefix {
        c.arg(a);
    }
    c.arg("prefix-{}-suffix");
    let output = c.write_stdin("alpha\nbeta\n").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("prefix-alpha-suffix"),
        "expected substring substitution; got: {:?}",
        stdout
    );
    assert!(
        stdout.contains("prefix-beta-suffix"),
        "expected substring substitution; got: {:?}",
        stdout
    );
    // Two invocations → two output lines
    assert_eq!(stdout.lines().count(), 2);
}

#[test]
fn test_xargs_empty_input_does_not_run_command() {
    // GNU 4.4+ default: empty input → don't invoke command, exit 0.
    let (echo_bin, prefix) = echo_invocation();
    let mut c = Command::cargo_bin("xargs").unwrap();
    c.arg(echo_bin);
    for a in &prefix {
        c.arg(a);
    }
    c.arg("WOULD_PRINT")
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::contains("WOULD_PRINT").not());
}

#[test]
fn test_xargs_I_rejects_combined_n() {
    // Mutual-exclusion enforcement: -I + -n must error with exit 125
    let (echo_bin, prefix) = echo_invocation();
    let mut c = Command::cargo_bin("xargs").unwrap();
    c.arg("-I").arg("-n").arg("2").arg(echo_bin);
    for a in &prefix {
        c.arg(a);
    }
    c.arg("{}");
    let output = c.write_stdin("a\nb\n").output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(125),
        "expected exit 125 (xargs internal error), got {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("-I"),
        "expected stderr to mention the conflict; got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_pipeline_find_print0_into_xargs_0() {
    // R016 + R015 cross-binary integration: find -print0 | xargs -0 echo
    // This is the GOW pipeline test — no shell, native pipe.
    // Proves that NUL bytes survive the Windows pipe without CRT text-mode corruption.
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("alpha.txt"), "").unwrap();
    fs::write(tmp.path().join("beta.txt"), "").unwrap();

    // Locate the freshly-built debug binaries from cargo target dir.
    let find_bin = assert_cmd::cargo::cargo_bin("find");
    let xargs_bin = assert_cmd::cargo::cargo_bin("xargs");

    // Spawn `find` with stdout piped.
    let mut find_proc = StdCommand::new(&find_bin)
        .arg(tmp.path())
        .arg("-name")
        .arg("*.txt")
        .arg("-type")
        .arg("f")
        .arg("-print0")
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn find");
    let find_stdout = find_proc.stdout.take().expect("find stdout");

    // Pipe into xargs -0 echo (or cmd /C echo on Windows).
    let (echo_bin, prefix) = if cfg!(windows) {
        ("cmd", vec!["/C", "echo"])
    } else {
        ("echo", vec![])
    };
    let mut xargs_cmd = StdCommand::new(&xargs_bin);
    xargs_cmd.arg("-0").arg(echo_bin);
    for a in &prefix {
        xargs_cmd.arg(a);
    }
    let xargs_out = xargs_cmd
        .stdin(Stdio::from(find_stdout))
        .output()
        .expect("run xargs");

    let _ = find_proc.wait();
    assert!(
        xargs_out.status.success(),
        "xargs failed: status={:?} stderr={}",
        xargs_out.status,
        String::from_utf8_lossy(&xargs_out.stderr)
    );
    let stdout = String::from_utf8_lossy(&xargs_out.stdout);
    assert!(
        stdout.contains("alpha.txt"),
        "missing alpha.txt in: {:?}",
        stdout
    );
    assert!(
        stdout.contains("beta.txt"),
        "missing beta.txt in: {:?}",
        stdout
    );
}
