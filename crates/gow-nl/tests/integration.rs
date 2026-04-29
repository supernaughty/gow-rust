use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn nl_default_numbers_nonempty_lines() {
    // Default mode (-b t): only non-empty lines get numbers
    Command::cargo_bin("nl").unwrap()
        .write_stdin("a\nb\nc\n")
        .assert().success()
        .stdout("     1\ta\n     2\tb\n     3\tc\n");
}

#[test]
fn nl_default_skips_blank_lines() {
    // Default mode: blank line emitted as bare newline with no number prefix
    Command::cargo_bin("nl").unwrap()
        .write_stdin("a\n\nb\n")
        .assert().success()
        .stdout("     1\ta\n\n     2\tb\n");
}

#[test]
fn nl_skips_blank_lines() {
    // Duplicate for must_haves contains check
    Command::cargo_bin("nl").unwrap()
        .write_stdin("a\n\nb\n")
        .assert().success()
        .stdout("     1\ta\n\n     2\tb\n");
}

#[test]
fn nl_b_a_numbers_all_lines() {
    Command::cargo_bin("nl").unwrap()
        .args(["-b", "a"])
        .write_stdin("a\n\nb\n")
        .assert().success()
        .stdout("     1\ta\n     2\t\n     3\tb\n");
}

#[test]
fn nl_b_n_numbers_no_lines() {
    Command::cargo_bin("nl").unwrap()
        .args(["-b", "n"])
        .write_stdin("a\nb\n")
        .assert().success()
        .stdout("\ta\n\tb\n");
}

#[test]
fn nl_w_3_narrow_field() {
    Command::cargo_bin("nl").unwrap()
        .args(["-w", "3"])
        .write_stdin("a\nb\n")
        .assert().success()
        .stdout("  1\ta\n  2\tb\n");
}

#[test]
fn nl_custom_separator() {
    Command::cargo_bin("nl").unwrap()
        .args(["-s", ": "])
        .write_stdin("a\nb\n")
        .assert().success()
        .stdout("     1: a\n     2: b\n");
}

#[test]
fn nl_reads_file() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "line1").unwrap();
    writeln!(tmp, "line2").unwrap();
    let path = tmp.path().to_str().unwrap().to_string();
    Command::cargo_bin("nl").unwrap()
        .arg(&path)
        .assert().success()
        .stdout("     1\tline1\n     2\tline2\n");
}

#[test]
fn nl_invalid_body_numbering() {
    Command::cargo_bin("nl").unwrap()
        .args(["-b", "x"])
        .write_stdin("")
        .assert().failure();
}
