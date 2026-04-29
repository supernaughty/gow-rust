use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn printf_decimal() {
    Command::cargo_bin("printf").unwrap()
        .args(["%d\n", "42"])
        .assert().success()
        .stdout("42\n");
}

#[test]
fn printf_string() {
    Command::cargo_bin("printf").unwrap()
        .args(["%s %s\n", "hello", "world"])
        .assert().success()
        .stdout("hello world\n");
}

#[test]
fn printf_format_repeats() {
    // Format string repeats for each extra positional arg
    Command::cargo_bin("printf").unwrap()
        .args(["%d\n", "1", "2", "3"])
        .assert().success()
        .stdout("1\n2\n3\n");
}

#[test]
fn printf_width_precision() {
    Command::cargo_bin("printf").unwrap()
        .args(["%05.2f\n", "3.1"])
        .assert().success()
        .stdout("03.10\n");
}

#[test]
fn printf_octal() {
    Command::cargo_bin("printf").unwrap()
        .args(["%o\n", "8"])
        .assert().success()
        .stdout("10\n");
}

#[test]
fn printf_hex() {
    Command::cargo_bin("printf").unwrap()
        .args(["%x\n", "255"])
        .assert().success()
        .stdout("ff\n");
}

#[test]
fn printf_literal_percent() {
    Command::cargo_bin("printf").unwrap()
        .args(["%%\n"])
        .assert().success()
        .stdout("%\n");
}

#[test]
fn printf_tab_escape() {
    let out = Command::cargo_bin("printf").unwrap()
        .args(["\t"])
        .assert().success()
        .get_output()
        .stdout.clone();
    assert_eq!(out, b"\t");
}

#[test]
fn printf_no_args_exits_1() {
    Command::cargo_bin("printf").unwrap()
        .assert().failure().code(1)
        .stderr(contains("printf:"));
}
