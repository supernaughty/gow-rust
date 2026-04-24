use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_uniq_basic() {
    let mut cmd = Command::cargo_bin("uniq").unwrap();
    cmd.write_stdin("a\na\nb\nc\nc\n")
        .assert()
        .success()
        .stdout("a\nb\nc\n");
}

#[test]
fn test_uniq_count() {
    let mut cmd = Command::cargo_bin("uniq").unwrap();
    cmd.arg("-c")
        .write_stdin("a\na\nb\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("      2 a\n      1 b\n"));
}

#[test]
fn test_uniq_repeated() {
    let mut cmd = Command::cargo_bin("uniq").unwrap();
    cmd.arg("-d")
        .write_stdin("a\na\nb\n")
        .assert()
        .success()
        .stdout("a\n");
}
