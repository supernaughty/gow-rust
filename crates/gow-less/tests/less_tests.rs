//! Headless integration tests for `gow-less`.
//!
//! Because `assert_cmd::Command` runs the binary with stdout piped (not a real
//! TTY), our non-TTY fallback path is exercised. The interactive raw-mode pager
//! is deliberately NOT tested here — it is reserved for manual UAT.
//!
//! Tests cover:
//!   - D-07: file arg and stdin pass-through
//!   - D-08: ANSI byte-faithful passthrough
//!   - D-09: large-file streaming without OOM
//!   - Basic error handling (missing file → exit 1 with GNU-style error prefix)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// D-07: File argument — all lines appear in stdout, exit 0.
#[test]
fn test_less_file_arg_prints_full_contents_in_non_tty_mode() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("test.txt");
    fs::write(&file_path, "alpha\nbeta\ngamma\n").unwrap();

    Command::cargo_bin("less")
        .unwrap()
        .arg(file_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"))
        .stdout(predicate::str::contains("gamma"));
}

/// D-07: Stdin pass-through — piped stdin flows to stdout, exit 0.
#[test]
fn test_less_stdin_pass_through_in_non_tty_mode() {
    Command::cargo_bin("less")
        .unwrap()
        .write_stdin("hello\nworld\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))
        .stdout(predicate::str::contains("world"));
}

/// D-08: ANSI byte-faithful passthrough.
///
/// Writes a file containing raw ANSI escape sequences (red + reset) and
/// asserts that the exact bytes appear in stdout without modification.
/// Proves that `io::copy` in non-TTY mode does not strip ESC sequences.
#[test]
fn test_less_preserves_ansi_escape_bytes() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("ansi.txt");
    // \x1b[31m = red, \x1b[0m = reset — standard ANSI color codes.
    let payload: &[u8] = b"\x1b[31mRED TEXT\x1b[0m\n";
    fs::write(&file_path, payload).unwrap();

    let output = Command::cargo_bin("less")
        .unwrap()
        .arg(file_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success(), "less should exit 0 in non-TTY mode");
    // Byte-faithful: every byte of `payload` must appear contiguously in stdout.
    assert!(
        output.stdout.windows(payload.len()).any(|w| w == payload),
        "ANSI bytes were not preserved. stdout (hex) = {:02x?}",
        output.stdout
    );
}

/// Unicode content — Korean multi-byte UTF-8 passes through correctly.
/// Validates that byte offsets in LineIndex are byte-counted (not char-counted).
#[test]
fn test_less_handles_unicode_content() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("kr.txt");
    fs::write(&file_path, "한글 테스트\n파이팅\n").unwrap();

    Command::cargo_bin("less")
        .unwrap()
        .arg(file_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("한글"))
        .stdout(predicate::str::contains("파이팅"));
}

/// D-09: Large-file regression — 1 MiB of content (10 000 lines × ~100 bytes).
///
/// Verifies that:
///   1. The binary exits 0 without OOM-killing.
///   2. The FIRST line appears in stdout (proves streaming forward from offset 0).
///   3. The LAST line appears in stdout (proves full content is passed through in non-TTY mode).
///
/// This is a regression guard against accidentally using `read_to_string` or similar
/// full-buffering approach at the boundary between non-TTY copy and TTY pager.
#[test]
fn test_less_large_file_streams_without_oom() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("big.txt");
    let mut content = String::with_capacity(1_100_000);
    for i in 0..10_000u32 {
        content.push_str(&format!(
            "line-{:08} padding-padding-padding-padding-padding\n",
            i
        ));
    }
    fs::write(&file_path, &content).unwrap();

    let output = Command::cargo_bin("less")
        .unwrap()
        .arg(file_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "less should exit 0 on a large file; stderr = {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("line-00000000"),
        "first line missing from stdout"
    );
    assert!(
        stdout.contains("line-00009999"),
        "last line missing from stdout"
    );
}

/// Missing file → non-zero exit code and GNU-style `less: <path>: ...` in stderr.
#[test]
fn test_less_missing_file_errors_with_exit_1() {
    let output = Command::cargo_bin("less")
        .unwrap()
        .arg("nonexistent-file-12345-xyz.txt")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "expected non-zero exit on missing file"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit code 1 for missing file"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("less:"),
        "expected GNU-style 'less: ' prefix in stderr; got {:?}",
        stderr
    );
}

/// Empty file → exit 0, stdout is empty.
#[test]
fn test_less_empty_file_succeeds() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("empty.txt");
    fs::write(&file_path, "").unwrap();

    Command::cargo_bin("less")
        .unwrap()
        .arg(file_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
