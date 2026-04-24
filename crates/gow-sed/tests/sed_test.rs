use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sed_basic_substitution() {
    let mut cmd = Command::cargo_bin("sed").unwrap();
    cmd.arg("s/hello/world/").write_stdin("hello universe\nhello again");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("world universe"))
        .stdout(predicate::str::contains("world again"));
}

#[test]
fn test_sed_global_substitution() {
    let mut cmd = Command::cargo_bin("sed").unwrap();
    cmd.arg("s/a/b/g").write_stdin("aaa");
    cmd.assert()
        .success()
        .stdout(predicate::eq("bbb\n"));
}

#[test]
fn test_sed_case_insensitive() {
    let mut cmd = Command::cargo_bin("sed").unwrap();
    cmd.arg("s/hello/world/I").write_stdin("HELLO universe");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("world universe"));
}

#[test]
fn test_sed_quiet_mode_and_print() {
    let mut cmd = Command::cargo_bin("sed").unwrap();
    cmd.args(&["-n", "s/match/REPLACED/p"]).write_stdin("match this\nignore this");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("REPLACED this"))
        .stdout(predicate::str::contains("ignore this").not());
}

#[test]
fn test_sed_in_place() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "original content").unwrap();

    let mut cmd = Command::cargo_bin("sed").unwrap();
    // Use -e to prevent -i from consuming the script as a suffix
    cmd.args(&["-i", "-e", "s/original/new/", file_path.to_str().unwrap()]);
    cmd.assert().success();

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content.trim(), "new content");
}

#[test]
fn test_sed_in_place_with_backup() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "original content").unwrap();

    let mut cmd = Command::cargo_bin("sed").unwrap();
    // For suffix, we can use attached short option or equals with long option
    cmd.args(&["--in-place=.bak", "s/original/new/", file_path.to_str().unwrap()]);
    cmd.assert().success();

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content.trim(), "new content");

    let backup_path = temp.path().join("test.txt.bak");
    assert!(backup_path.exists());
    let backup_content = fs::read_to_string(&backup_path).unwrap();
    assert_eq!(backup_content.trim(), "original content");
}

#[test]
fn test_sed_multiple_expressions() {
    let mut cmd = Command::cargo_bin("sed").unwrap();
    cmd.args(&["-e", "s/a/b/", "-e", "s/b/c/"]).write_stdin("a");
    cmd.assert()
        .success()
        .stdout(predicate::eq("c\n"));
}

#[test]
fn test_sed_semicolon_separator() {
    let mut cmd = Command::cargo_bin("sed").unwrap();
    cmd.arg("s/a/b/; s/b/c/").write_stdin("a");
    cmd.assert()
        .success()
        .stdout(predicate::eq("c\n"));
}

#[test]
fn test_sed_escaped_delimiter() {
    let mut cmd = Command::cargo_bin("sed").unwrap();
    cmd.arg("s/a\\/b/c/").write_stdin("a/b");
    cmd.assert()
        .success()
        .stdout(predicate::eq("c\n"));
}

#[test]
fn test_sed_capture_groups() {
    let mut cmd = Command::cargo_bin("sed").unwrap();
    cmd.arg("s/\\(hello\\) \\(world\\)/\\2, \\1/").write_stdin("hello world");
    cmd.assert()
        .success()
        .stdout(predicate::eq("world, hello\n"));
}

#[test]
fn test_sed_ampersand() {
    let mut cmd = Command::cargo_bin("sed").unwrap();
    cmd.arg("s/hello/[&]/").write_stdin("hello");
    cmd.assert()
        .success()
        .stdout(predicate::eq("[hello]\n"));
}
