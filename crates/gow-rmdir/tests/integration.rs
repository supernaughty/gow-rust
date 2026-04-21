//! Integration tests for `rmdir` (FILE-07).
//! Covers VALIDATION.md Dimensions 1 (GNU compat), 2 (UTF-8), 4 (error path).

use assert_cmd::Command;
use predicates::prelude::*;

fn rmdir() -> Command {
    Command::cargo_bin("rmdir")
        .expect("rmdir binary not found — run `cargo build -p gow-rmdir` first")
}

#[test]
fn test_remove_empty_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("empty");
    std::fs::create_dir(&target).unwrap();
    rmdir().arg(&target).assert().success();
    assert!(!target.exists());
}

#[test]
fn test_remove_nonempty_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("nonempty");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("file.txt"), "content").unwrap();
    rmdir()
        .arg(&target)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::starts_with("rmdir:"));
    assert!(
        target.is_dir(),
        "nonempty dir should still exist after failed rmdir"
    );
}

#[test]
fn test_p_removes_parent_chain() {
    // Empty a/b/c chain → rmdir -p should remove all three.
    let tmp = tempfile::tempdir().unwrap();
    let leaf = tmp.path().join("a").join("b").join("c");
    std::fs::create_dir_all(&leaf).unwrap();

    rmdir().args(["-p"]).arg(&leaf).assert().success();

    assert!(!leaf.exists());
    assert!(!tmp.path().join("a").join("b").exists());
    assert!(!tmp.path().join("a").exists());
}

#[test]
fn test_p_stops_at_nonempty_parent() {
    // Critical D-28 regression guard: parent with a sibling file must NOT be removed.
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a");
    let leaf = a.join("b").join("c");
    std::fs::create_dir_all(&leaf).unwrap();
    std::fs::write(a.join("sibling.txt"), "keep me").unwrap();

    rmdir().args(["-p"]).arg(&leaf).assert().success();

    assert!(!leaf.exists());
    assert!(!a.join("b").exists());
    assert!(
        a.is_dir(),
        "parent with sibling file must NOT be removed (D-28 stop condition)"
    );
    assert!(a.join("sibling.txt").is_file(), "sibling must be preserved");
}

#[test]
fn test_verbose_prints_removed_line() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("vtest");
    std::fs::create_dir(&target).unwrap();
    rmdir()
        .args(["-v"])
        .arg(&target)
        .assert()
        .success()
        .stdout(predicate::str::contains("removing directory"));
}

#[test]
fn test_no_args_error() {
    rmdir()
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("missing operand"));
}

#[test]
fn test_bad_flag_exits_1() {
    // Per Phase 1 D-02, gow_core::args::parse_gnu maps clap's exit 2 → exit 1.
    rmdir()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_utf8_directory_name() {
    let tmp = tempfile::tempdir().unwrap();
    let utf8_dir = tmp.path().join("안녕");
    std::fs::create_dir(&utf8_dir).unwrap();
    rmdir().arg(&utf8_dir).assert().success();
    assert!(!utf8_dir.exists());
}
