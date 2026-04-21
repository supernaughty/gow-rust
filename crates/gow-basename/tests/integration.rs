//! Integration tests for `basename` (UTIL-05).

use assert_cmd::Command;
use predicates::prelude::*;

fn basename() -> Command {
    Command::cargo_bin("basename")
        .expect("basename binary not found — run `cargo build -p gow-basename` first")
}

#[test]
fn test_basic_filename() {
    basename()
        .arg("foo/bar.txt")
        .assert()
        .success()
        .stdout("bar.txt\n");
}

#[test]
fn test_suffix_strip_positional() {
    basename()
        .args(["foo/bar.txt", ".txt"])
        .assert()
        .success()
        .stdout("bar\n");
}

#[test]
fn test_msys_path_preconverted() {
    basename()
        .arg("/c/Users/foo/doc.md")
        .assert()
        .success()
        .stdout("doc.md\n");
}

#[test]
fn test_multi_with_a_flag() {
    basename()
        .args(["-a", "foo/bar", "baz/qux"])
        .assert()
        .success()
        .stdout("bar\nqux\n");
}

#[test]
fn test_suffix_with_s_flag_implicit_multi() {
    basename()
        .args(["-s", ".txt", "foo/a.txt", "bar/b.txt"])
        .assert()
        .success()
        .stdout("a\nb\n");
}

#[test]
fn test_trailing_slash_stripped() {
    basename()
        .arg("foo/bar/")
        .assert()
        .success()
        .stdout("bar\n");
}

#[test]
fn test_no_args_error() {
    basename()
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("missing operand"));
}

#[test]
fn test_utf8_arg_roundtrip() {
    basename()
        .arg("foo/안녕.txt")
        .assert()
        .success()
        .stdout("안녕.txt\n");
}

#[test]
fn test_bad_flag_exits_1() {
    basename()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(1);
}
