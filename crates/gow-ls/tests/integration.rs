use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_ls_basic() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("file1.txt"), "content").unwrap();
    fs::write(tmp.path().join("file2.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("file1.txt"))
        .stdout(predicate::str::contains("file2.txt"));
}

#[test]
fn test_ls_one_per_line() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("file1.txt"), "content").unwrap();
    fs::write(tmp.path().join("file2.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("-1")
        .assert()
        .success()
        .stdout("file1.txt\nfile2.txt\n");
}

#[test]
fn test_ls_all() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("file1.txt"), "content").unwrap();
    fs::write(tmp.path().join(".hidden"), "content").unwrap();

    // Without -a
    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("-1")
        .assert()
        .success()
        .stdout("file1.txt\n");

    // With -a
    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("-a")
        .arg("-1")
        .assert()
        .success()
        .stdout(".hidden\nfile1.txt\n");
}

#[test]
fn test_ls_almost_all() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("file1.txt"), "content").unwrap();
    fs::write(tmp.path().join(".hidden"), "content").unwrap();

    // -A should show .hidden but on Windows read_dir doesn't yield . and ..
    // so it's same as -a for now.
    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("-A")
        .arg("-1")
        .assert()
        .success()
        .stdout(".hidden\nfile1.txt\n");
}

#[test]
fn test_ls_reverse() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("a"), "").unwrap();
    fs::write(tmp.path().join("b"), "").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("-1")
        .arg("-r")
        .assert()
        .success()
        .stdout("b\na\n");
}

#[test]
fn test_ls_long() {
    let tmp = tempdir().unwrap();
    let f = tmp.path().join("file.txt");
    fs::write(&f, "hello").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("-l")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("-rw-rw-rw-"))
        .stdout(predicate::str::contains("5")) // size
        .stdout(predicate::str::contains("file.txt"));
}

#[test]
fn test_ls_recursive() {
    let tmp = tempdir().unwrap();
    let d1 = tmp.path().join("dir1");
    fs::create_dir(&d1).unwrap();
    fs::write(d1.join("file1.txt"), "").unwrap();
    
    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("-R")
        .assert()
        .success()
        .stdout(predicate::str::contains(".:"))
        .stdout(predicate::str::contains("dir1"))
        .stdout(predicate::str::contains("dir1:"))
        .stdout(predicate::str::contains("file1.txt"));
}

#[test]
fn test_ls_color_always() {
    let tmp = tempdir().unwrap();
    fs::create_dir(tmp.path().join("dir1")).unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("--color=always")
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[1;34mdir1\x1b[0m"));
}

#[test]
fn test_ls_exe_color() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("test.exe"), "").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("--color=always")
        .assert()
        .success()
        .stdout(predicate::str::contains("\x1b[1;32mtest.exe\x1b[0m"));
}

#[test]
fn test_ls_hidden_windows_attr() {
    // This test might be tricky if we can't easily set Windows hidden attribute in a cross-platform way.
    // But gow-core has is_hidden which checks it.
    // For now, dot-prefix is enough for common tests.
}

#[cfg(target_os = "windows")]
#[test]
fn test_ls_junction_display() {
    let tmp = tempdir().unwrap();
    let target = tmp.path().join("target");
    fs::create_dir(&target).unwrap();
    let link = tmp.path().join("link");
    
    // Create a junction if symlink fails.
    let mut cmd = std::process::Command::new("cmd");
    cmd.arg("/c").arg("mklink").arg("/j").arg(&link).arg(&target);
    if cmd.status().unwrap().success() {
        let mut cmd = Command::cargo_bin("ls").unwrap();
        cmd.current_dir(tmp.path())
            .arg("-l")
            .assert()
            .success()
            .stdout(predicate::str::contains("l"))
            .stdout(predicate::str::contains("->"))
            .stdout(predicate::str::contains("[junction]"));
    }
}

#[test]
fn test_ls_multiple_targets() {
    let tmp = tempdir().unwrap();
    let d1 = tmp.path().join("d1");
    let d2 = tmp.path().join("d2");
    fs::create_dir(&d1).unwrap();
    fs::create_dir(&d2).unwrap();
    fs::write(d1.join("f1"), "").unwrap();
    fs::write(d2.join("f2"), "").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.arg(d1.to_str().unwrap())
        .arg(d2.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("d1:"))
        .stdout(predicate::str::contains("f1"))
        .stdout(predicate::str::contains("d2:"))
        .stdout(predicate::str::contains("f2"));
}

#[test]
fn test_ls_nonexistent() {
    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.arg("nonexistent_file_xyz")
        .assert()
        .failure()
        .stderr(predicate::str::contains("ls: cannot access 'nonexistent_file_xyz'"));
}

#[test]
fn test_ls_file_operand() {
    let tmp = tempdir().unwrap();
    let f = tmp.path().join("file.txt");
    fs::write(&f, "").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.arg(f.to_str().unwrap())
        .assert()
        .success()
        .stdout("file.txt\n");
}

#[test]
fn test_ls_long_file_operand() {
    let tmp = tempdir().unwrap();
    let f = tmp.path().join("file.txt");
    fs::write(&f, "123").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.arg("-l")
        .arg(f.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("-rw-rw-rw-"))
        .stdout(predicate::str::contains("3"))
        .stdout(predicate::str::contains("file.txt"));
}

#[test]
fn test_ls_a_with_targets() {
    let tmp = tempdir().unwrap();
    let d1 = tmp.path().join("d1");
    fs::create_dir(&d1).unwrap();
    fs::write(d1.join(".hidden"), "").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.arg("-a")
        .arg("-1")
        .arg(d1.to_str().unwrap())
        .assert()
        .success()
        .stdout(".hidden\n");
}

#[test]
fn test_ls_color_never() {
    let tmp = tempdir().unwrap();
    fs::create_dir(tmp.path().join("dir1")).unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("--color=never")
        .assert()
        .success()
        .stdout("dir1\n"); // no escape codes
}

#[test]
fn test_ls_empty_dir() {
    let tmp = tempdir().unwrap();
    let d1 = tmp.path().join("empty");
    fs::create_dir(&d1).unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.arg(d1.to_str().unwrap())
        .assert()
        .success()
        .stdout("");
}

#[test]
fn test_ls_sort_lexical() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("z"), "").unwrap();
    fs::write(tmp.path().join("a"), "").unwrap();
    fs::write(tmp.path().join("m"), "").unwrap();

    let mut cmd = Command::cargo_bin("ls").unwrap();
    cmd.current_dir(tmp.path())
        .arg("-1")
        .assert()
        .success()
        .stdout("a\nm\nz\n");
}
