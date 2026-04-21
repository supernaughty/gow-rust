//! Integration tests for `yes` (UTIL-07, D-23).
//!
//! Covers:
//!   Dimension 1 (GNU compat): default `y\n` output, multi-arg space-join
//!   Dimension 2 (UTF-8 argv round-trip)
//!   Dimension 4 (error path): `BrokenPipe` → exit 0
//!
//! Because `yes` loops forever, we use `std::process::Command` directly with
//! `Stdio::piped()` so we can read a bounded prefix and then `child.kill()`
//! (or let the reader drop, which closes the pipe and triggers BrokenPipe
//! in the child — the BrokenPipe test relies on that behavior).

use std::io::Read;
use std::process::{Command, Stdio};

/// Path to the `yes` binary built by cargo (`target/<triple>/debug/yes.exe`).
fn yes_bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("yes")
}

/// Spawn `yes [args...]`, read up to `n_bytes` from stdout, then kill the child.
fn capture_prefix(args: &[&str], n_bytes: usize) -> Vec<u8> {
    let mut child = Command::new(yes_bin())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn yes");

    let mut stdout = child.stdout.take().expect("stdout handle");
    let mut buf = vec![0u8; n_bytes];
    stdout
        .read_exact(&mut buf)
        .expect("read_exact from yes stdout");
    // Kill the child; drop the stdout reader first so OS flushes cleanly.
    drop(stdout);
    let _ = child.kill();
    let _ = child.wait();
    buf
}

// ── Dimension 1: default `y\n` output ───────────────────────────────────────

#[test]
fn test_default_outputs_y_newline() {
    // Read first 32 bytes; they should all be `y\n` repeated 16 times.
    let out = capture_prefix(&[], 32);
    let expected: Vec<u8> = std::iter::repeat_n(*b"y\n", 16).flatten().collect();
    assert_eq!(
        out,
        expected,
        "expected 16 'y\\n' pairs, got {}",
        String::from_utf8_lossy(&out)
    );
}

// ── Dimension 1: multi-arg space-join (D-23) ────────────────────────────────

#[test]
fn test_multi_arg_space_joined() {
    // `yes hello world` should output `hello world\n` repeatedly.
    let out = capture_prefix(&["hello", "world"], 24);
    let text = String::from_utf8_lossy(&out).into_owned();
    assert!(
        text.contains("hello world\n"),
        "expected 'hello world\\n' in output, got: {text:?}"
    );
}

#[test]
fn test_three_arg_space_joined() {
    // `yes a b c` should output `a b c\n` repeatedly.
    let out = capture_prefix(&["a", "b", "c"], 12);
    let text = String::from_utf8_lossy(&out).into_owned();
    assert!(
        text.contains("a b c\n"),
        "expected 'a b c\\n' in output, got: {text:?}"
    );
}

#[test]
fn test_single_arg_with_trailing_newline() {
    // `yes foo` → `foo\n` repeated at least twice in a 16-byte sample.
    let out = capture_prefix(&["foo"], 16);
    let text = String::from_utf8_lossy(&out).into_owned();
    let count = text.matches("foo\n").count();
    assert!(
        count >= 2,
        "expected at least 2 'foo\\n' occurrences, got {count} in {text:?}"
    );
}

// ── Dimension 2: UTF-8 argv round-trip ──────────────────────────────────────

#[test]
fn test_utf8_arg_roundtrip() {
    // Korean input via argv must survive to stdout as UTF-8.
    let out = capture_prefix(&["안녕"], 32);
    let text = String::from_utf8_lossy(&out).into_owned();
    assert!(
        text.contains("안녕\n"),
        "expected '안녕\\n' in output, got: {text:?}"
    );
}

// ── Dimension 4: BrokenPipe → exit 0 (D-23 / RESEARCH Q4) ───────────────────

#[test]
fn test_broken_pipe_exits_zero() {
    // Spawn yes, read 64 bytes, drop the pipe reader → yes gets BrokenPipe.
    // GNU yes behavior: exit 0 (not 1, not a panic). RESEARCH.md Q4.
    let mut child = Command::new(yes_bin())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn yes");

    let mut stdout = child.stdout.take().expect("stdout handle");
    let mut buf = [0u8; 64];
    stdout.read_exact(&mut buf).expect("read 64 bytes");
    drop(stdout); // Close the read end → write end sees BrokenPipe.

    let status = child.wait().expect("wait for yes");
    assert!(
        status.success(),
        "yes should exit 0 on BrokenPipe, got {status:?}"
    );
    assert_eq!(status.code(), Some(0), "exit code should be 0");
}
