use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_mv_file_to_new_path() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "hello").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&src).arg(&dest)
        .assert()
        .success();

    assert!(!src.exists());
    assert!(dest.exists());
    assert_eq!(fs::read_to_string(dest).unwrap(), "hello");
}

#[test]
fn test_mv_file_to_directory() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest_dir = dir.path().join("dest_dir");
    fs::create_dir(&dest_dir).unwrap();
    fs::write(&src, "hello").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&src).arg(&dest_dir)
        .assert()
        .success();

    assert!(!src.exists());
    assert!(dest_dir.join("src.txt").exists());
}

#[test]
fn test_mv_multiple_files_to_directory() {
    let dir = tempdir().unwrap();
    let src1 = dir.path().join("src1.txt");
    let src2 = dir.path().join("src2.txt");
    let dest_dir = dir.path().join("dest_dir");
    fs::create_dir(&dest_dir).unwrap();
    fs::write(&src1, "one").unwrap();
    fs::write(&src2, "two").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&src1).arg(&src2).arg(&dest_dir)
        .assert()
        .success();

    assert!(!src1.exists());
    assert!(!src2.exists());
    assert!(dest_dir.join("src1.txt").exists());
    assert!(dest_dir.join("src2.txt").exists());
}

#[test]
fn test_mv_directory_to_new_path() {
    let dir = tempdir().unwrap();
    let src_dir = dir.path().join("src_dir");
    let dest_dir = dir.path().join("dest_dir");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("file.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&src_dir).arg(&dest_dir)
        .assert()
        .success();

    assert!(!src_dir.exists());
    assert!(dest_dir.exists());
    assert!(dest_dir.join("file.txt").exists());
}

#[test]
fn test_mv_overwrite_file() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "new content").unwrap();
    fs::write(&dest, "old content").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&src).arg(&dest)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dest).unwrap(), "new content");
}

#[test]
fn test_mv_no_clobber() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "new content").unwrap();
    fs::write(&dest, "old content").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg("-n").arg(&src).arg(&dest)
        .assert()
        .success();

    assert!(src.exists()); // src should still exist
    assert_eq!(fs::read_to_string(dest).unwrap(), "old content");
}

#[test]
fn test_mv_same_file_error() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    fs::write(&src, "content").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&src).arg(&src)
        .assert()
        .failure()
        .stderr(predicate::str::contains("are the same file"));
}

#[test]
fn test_mv_nonexistent_source() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("nonexistent");
    let dest = dir.path().join("dest");

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&src).arg(&dest)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));
}

#[test]
fn test_mv_verbose() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "hello").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg("-v").arg(&src).arg(&dest)
        .assert()
        .success()
        .stdout(predicate::str::contains("renamed"));
}

#[test]
fn test_mv_multiple_sources_to_non_directory_error() {
    let dir = tempdir().unwrap();
    let src1 = dir.path().join("src1.txt");
    let src2 = dir.path().join("src2.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src1, "one").unwrap();
    fs::write(&src2, "two").unwrap();
    fs::write(&dest, "three").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&src1).arg(&src2).arg(&dest)
        .assert()
        .failure()
        .stderr(predicate::str::contains("is not a directory"));
}

#[test]
fn test_mv_directory_onto_file_error() {
    let dir = tempdir().unwrap();
    let src_dir = dir.path().join("src_dir");
    let dest_file = dir.path().join("dest.txt");
    fs::create_dir(&src_dir).unwrap();
    fs::write(&dest_file, "content").unwrap();

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&src_dir).arg(&dest_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot overwrite non-directory"));
}

#[test]
fn test_mv_symlink() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("target.txt");
    let link = dir.path().join("link.txt");
    let dest = dir.path().join("dest_link.txt");
    fs::write(&target, "target content").unwrap();
    
    #[cfg(target_os = "windows")]
    {
        if std::os::windows::fs::symlink_file(&target, &link).is_err() {
            // Skip if no privilege
            return;
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(&target, &link).unwrap();
    }

    let mut cmd = Command::cargo_bin("mv").unwrap();
    cmd.arg(&link).arg(&dest)
        .assert()
        .success();

    assert!(!link.exists());
    assert!(dest.is_symlink());
}
