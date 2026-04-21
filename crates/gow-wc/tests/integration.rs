//! Integration tests for `wc` (TEXT-03). Covers ROADMAP success criterion 2
//! (`wc -m file_with_korean.txt` != `wc -c` on UTF-8 files) plus Dimensions 1/2/4
//! (GNU compat, UTF-8, error path) of 02-VALIDATION.md.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn wc() -> Command {
    Command::cargo_bin("wc").expect("wc binary not found — run `cargo build -p gow-wc` first")
}

fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(contents).unwrap();
    path
}

#[test]
fn test_default_prints_lines_words_bytes() {
    // "hello world\nfoo bar\n" = 2 lines, 4 words, 20 bytes
    let tmp = tempfile::tempdir().unwrap();
    let p = write_fixture(tmp.path(), "simple.txt", b"hello world\nfoo bar\n");
    wc().arg(&p)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\s*2\s+4\s+20\s+.*simple\.txt").unwrap());
}

#[test]
fn test_l_flag_only_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let p = write_fixture(tmp.path(), "l.txt", b"a\nb\nc\n");
    wc().args(["-l"])
        .arg(&p)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\s*3\s+.*l\.txt").unwrap());
}

#[test]
fn test_c_flag_only_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let p = write_fixture(tmp.path(), "c.txt", b"12345");
    wc().args(["-c"])
        .arg(&p)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\s*5\s+.*c\.txt").unwrap());
}

#[test]
fn test_w_flag_only_words() {
    let tmp = tempfile::tempdir().unwrap();
    let p = write_fixture(tmp.path(), "w.txt", b"one two three\n");
    wc().args(["-w"])
        .arg(&p)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\s*3\s+.*w\.txt").unwrap());
}

#[test]
fn test_m_flag_utf8_char_count() {
    // "안녕 세상\n" = 4 Korean + space + newline = 6 scalar values, 14 bytes.
    let tmp = tempfile::tempdir().unwrap();
    let p = write_fixture(tmp.path(), "m.txt", "안녕 세상\n".as_bytes());
    wc().args(["-m"])
        .arg(&p)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\s*6\s+.*m\.txt").unwrap());
}

#[test]
fn test_c_vs_m_differ_on_utf8() {
    // For "안녕\n" (7 bytes, 3 scalar values), -c=7 but -m=3. This is ROADMAP criterion 2.
    let tmp = tempfile::tempdir().unwrap();
    let p = write_fixture(tmp.path(), "diff.txt", "안녕\n".as_bytes());
    // -c prints byte count
    wc().args(["-c"])
        .arg(&p)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\s*7\s+").unwrap());
    // -m prints scalar-value count
    wc().args(["-m"])
        .arg(&p)
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\s*3\s+").unwrap());
}

#[test]
fn test_multi_file_prints_total_line() {
    let tmp = tempfile::tempdir().unwrap();
    let p1 = write_fixture(tmp.path(), "a.txt", b"a\n");
    let p2 = write_fixture(tmp.path(), "b.txt", b"b\nc\n");
    wc().arg(&p1)
        .arg(&p2)
        .assert()
        .success()
        .stdout(predicate::str::contains("total"));
}

#[test]
fn test_stdin_with_dash_operand() {
    // "x y z\n" = 1 line, 3 words, 6 bytes; operand "-" must appear in output
    wc().arg("-")
        .write_stdin("x y z\n")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\s*1\s+3\s+6\s+-").unwrap());
}

#[test]
fn test_stdin_no_operand() {
    // "alpha beta\n" = 1 line, 2 words, 11 bytes; no trailing filename
    wc().write_stdin("alpha beta\n")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"\s*1\s+2\s+11\s*$").unwrap());
}

#[test]
fn test_missing_file_exits_1() {
    wc().arg("/this/absolutely/does/not/exist/xyzzy.txt")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::starts_with("wc:"));
}

#[test]
fn test_bad_flag_exits_1() {
    wc().arg("--completely-unknown-xyz").assert().failure().code(1);
}

#[test]
fn test_invalid_utf8_no_panic_on_m() {
    // Two invalid UTF-8 bytes followed by a newline. `wc -m` must not crash.
    let tmp = tempfile::tempdir().unwrap();
    let p = write_fixture(tmp.path(), "bad_utf8.txt", &[0xFF, 0xFE, b'\n']);
    wc().args(["-m"]).arg(&p).assert().success(); // exit 0 per D-17b
}
