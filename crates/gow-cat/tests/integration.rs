//! Integration tests for `cat` (FILE-01). Covers ROADMAP success criterion 4
//! (`cat -n` on UTF-8/Korean content without mojibake) plus all 7 flags + error
//! paths + multi-operand + stdin dash + CP949 byte-level passthrough.

use assert_cmd::Command;
use predicates::prelude::*;

fn cat() -> Command {
    Command::cargo_bin("cat")
        .expect("cat binary not found — run `cargo build -p gow-cat` first")
}

fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn test_cat_passthrough() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"hello\n");
    cat().arg(&f).assert().success().stdout("hello\n");
}

#[test]
fn test_cat_n_line_numbers() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"a\nb\nc\n");
    cat().arg("-n")
        .arg(&f)
        .assert()
        .success()
        .stdout("     1\ta\n     2\tb\n     3\tc\n");
}

#[test]
fn test_cat_n_utf8_korean() {
    // ROADMAP criterion 4: cat -n on Korean UTF-8 preserves bytes without mojibake.
    let tmp = tempfile::tempdir().unwrap();
    let utf8 = "안녕\n세계\n".as_bytes();
    let f = write_fixture(tmp.path(), "korean.txt", utf8);
    let expected = "     1\t안녕\n     2\t세계\n".as_bytes().to_vec();
    cat().arg("-n")
        .arg(&f)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
}

#[test]
fn test_cat_b_nonblank_only() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"a\n\nb\n");
    cat().arg("-b")
        .arg(&f)
        .assert()
        .success()
        .stdout("     1\ta\n\n     2\tb\n");
}

#[test]
fn test_cat_s_squeeze_blanks() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"a\n\n\n\nb\n");
    cat().arg("-s")
        .arg(&f)
        .assert()
        .success()
        .stdout("a\n\nb\n");
}

#[test]
fn test_cat_v_control_chars() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"a\r\x01\n");
    cat().arg("-v")
        .arg(&f)
        .assert()
        .success()
        .stdout("a^M^A\n");
}

#[test]
fn test_cat_e_dollar() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"a\nb\n");
    cat().arg("-E")
        .arg(&f)
        .assert()
        .success()
        .stdout("a$\nb$\n");
}

#[test]
fn test_cat_t_tabs() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"a\tb\n");
    cat().arg("-T")
        .arg(&f)
        .assert()
        .success()
        .stdout("a^Ib\n");
}

#[test]
fn test_cat_a_equiv_vet() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"a\tb\n");
    cat().arg("-A")
        .arg(&f)
        .assert()
        .success()
        .stdout("a^Ib$\n");
}

#[test]
fn test_cat_dash_reads_stdin() {
    cat().arg("-")
        .write_stdin("hi")
        .assert()
        .success()
        .stdout("hi");
}

#[test]
fn test_cat_no_operand_reads_stdin() {
    cat().write_stdin("hi").assert().success().stdout("hi");
}

#[test]
fn test_cat_multi_file_concat() {
    let tmp = tempfile::tempdir().unwrap();
    let f1 = write_fixture(tmp.path(), "1.txt", b"a\n");
    let f2 = write_fixture(tmp.path(), "2.txt", b"b\n");
    cat().arg(&f1)
        .arg(&f2)
        .assert()
        .success()
        .stdout("a\nb\n");
}

#[test]
fn test_cat_multi_file_with_dash() {
    // cat f1 - f2 mixes file + stdin + file with no headers.
    let tmp = tempfile::tempdir().unwrap();
    let f1 = write_fixture(tmp.path(), "1.txt", b"a\n");
    let f2 = write_fixture(tmp.path(), "2.txt", b"b\n");
    cat().arg(&f1)
        .arg("-")
        .arg(&f2)
        .write_stdin("MID\n")
        .assert()
        .success()
        .stdout("a\nMID\nb\n");
}

#[test]
fn test_cat_nonexistent_file() {
    cat().arg("no-such-file.txt")
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("cat:").and(predicate::str::contains("no-such-file")),
        );
}

#[test]
fn test_cat_partial_failure_continues() {
    let tmp = tempfile::tempdir().unwrap();
    let ok = write_fixture(tmp.path(), "ok.txt", b"ok-content\n");
    cat().arg("no-such")
        .arg(&ok)
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("ok-content"))
        .stderr(predicate::str::contains("no-such"));
}

#[test]
fn test_cat_cp949_bytes_passthrough() {
    // CP949 bytes for "안녕" — NOT valid UTF-8. D-48: bytes must pass through
    // unchanged with no decode attempt.
    let tmp = tempfile::tempdir().unwrap();
    let cp949 = &[0xBE, 0xC8, 0xB3, 0xE7, b'\n'];
    let f = write_fixture(tmp.path(), "cp949.txt", cp949);
    cat().arg(&f)
        .assert()
        .success()
        .stdout(predicate::eq(cp949 as &[u8]));
}

#[test]
fn test_cat_n_accumulates_across_files() {
    // GNU: -n counter does NOT reset between files.
    let tmp = tempfile::tempdir().unwrap();
    let f1 = write_fixture(tmp.path(), "1.txt", b"a\n");
    let f2 = write_fixture(tmp.path(), "2.txt", b"b\n");
    cat().arg("-n")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .success()
        .stdout("     1\ta\n     2\tb\n");
}

#[test]
fn test_cat_help_does_not_panic() {
    cat().arg("--help").assert().success();
}

#[test]
fn test_cat_bad_flag_exits_1() {
    // Phase 1 D-02: bad args exit 1, not 2.
    cat().arg("--invalid-flag-xyz").assert().failure().code(1);
}
