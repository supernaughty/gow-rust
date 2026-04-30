use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn unlink_removes_file() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "hello").unwrap();
    let path = f.path().to_str().unwrap().to_string();
    // Prevent NamedTempFile from auto-deleting; we'll unlink manually
    let path_clone = path.clone();
    let (_, _tmp) = f.keep().unwrap(); // keep = don't auto-delete
    Command::cargo_bin("unlink")
        .unwrap()
        .arg(&path_clone)
        .assert()
        .success()
        .code(0);
    assert!(
        !std::path::Path::new(&path_clone).exists(),
        "file should be gone"
    );
}

#[test]
fn unlink_missing_file_exits_1() {
    Command::cargo_bin("unlink")
        .unwrap()
        .arg("nonexistent_xyz_unlink_98765.txt")
        .assert()
        .failure()
        .code(1)
        .stderr(contains("unlink:"));
}

#[test]
fn unlink_no_args_exits_2() {
    Command::cargo_bin("unlink")
        .unwrap()
        .assert()
        .failure()
        .code(2)
        .stderr(contains("unlink:"));
}

#[test]
fn unlink_two_args_exits_2() {
    Command::cargo_bin("unlink")
        .unwrap()
        .arg("a")
        .arg("b")
        .assert()
        .failure()
        .code(2)
        .stderr(contains("unlink:"));
}
