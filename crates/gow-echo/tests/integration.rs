//! Integration tests for `echo` (UTIL-01).
//! Covers Dimensions 1 (GNU compat), 2 (UTF-8), 4 (error path).

use assert_cmd::Command;

fn echo() -> Command {
    Command::cargo_bin("echo")
        .expect("echo binary not found — run `cargo build -p gow-echo` first")
}

#[test]
fn test_default_appends_newline() {
    echo().arg("hello").assert().success().stdout("hello\n");
}

#[test]
fn test_multi_arg_space_joined() {
    echo()
        .args(["hello", "world"])
        .assert()
        .success()
        .stdout("hello world\n");
}

#[test]
fn test_n_flag_suppresses_newline() {
    echo()
        .args(["-n", "hi"])
        .assert()
        .success()
        .stdout("hi"); // no trailing \n
}

#[test]
fn test_e_flag_tab_escape() {
    echo()
        .args(["-e", "a\\tb"])
        .assert()
        .success()
        .stdout("a\tb\n");
}

#[test]
fn test_e_flag_backslash_c_early_break() {
    // \c should suppress trailing newline AND everything after \c
    echo()
        .args(["-e", "a\\cb"])
        .assert()
        .success()
        .stdout("a"); // no 'b', no newline
}

#[test]
fn test_e_flag_disables_escapes() {
    echo()
        .args(["-E", "\\t"])
        .assert()
        .success()
        .stdout("\\t\n"); // literal backslash-t plus newline
}

#[test]
fn test_default_no_e_treats_escapes_literal() {
    // Without -e, \t should NOT be interpreted (D-21 default is -E behavior).
    echo().arg("\\t").assert().success().stdout("\\t\n");
}

#[test]
fn test_e_hex_escape() {
    echo()
        .args(["-e", "\\x41"])
        .assert()
        .success()
        .stdout("A\n");
}

#[test]
fn test_e_octal_escape() {
    echo()
        .args(["-e", "\\0101"]) // octal 101 = 0x41 = 'A'
        .assert()
        .success()
        .stdout("A\n");
}

#[test]
fn test_e_esc_escape() {
    // \033 = octal 33 = 0x1B (ESC); \e is the GNU-extension alias.
    echo()
        .args(["-e", "\\033"])
        .assert()
        .success()
        .stdout("\x1B\n");
}

#[test]
fn test_utf8_arg_roundtrip() {
    echo().arg("안녕").assert().success().stdout("안녕\n");
}

#[test]
fn test_no_args_prints_just_newline() {
    echo().assert().success().stdout("\n");
}

#[test]
fn test_bad_flag_exits_1_not_2() {
    // Per Phase 1 D-02, gow_core::args::parse_gnu maps clap's exit 2 → exit 1.
    echo()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(1);
}
