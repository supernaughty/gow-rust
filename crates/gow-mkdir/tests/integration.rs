//! Integration tests for `mkdir` (FILE-06).
//! Covers VALIDATION.md Dimensions 1 (GNU compat), 3 (MSYS path), 4 (error path).

use assert_cmd::Command;
use predicates::prelude::*;

fn mkdir() -> Command {
    Command::cargo_bin("mkdir")
        .expect("mkdir binary not found — run `cargo build -p gow-mkdir` first")
}

#[test]
fn test_create_single_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("new_dir");
    mkdir().arg(&target).assert().success();
    assert!(target.is_dir(), "expected {target:?} to be a directory");
}

#[test]
fn test_create_existing_without_p_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("exists");
    std::fs::create_dir(&target).unwrap();
    mkdir()
        .arg(&target)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::starts_with("mkdir:"));
}

#[test]
fn test_p_creates_nested() {
    // FILE-06 / ROADMAP criterion 4: `mkdir -p a/b/c` creates the chain.
    let tmp = tempfile::tempdir().unwrap();
    let nested = tmp.path().join("a").join("b").join("c");
    mkdir().args(["-p"]).arg(&nested).assert().success();
    assert!(nested.is_dir());
    // Parents exist too.
    assert!(tmp.path().join("a").is_dir());
    assert!(tmp.path().join("a").join("b").is_dir());
}

#[test]
fn test_p_is_idempotent() {
    // ROADMAP criterion 4 / GOW #133: running `mkdir -p` twice on an existing
    // nested path still succeeds (the whole point of -p).
    let tmp = tempfile::tempdir().unwrap();
    let nested = tmp.path().join("a").join("b").join("c");
    mkdir().args(["-p"]).arg(&nested).assert().success();
    mkdir().args(["-p"]).arg(&nested).assert().success();
    assert!(nested.is_dir());
}

#[test]
fn test_verbose_prints_created_line() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("vtest");
    mkdir()
        .args(["-v"])
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("created directory"));
}

#[test]
fn test_multiple_operands() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("alpha");
    let b = tmp.path().join("beta");
    mkdir().args([&a, &b]).assert().success();
    assert!(a.is_dir());
    assert!(b.is_dir());
}

#[test]
fn test_no_args_error() {
    mkdir()
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("missing operand"));
}

#[test]
fn test_bad_flag_exits_1() {
    // Per Phase 1 D-02, gow_core::args::parse_gnu maps clap's exit 2 → exit 1.
    mkdir()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_utf8_directory_name() {
    // Dimension 2: UTF-8 filename round-trips correctly through the Windows FS.
    let tmp = tempfile::tempdir().unwrap();
    let utf8_dir = tmp.path().join("안녕_dir");
    mkdir().arg(&utf8_dir).assert().success();
    assert!(utf8_dir.is_dir());
}
