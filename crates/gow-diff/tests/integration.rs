use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_diff_identical_files() {
    let tmp = tempdir().unwrap();
    let file1 = tmp.path().join("a.txt");
    let file2 = tmp.path().join("b.txt");
    fs::write(&file1, "hello\nworld\n").unwrap();
    fs::write(&file2, "hello\nworld\n").unwrap();

    let mut cmd = Command::cargo_bin("diff").unwrap();
    cmd.arg(file1.to_str().unwrap())
        .arg(file2.to_str().unwrap())
        .assert()
        .success()
        .code(0)
        .stdout("");
}

#[test]
fn test_diff_different_files() {
    let tmp = tempdir().unwrap();
    let file1 = tmp.path().join("a.txt");
    let file2 = tmp.path().join("b.txt");
    fs::write(&file1, "hello\nworld\n").unwrap();
    fs::write(&file2, "hello\nrust\n").unwrap();

    let mut cmd = Command::cargo_bin("diff").unwrap();
    cmd.arg(file1.to_str().unwrap())
        .arg(file2.to_str().unwrap())
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("+++"))
        .stdout(predicate::str::contains("@@"));
}

#[test]
fn test_diff_unified_context_zero() {
    let tmp = tempdir().unwrap();
    let file1 = tmp.path().join("a.txt");
    let file2 = tmp.path().join("b.txt");
    fs::write(&file1, "line1\nline2\nline3\nline4\nline5\n").unwrap();
    fs::write(&file2, "line1\nline2\nchanged\nline4\nline5\n").unwrap();

    // With -U 0, no context lines should appear around changed lines
    let mut cmd = Command::cargo_bin("diff").unwrap();
    cmd.arg("-U")
        .arg("0")
        .arg(file1.to_str().unwrap())
        .arg(file2.to_str().unwrap());
    let output = cmd.output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain unified diff headers
    assert!(stdout.contains("---"), "stdout should contain ---: {stdout}");
    assert!(stdout.contains("+++"), "stdout should contain +++: {stdout}");
    assert!(stdout.contains("@@"), "stdout should contain @@: {stdout}");
    // With 0 context, should NOT contain the unchanged surrounding lines
    assert!(!stdout.contains("line1"), "with 0 context should not contain line1: {stdout}");
    assert!(!stdout.contains("line5"), "with 0 context should not contain line5: {stdout}");
}

#[test]
fn test_diff_error_missing_file() {
    let tmp = tempdir().unwrap();
    let file1 = tmp.path().join("exists.txt");
    fs::write(&file1, "hello\n").unwrap();

    let mut cmd = Command::cargo_bin("diff").unwrap();
    cmd.arg(file1.to_str().unwrap())
        .arg("nonexistent_file_xyz.txt")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn test_diff_recursive() {
    let tmp = tempdir().unwrap();
    let dir1 = tmp.path().join("dir1");
    let dir2 = tmp.path().join("dir2");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();

    // Same file in both dirs
    fs::write(dir1.join("same.txt"), "same content\n").unwrap();
    fs::write(dir2.join("same.txt"), "same content\n").unwrap();

    // Different file
    fs::write(dir1.join("diff.txt"), "old content\n").unwrap();
    fs::write(dir2.join("diff.txt"), "new content\n").unwrap();

    let mut cmd = Command::cargo_bin("diff").unwrap();
    cmd.arg("-r")
        .arg(dir1.to_str().unwrap())
        .arg(dir2.to_str().unwrap());
    let output = cmd.output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show diff for diff.txt
    assert!(
        stdout.contains("@@") || stdout.contains("diff.txt"),
        "stdout should contain diff output: {stdout}"
    );
}

#[test]
fn test_diff_absent_as_empty() {
    let tmp = tempdir().unwrap();
    let file2 = tmp.path().join("new.txt");
    fs::write(&file2, "hello\nworld\n").unwrap();

    // -N treats absent file as empty
    let mut cmd = Command::cargo_bin("diff").unwrap();
    cmd.arg("-N")
        .arg(tmp.path().join("nonexistent.txt").to_str().unwrap())
        .arg(file2.to_str().unwrap());
    let output = cmd.output().unwrap();
    // Should exit 1 (differences found) not 2 (error)
    assert_eq!(output.status.code(), Some(1), "should exit 1 with -N for absent file");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("@@"), "should show diff output: {stdout}");
}
