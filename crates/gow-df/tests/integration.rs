//! Integration tests for gow-df (U-09).

use assert_cmd::Command;

/// df with no args runs successfully and output contains "Filesystem" + "1K-blocks" header
#[test]
fn df_runs_without_error() {
    let assert = Command::cargo_bin("df").unwrap().assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Filesystem"), "missing header: {stdout}");
    assert!(stdout.contains("1K-blocks"), "missing 1K-blocks column: {stdout}");
    assert!(stdout.contains("Mounted on"), "missing Mounted on column: {stdout}");
}

/// df -h runs successfully and output header contains "Size" and "Avail"
#[test]
fn df_human_readable_header() {
    let assert = Command::cargo_bin("df").unwrap().arg("-h").assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("Size"), "missing Size column: {stdout}");
    assert!(stdout.contains("Avail"), "missing Avail column: {stdout}");
}

/// df output has at least 1 data row (C:\ should always respond on Windows)
#[test]
fn df_at_least_one_drive() {
    let assert = Command::cargo_bin("df").unwrap().assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let line_count = stdout.lines().count();
    // 1 header + at least 1 data row
    assert!(
        line_count >= 2,
        "expected at least 2 lines (header + one drive): {stdout}"
    );
}

/// df output contains a drive letter followed by ":\"
#[test]
fn df_drive_letter_appears() {
    let assert = Command::cargo_bin("df").unwrap().assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains(":\\"), "no drive letter in output: {stdout}");
}

/// df output contains a Use% column (percentage sign)
#[test]
fn df_percent_column_present() {
    let assert = Command::cargo_bin("df").unwrap().assert().success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains('%'), "no percent column: {stdout}");
}
