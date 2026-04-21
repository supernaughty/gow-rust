//! Integration tests for `dirname` (UTIL-06).

use assert_cmd::Command;
use predicates::prelude::*;

fn dirname() -> Command {
    Command::cargo_bin("dirname")
        .expect("dirname binary not found — run `cargo build -p gow-dirname` first")
}

#[test]
fn test_basic_parent() {
    dirname()
        .arg("foo/bar.txt")
        .assert()
        .success()
        .stdout("foo\n");
}

#[test]
fn test_bare_file_is_dot() {
    dirname()
        .arg("foo")
        .assert()
        .success()
        .stdout(".\n");
}

#[test]
fn test_msys_path_converted_then_dirname() {
    dirname()
        .arg("/c/Users/foo/doc.md")
        .assert()
        .success()
        .stdout("C:\\Users\\foo\n");
}

#[test]
fn test_multi_arg_each_on_own_line() {
    dirname()
        .args(["foo/bar", "baz/qux"])
        .assert()
        .success()
        .stdout("foo\nbaz\n");
}

#[test]
fn test_trailing_slash_handled() {
    dirname()
        .arg("foo/bar/")
        .assert()
        .success()
        .stdout("foo\n");
}

#[test]
fn test_no_args_error() {
    dirname()
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("missing operand"));
}

#[test]
fn test_utf8_arg_roundtrip() {
    // On Windows, Path::parent of `foo/안녕/file.txt` preserves the forward slash
    // since Path doesn't normalize separators on display. Use a loose assertion
    // that tolerates either `foo/안녕` or `foo\안녕` serialization.
    dirname()
        .arg("foo/안녕/file.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("안녕"))
        .stdout(predicate::str::contains("foo"));
}

#[test]
fn test_bad_flag_exits_1() {
    dirname()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(1);
}
