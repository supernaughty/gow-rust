//! Integration tests for gow-tac (U-03).

use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn tac_reverses() {
    Command::cargo_bin("tac")
        .unwrap()
        .write_stdin("a\nb\nc\n")
        .assert()
        .success()
        .stdout("c\nb\na\n");
}

#[test]
fn tac_handles_no_trailing_newline() {
    Command::cargo_bin("tac")
        .unwrap()
        .write_stdin("a\nb\nc")
        .assert()
        .success()
        .stdout("c\nb\na\n");
}

#[test]
fn tac_empty_input() {
    Command::cargo_bin("tac")
        .unwrap()
        .write_stdin("")
        .assert()
        .success()
        .stdout("");
}

#[test]
fn tac_reads_file() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "line1").unwrap();
    writeln!(tmp, "line2").unwrap();
    writeln!(tmp, "line3").unwrap();

    Command::cargo_bin("tac")
        .unwrap()
        .arg(tmp.path())
        .assert()
        .success()
        .stdout("line3\nline2\nline1\n");
}

#[test]
fn tac_missing_file_errors() {
    Command::cargo_bin("tac")
        .unwrap()
        .arg("nonexistent_xyz_98765_tac.txt")
        .assert()
        .failure()
        .code(1)
        .stderr(contains("tac:"));
}
