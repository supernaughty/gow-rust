//! Integration tests for gow-fold (U-06).

use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn fold_wraps_at_width() {
    Command::cargo_bin("fold")
        .unwrap()
        .arg("-w")
        .arg("3")
        .write_stdin("abcdefghij\n")
        .assert()
        .success()
        .stdout("abc\ndef\nghi\nj\n");
}

#[test]
fn fold_default_width_80() {
    let input = "a".repeat(40);
    Command::cargo_bin("fold")
        .unwrap()
        .write_stdin(format!("{input}\n"))
        .assert()
        .success()
        .stdout(format!("{input}\n"));
}

#[test]
fn fold_no_wrap_needed() {
    Command::cargo_bin("fold")
        .unwrap()
        .write_stdin("a\n")
        .assert()
        .success()
        .stdout("a\n");
}

#[test]
fn fold_word_boundary_with_s() {
    Command::cargo_bin("fold")
        .unwrap()
        .args(["-w", "11", "-s"])
        .write_stdin("hello world goodbye\n")
        .assert()
        .success()
        .stdout("hello world\n goodbye\n");
}

#[test]
fn fold_zero_width_errors() {
    Command::cargo_bin("fold")
        .unwrap()
        .args(["-w", "0"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn fold_reads_stdin_no_file() {
    Command::cargo_bin("fold")
        .unwrap()
        .args(["-w", "5"])
        .write_stdin("abcde\n")
        .assert()
        .success()
        .stdout("abcde\n");
}

#[test]
fn fold_reads_file() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "abcdef").unwrap();

    Command::cargo_bin("fold")
        .unwrap()
        .args(["-w", "3"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout("abc\ndef\n");
}

#[test]
fn fold_wraps_exactly_at_width() {
    // Line exactly equal to width should not be wrapped
    Command::cargo_bin("fold")
        .unwrap()
        .args(["-w", "4"])
        .write_stdin("abcd\n")
        .assert()
        .success()
        .stdout("abcd\n");
}
