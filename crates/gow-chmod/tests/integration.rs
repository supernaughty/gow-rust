//! Integration tests for `chmod` (FILE-10). Covers octal + symbolic + recursion
//! and verifies actual readonly-bit changes via `std::fs::metadata`.
//!
//! Per D-32 the only observable effect on Windows is the RO attribute, so every
//! test checks `metadata.permissions().readonly()` after the binary runs.

use assert_cmd::Command;
use predicates::prelude::*;

fn chmod() -> Command {
    Command::cargo_bin("chmod")
        .expect("chmod binary not found — run `cargo build -p gow-chmod` first")
}

fn write_fixture(dir: &std::path::Path, name: &str, ro: bool) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, b"content").unwrap();
    #[allow(clippy::permissions_set_readonly_false)]
    {
        let mut p = std::fs::metadata(&path).unwrap().permissions();
        p.set_readonly(ro);
        std::fs::set_permissions(&path, p).unwrap();
    }
    path
}

fn is_readonly(path: &std::path::Path) -> bool {
    std::fs::metadata(path).unwrap().permissions().readonly()
}

#[test]
fn test_chmod_644_writable() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", true);
    chmod().arg("644").arg(&f).assert().success();
    assert!(!is_readonly(&f));
}

#[test]
fn test_chmod_444_readonly() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", false);
    chmod().arg("444").arg(&f).assert().success();
    assert!(is_readonly(&f));
}

#[test]
fn test_chmod_0644_writable() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", true);
    chmod().arg("0644").arg(&f).assert().success();
    assert!(!is_readonly(&f));
}

#[test]
fn test_chmod_plus_w_clears() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", true);
    chmod().arg("+w").arg(&f).assert().success();
    assert!(!is_readonly(&f));
}

#[test]
fn test_chmod_minus_w_sets() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", false);
    chmod().arg("-w").arg(&f).assert().success();
    assert!(is_readonly(&f));
}

#[test]
fn test_chmod_u_equals_r_sets_ro() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", false);
    chmod().arg("u=r").arg(&f).assert().success();
    assert!(is_readonly(&f));
}

#[test]
fn test_chmod_u_equals_rw_clears() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", true);
    chmod().arg("u=rw").arg(&f).assert().success();
    assert!(!is_readonly(&f));
}

#[test]
fn test_chmod_g_only_is_noop() {
    let tmp = tempfile::tempdir().unwrap();
    // Start writable; g+w should leave RO bit unchanged → still writable.
    let f = write_fixture(tmp.path(), "f.txt", false);
    chmod().arg("g+w").arg(&f).assert().success();
    assert!(!is_readonly(&f));
}

#[test]
fn test_chmod_x_bit_noop() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", false);
    chmod().arg("+x").arg(&f).assert().success();
    assert!(!is_readonly(&f));
}

#[test]
fn test_chmod_recursive() {
    let tmp = tempfile::tempdir().unwrap();
    let subdir = tmp.path().join("sub");
    std::fs::create_dir(&subdir).unwrap();
    let f1 = subdir.join("a.txt");
    let f2 = subdir.join("b.txt");
    std::fs::write(&f1, b"x").unwrap();
    std::fs::write(&f2, b"x").unwrap();
    // Make both RO initially.
    {
        let mut p = std::fs::metadata(&f1).unwrap().permissions();
        p.set_readonly(true);
        std::fs::set_permissions(&f1, p).unwrap();
    }
    {
        let mut p = std::fs::metadata(&f2).unwrap().permissions();
        p.set_readonly(true);
        std::fs::set_permissions(&f2, p).unwrap();
    }

    chmod().arg("-R").arg("644").arg(&subdir).assert().success();
    assert!(!is_readonly(&f1));
    assert!(!is_readonly(&f2));
}

#[test]
fn test_chmod_nonexistent_file() {
    chmod()
        .arg("644")
        .arg("no-such-file.txt")
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("chmod:").and(predicate::str::contains("no-such-file")),
        );
}

#[test]
fn test_chmod_invalid_mode_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", false);
    chmod()
        .arg("xyz")
        .arg(&f)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("chmod:").and(predicate::str::contains("invalid")));
}

#[test]
fn test_chmod_missing_operand() {
    chmod()
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("missing operand"));
}

#[test]
fn test_chmod_verbose_prints() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", true);
    chmod()
        .arg("-v")
        .arg("644")
        .arg(&f)
        .assert()
        .success()
        .stdout(predicate::str::contains("mode of").and(predicate::str::contains("writable")));
}

#[test]
fn test_chmod_bad_flag_exits_1() {
    chmod()
        .arg("--invalid-flag-xyz")
        .arg("644")
        .arg("file")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_chmod_partial_failure_continues() {
    let tmp = tempfile::tempdir().unwrap();
    let ok = write_fixture(tmp.path(), "ok.txt", true);
    chmod()
        .arg("644")
        .arg("no-such")
        .arg(&ok)
        .assert()
        .failure()
        .code(1);
    // ok.txt should still have been processed to writable.
    assert!(!is_readonly(&ok));
}
