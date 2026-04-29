//! Integration tests for gow-seq (U-01).

use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn seq_basic_single_arg() {
    Command::cargo_bin("seq")
        .unwrap()
        .arg("3")
        .assert()
        .success()
        .stdout("1\n2\n3\n");
}

#[test]
fn seq_ten() {
    Command::cargo_bin("seq")
        .unwrap()
        .arg("10")
        .assert()
        .success()
        .stdout("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n");
}

#[test]
fn seq_step_two() {
    Command::cargo_bin("seq")
        .unwrap()
        .args(["1", "2", "10"])
        .assert()
        .success()
        .stdout("1\n3\n5\n7\n9\n");
}

#[test]
fn seq_decimal_precision() {
    Command::cargo_bin("seq")
        .unwrap()
        .args(["1.5", "0.5", "3"])
        .assert()
        .success()
        .stdout("1.5\n2.0\n2.5\n3.0\n");
}

#[test]
fn seq_no_f64_accumulation_error() {
    let out = Command::cargo_bin("seq")
        .unwrap()
        .args(["0.1", "0.1", "1.0"])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 10, "expected 10 lines, got {}", lines.len());
    assert_eq!(lines.last().unwrap(), &"1.0");
}

#[test]
fn seq_negative_increment() {
    Command::cargo_bin("seq")
        .unwrap()
        .args(["5", "-1", "1"])
        .assert()
        .success()
        .stdout("5\n4\n3\n2\n1\n");
}

#[test]
fn seq_custom_separator() {
    Command::cargo_bin("seq")
        .unwrap()
        .args(["-s", ",", "1", "5"])
        .assert()
        .success()
        .stdout("1,2,3,4,5\n");
}

#[test]
fn seq_no_args_exits_two() {
    Command::cargo_bin("seq")
        .unwrap()
        .assert()
        .failure()
        .code(2);
}

#[test]
fn seq_invalid_arg_exits_nonzero() {
    Command::cargo_bin("seq")
        .unwrap()
        .arg("abc")
        .assert()
        .failure()
        .stderr(contains("seq:"));
}
