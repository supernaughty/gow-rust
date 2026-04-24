use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_grep_stdin() {
    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never")
        .arg("world")
        .write_stdin("hello\nworld\nrust\n")
        .assert()
        .success()
        .stdout("world\n");
}

#[test]
fn test_grep_file() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("test.txt");
    fs::write(&file_path, "hello\nworld\nrust\n").unwrap();

    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never").arg("world").arg(file_path.to_str().unwrap());
    cmd.assert()
        .success()
        .stdout("world\n");
}

#[test]
fn test_grep_recursive() {
    let tmp = tempdir().unwrap();
    let dir1 = tmp.path().join("dir1");
    fs::create_dir(&dir1).unwrap();
    fs::write(dir1.join("file1.txt"), "hello world\n").unwrap();
    
    let dir2 = tmp.path().join("dir2");
    fs::create_dir(&dir2).unwrap();
    fs::write(dir2.join("file2.txt"), "goodbye world\n").unwrap();

    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never").arg("-r").arg("world").arg(tmp.path().to_str().unwrap());
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("file1.txt:hello world\n"))
        .stdout(predicate::str::contains("file2.txt:goodbye world\n"));
}

#[test]
fn test_grep_count() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("test.txt");
    fs::write(&file_path, "world\nhello\nworld\n").unwrap();

    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never").arg("-c").arg("world").arg(file_path.to_str().unwrap());
    cmd.assert()
        .success()
        .stdout("2\n");
}

#[test]
fn test_grep_ignore_case() {
    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never").arg("-i").arg("WORLD")
        .write_stdin("hello\nworld\nrust\n")
        .assert()
        .success()
        .stdout("world\n");
}

#[test]
fn test_grep_with_filename() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("test.txt");
    fs::write(&file_path, "world\n").unwrap();

    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never").arg("-H").arg("world").arg(file_path.to_str().unwrap());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.txt:world\n"));
}

#[test]
fn test_grep_no_filename() {
    let tmp = tempdir().unwrap();
    let f1 = tmp.path().join("f1.txt");
    let f2 = tmp.path().join("f2.txt");
    fs::write(&f1, "world1\n").unwrap();
    fs::write(&f2, "world2\n").unwrap();

    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never").arg("-h").arg("world").arg(f1.to_str().unwrap()).arg(f2.to_str().unwrap());
    cmd.assert()
        .success()
        .stdout("world1\nworld2\n");
}

#[test]
fn test_grep_invert_match() {
    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never").arg("-v").arg("world")
        .write_stdin("hello\nworld\nrust\n")
        .assert()
        .success()
        .stdout("hello\nrust\n");
}

#[test]
fn test_grep_line_number() {
    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never").arg("-n").arg("world")
        .write_stdin("hello\nworld\nrust\n")
        .assert()
        .success()
        .stdout("2:world\n");
}

#[test]
fn test_grep_no_match_exit_code() {
    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("nomatch")
        .write_stdin("hello\nworld\n")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_grep_fixed_strings() {
    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("--color=never").arg("-F").arg(".*")
        .write_stdin("hello\n.*\nrust\n")
        .assert()
        .success()
        .stdout(".*\n");
}

#[test]
fn test_grep_directory_error() {
    let tmp = tempdir().unwrap();
    let dir_path = tmp.path().join("mydir");
    fs::create_dir(&dir_path).unwrap();

    let mut cmd = Command::cargo_bin("grep").unwrap();
    cmd.arg("pattern").arg(dir_path.to_str().unwrap());
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Is a directory"));
}
