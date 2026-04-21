//! Integration tests for `head` (TEXT-01). Exercises -n, -c, numeric shorthand
//! (`-5`), multi-file headers (`==> file <==`), -q/-v toggles, stdin (`-` and
//! no operand), UTF-8 preservation, empty file, bad flag, and partial-failure
//! continuation (Pattern E).

use assert_cmd::Command;
use predicates::prelude::*;

fn head() -> Command {
    Command::cargo_bin("head")
        .expect("head binary not found — run `cargo build -p gow-head` first")
}

fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn test_head_default_10() {
    let tmp = tempfile::tempdir().unwrap();
    let mut content = String::new();
    for i in 1..=15 {
        content.push_str(&format!("{i}\n"));
    }
    let f = write_fixture(tmp.path(), "f.txt", content.as_bytes());

    let mut expected = String::new();
    for i in 1..=10 {
        expected.push_str(&format!("{i}\n"));
    }

    head().arg(&f).assert().success().stdout(expected);
}

#[test]
fn test_head_n_3() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"a\nb\nc\nd\ne\n");
    head()
        .arg("-n")
        .arg("3")
        .arg(&f)
        .assert()
        .success()
        .stdout("a\nb\nc\n");
}

#[test]
fn test_head_shorthand_5() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"1\n2\n3\n4\n5\n6\n7\n");
    head()
        .arg("-5")
        .arg(&f)
        .assert()
        .success()
        .stdout("1\n2\n3\n4\n5\n");
}

#[test]
fn test_head_c_10() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"hello world\n");
    head()
        .arg("-c")
        .arg("10")
        .arg(&f)
        .assert()
        .success()
        .stdout("hello worl");
}

#[test]
fn test_head_c_over_size() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"hi");
    head()
        .arg("-c")
        .arg("100")
        .arg(&f)
        .assert()
        .success()
        .stdout("hi");
}

#[test]
fn test_head_n_0_empty_output() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"a\nb\n");
    head()
        .arg("-n")
        .arg("0")
        .arg(&f)
        .assert()
        .success()
        .stdout("");
}

#[test]
fn test_head_multi_file_headers() {
    let tmp = tempfile::tempdir().unwrap();
    let f1 = write_fixture(tmp.path(), "1.txt", b"alpha\n");
    let f2 = write_fixture(tmp.path(), "2.txt", b"beta\n");
    head()
        .arg("-n")
        .arg("1")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("==>").and(
                predicate::str::contains("alpha").and(predicate::str::contains("beta")),
            ),
        );
}

#[test]
fn test_head_q_suppresses_headers() {
    let tmp = tempfile::tempdir().unwrap();
    let f1 = write_fixture(tmp.path(), "1.txt", b"alpha\n");
    let f2 = write_fixture(tmp.path(), "2.txt", b"beta\n");
    head()
        .arg("-q")
        .arg("-n")
        .arg("1")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .success()
        .stdout("alpha\nbeta\n");
}

#[test]
fn test_head_v_forces_header_single_file() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"x\n");
    head()
        .arg("-v")
        .arg("-n")
        .arg("1")
        .arg(&f)
        .assert()
        .success()
        .stdout(predicate::str::contains("==>").and(predicate::str::contains("x\n")));
}

#[test]
fn test_head_stdin_no_operand() {
    head()
        .arg("-n")
        .arg("2")
        .write_stdin("one\ntwo\nthree\n")
        .assert()
        .success()
        .stdout("one\ntwo\n");
}

#[test]
fn test_head_dash_operand() {
    head()
        .arg("-n")
        .arg("1")
        .arg("-")
        .write_stdin("first\nsecond\n")
        .assert()
        .success()
        .stdout("first\n");
}

#[test]
fn test_head_nonexistent_file() {
    head()
        .arg("no-such-file.txt")
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("head:").and(predicate::str::contains("no-such-file")),
        );
}

#[test]
fn test_head_utf8_preserved() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "k.txt", "안녕\n세계\n".as_bytes());
    head()
        .arg("-n")
        .arg("1")
        .arg(&f)
        .assert()
        .success()
        .stdout("안녕\n");
}

#[test]
fn test_head_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "empty.txt", b"");
    head().arg(&f).assert().success().stdout("");
}

#[test]
fn test_head_bad_flag_exits_1() {
    head().arg("--invalid-flag-xyz").assert().failure().code(1);
}

#[test]
fn test_head_partial_failure_continues() {
    let tmp = tempfile::tempdir().unwrap();
    let ok = write_fixture(tmp.path(), "ok.txt", b"ok-content\n");
    head()
        .arg("-n")
        .arg("1")
        .arg("no-such")
        .arg(&ok)
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("ok-content"))
        .stderr(predicate::str::contains("no-such"));
}

#[test]
fn test_head_n_attached_value() {
    // `-n5` (value attached) is clap's standard short-with-value form.
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "f.txt", b"1\n2\n3\n4\n5\n6\n");
    head()
        .arg("-n5")
        .arg(&f)
        .assert()
        .success()
        .stdout("1\n2\n3\n4\n5\n");
}

#[test]
fn test_head_c_mid_multibyte_splits_bytes() {
    // "안" is 3 UTF-8 bytes (0xEC 0x95 0x88). `-c 2` must emit the first 2 raw
    // bytes — byte-exact behavior (D-48). We compare on raw bytes to avoid
    // UTF-8 validity requirements in the assertion.
    let tmp = tempfile::tempdir().unwrap();
    let f = write_fixture(tmp.path(), "k.txt", "안".as_bytes());
    let out = head()
        .arg("-c")
        .arg("2")
        .arg(&f)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(out, vec![0xEC, 0x95]);
}
