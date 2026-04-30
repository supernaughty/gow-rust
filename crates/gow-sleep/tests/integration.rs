//! Integration tests for gow-sleep (U-02).

use assert_cmd::Command;
use std::time::Instant;

#[test]
fn sleep_zero_returns_immediately() {
    Command::cargo_bin("sleep")
        .unwrap()
        .arg("0")
        .assert()
        .success();
}

#[test]
fn sleep_fractional() {
    let start = Instant::now();
    Command::cargo_bin("sleep")
        .unwrap()
        .arg("0.1")
        .assert()
        .success();
    assert!(
        start.elapsed().as_millis() >= 80,
        "sleep 0.1 elapsed too short: {}ms",
        start.elapsed().as_millis()
    );
}

#[test]
fn sleep_no_arg_errors() {
    Command::cargo_bin("sleep")
        .unwrap()
        .assert()
        .failure()
        .code(1);
}

#[test]
fn sleep_bad_arg_errors() {
    Command::cargo_bin("sleep")
        .unwrap()
        .arg("abc")
        .assert()
        .failure()
        .code(1);
}
