//! Integration tests for `tee` (UTIL-04).
//!
//! Covers ROADMAP Phase 2 success criterion 5 (tee writes to both file and stdout;
//! -a appends). See .planning/phases/02-stateless/02-07-PLAN.md.

use assert_cmd::Command;
use predicates::prelude::*;

fn tee() -> Command {
    Command::cargo_bin("tee")
        .expect("tee binary not found — run `cargo build -p gow-tee` first")
}

#[test]
fn test_basic_write_to_file_and_stdout() {
    // ROADMAP criterion 5: `tee file.txt` writes stdin to both file AND stdout.
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("out.txt");

    tee()
        .arg(&target)
        .write_stdin("hello\n")
        .assert()
        .success()
        .stdout("hello\n");

    let contents = std::fs::read_to_string(&target).unwrap();
    assert_eq!(contents, "hello\n");
}

#[test]
fn test_append_flag_appends() {
    // ROADMAP criterion 5: `tee -a` appends rather than truncating.
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("append.txt");
    std::fs::write(&target, "existing\n").unwrap();

    tee()
        .args(["-a"])
        .arg(&target)
        .write_stdin("added\n")
        .assert()
        .success();

    let contents = std::fs::read_to_string(&target).unwrap();
    assert_eq!(contents, "existing\nadded\n");
}

#[test]
fn test_truncates_without_append() {
    // Default (no -a): existing content is discarded.
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("trunc.txt");
    std::fs::write(&target, "old content that should disappear\n").unwrap();

    tee()
        .arg(&target)
        .write_stdin("new\n")
        .assert()
        .success();

    let contents = std::fs::read_to_string(&target).unwrap();
    assert_eq!(contents, "new\n");
}

#[test]
fn test_multiple_files_all_receive_input() {
    // Fan-out: multiple file operands + stdout each get a full copy.
    let tmp = tempfile::tempdir().unwrap();
    let f1 = tmp.path().join("out1.txt");
    let f2 = tmp.path().join("out2.txt");

    tee()
        .arg(&f1)
        .arg(&f2)
        .write_stdin("fanout\n")
        .assert()
        .success()
        .stdout("fanout\n");

    assert_eq!(std::fs::read_to_string(&f1).unwrap(), "fanout\n");
    assert_eq!(std::fs::read_to_string(&f2).unwrap(), "fanout\n");
}

#[test]
fn test_i_flag_smoke() {
    // -i flag must not crash. The console handler either succeeds (silently) or
    // is silently ignored (detached context); either way output is unchanged.
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("i.txt");
    tee()
        .args(["-i"])
        .arg(&target)
        .write_stdin("ignored-interrupts\n")
        .assert()
        .success()
        .stdout("ignored-interrupts\n");

    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "ignored-interrupts\n"
    );
}

#[test]
fn test_no_files_just_echoes_stdin() {
    // `tee` with no file operands writes stdin straight to stdout.
    tee()
        .write_stdin("naked\n")
        .assert()
        .success()
        .stdout("naked\n");
}

#[test]
fn test_bad_file_exits_1_but_continues() {
    // Opening a directory as a file should fail. tee must still write stdin to
    // stdout and report the error to stderr. Final exit code = 1. Other sinks
    // (the good file and stdout) still receive the data.
    let tmp = tempfile::tempdir().unwrap();
    let good_file = tmp.path().join("good.txt");
    let dir_path = tmp.path().to_path_buf(); // a directory, not a regular file

    tee()
        .arg(&dir_path)
        .arg(&good_file)
        .write_stdin("partial\n")
        .assert()
        .failure()
        .code(1)
        .stdout("partial\n")
        .stderr(predicate::str::starts_with("tee:"));

    // good_file still received the data despite the bad_path error.
    assert_eq!(std::fs::read_to_string(&good_file).unwrap(), "partial\n");
}

#[test]
fn test_bad_flag_exits_1() {
    // GNU convention: unknown flag -> exit 1 (D-02 via gow_core::args::parse_gnu).
    tee()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_utf8_content_roundtrip() {
    // UTF-8 bytes round-trip cleanly through both stdout and the file sink.
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("utf8.txt");
    tee()
        .arg(&target)
        .write_stdin("안녕 세상\n")
        .assert()
        .success()
        .stdout("안녕 세상\n");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "안녕 세상\n");
}
