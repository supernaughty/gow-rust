use assert_cmd::Command;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

fn make_tempfile() -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "content").unwrap();
    f
}

#[test]
fn test_file_exists() {
    let f = make_tempfile();
    Command::cargo_bin("test")
        .unwrap()
        .arg("-f")
        .arg(f.path())
        .assert()
        .code(0);
}

#[test]
fn test_file_not_exists() {
    Command::cargo_bin("test")
        .unwrap()
        .arg("-f")
        .arg("nonexistent_xyz_test_98765.txt")
        .assert()
        .code(1);
}

#[test]
fn test_directory_exists() {
    let d = TempDir::new().unwrap();
    Command::cargo_bin("test")
        .unwrap()
        .arg("-d")
        .arg(d.path())
        .assert()
        .code(0);
}

#[test]
fn test_exists_any_type() {
    let f = make_tempfile();
    Command::cargo_bin("test")
        .unwrap()
        .arg("-e")
        .arg(f.path())
        .assert()
        .code(0);
}

#[test]
fn test_zero_length_string() {
    Command::cargo_bin("test")
        .unwrap()
        .arg("-z")
        .arg("")
        .assert()
        .code(0);
}

#[test]
fn test_zero_length_nonempty_exits_1() {
    Command::cargo_bin("test")
        .unwrap()
        .arg("-z")
        .arg("hello")
        .assert()
        .code(1);
}

#[test]
fn test_nonzero_length_string() {
    Command::cargo_bin("test")
        .unwrap()
        .arg("-n")
        .arg("hello")
        .assert()
        .code(0);
}

#[test]
fn test_nonzero_length_empty_exits_1() {
    Command::cargo_bin("test")
        .unwrap()
        .arg("-n")
        .arg("")
        .assert()
        .code(1);
}

#[test]
fn test_integer_gt_true() {
    Command::cargo_bin("test")
        .unwrap()
        .args(["5", "-gt", "3"])
        .assert()
        .code(0);
}

#[test]
fn test_integer_gt_false() {
    Command::cargo_bin("test")
        .unwrap()
        .args(["3", "-gt", "5"])
        .assert()
        .code(1);
}

#[test]
fn test_integer_eq_true() {
    Command::cargo_bin("test")
        .unwrap()
        .args(["5", "-eq", "5"])
        .assert()
        .code(0);
}

#[test]
fn test_integer_ne_false() {
    Command::cargo_bin("test")
        .unwrap()
        .args(["5", "-ne", "5"])
        .assert()
        .code(1);
}

#[test]
fn test_string_eq_true() {
    Command::cargo_bin("test")
        .unwrap()
        .args(["hello", "=", "hello"])
        .assert()
        .code(0);
}

#[test]
fn test_string_eq_false() {
    Command::cargo_bin("test")
        .unwrap()
        .args(["hello", "=", "world"])
        .assert()
        .code(1);
}

#[test]
fn test_no_args_exits_1() {
    // GNU test: empty expression = false = exit 1 (NOT exit 2)
    Command::cargo_bin("test")
        .unwrap()
        .assert()
        .code(1);
}

#[test]
fn test_negation() {
    // !(-f nonexistent) = !(false) = true = exit 0
    Command::cargo_bin("test")
        .unwrap()
        .args(["!", "-f", "nonexistent_xyz_test_98765.txt"])
        .assert()
        .code(0);
}

#[test]
fn test_bracket_mode_with_sentinel() {
    // Simulate what [.bat does: pass --_bracket_ then args then ]
    let f = make_tempfile();
    Command::cargo_bin("test")
        .unwrap()
        .arg("--_bracket_")
        .arg("-f")
        .arg(f.path())
        .arg("]")
        .assert()
        .code(0);
}

#[test]
fn test_bracket_mode_missing_close_bracket() {
    // --_bracket_ without ] at end → exit 2
    let f = make_tempfile();
    Command::cargo_bin("test")
        .unwrap()
        .arg("--_bracket_")
        .arg("-f")
        .arg(f.path())
        // no ] here
        .assert()
        .code(2);
}
