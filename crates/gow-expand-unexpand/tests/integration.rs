use assert_cmd::Command;

#[test]
fn expand_replaces_tabs() {
    // 'a' is at column 0; tab pads to next multiple of 8 = col 8 (7 spaces)
    Command::cargo_bin("expand").unwrap()
        .write_stdin("a\tb\n")
        .assert().success()
        .stdout("a       b\n");
}

#[test]
fn expand_full_tab_at_start() {
    // Tab at column 0 pads to column 8 (8 spaces)
    Command::cargo_bin("expand").unwrap()
        .write_stdin("\tx\n")
        .assert().success()
        .stdout("        x\n");
}

#[test]
fn expand_custom_tabstop() {
    // 'a' at col 0, -t 4: pad to col 4 (3 spaces)
    Command::cargo_bin("expand").unwrap()
        .args(["-t", "4"])
        .write_stdin("a\tb\n")
        .assert().success()
        .stdout("a   b\n");
}

#[test]
fn expand_no_tabs_passthrough() {
    Command::cargo_bin("expand").unwrap()
        .write_stdin("plain\n")
        .assert().success()
        .stdout("plain\n");
}

#[test]
fn expand_tab_at_column_7() {
    // "abcdefg" is 7 chars (col 7); tab pads to col 8 (1 space)
    Command::cargo_bin("expand").unwrap()
        .write_stdin("abcdefg\tx\n")
        .assert().success()
        .stdout("abcdefg x\n");
}

#[test]
fn unexpand_leading_spaces_to_tab() {
    // 8 leading spaces -> 1 tab (default tabstop 8)
    Command::cargo_bin("unexpand").unwrap()
        .write_stdin("        x\n")
        .assert().success()
        .stdout("\tx\n");
}

#[test]
fn unexpand_partial_leading_kept() {
    // Only 4 spaces, less than tabstop 8 — kept as-is
    Command::cargo_bin("unexpand").unwrap()
        .write_stdin("    x\n")
        .assert().success()
        .stdout("    x\n");
}

#[test]
fn unexpand_custom_tabstop() {
    // 8 spaces with tabstop 4 -> 2 tabs
    Command::cargo_bin("unexpand").unwrap()
        .args(["-t", "4"])
        .write_stdin("        x\n")
        .assert().success()
        .stdout("\t\tx\n");
}

#[test]
fn unexpand_default_only_leading() {
    // Mid-line spaces should NOT be converted by default
    Command::cargo_bin("unexpand").unwrap()
        .write_stdin("a        b\n")
        .assert().success()
        .stdout("a        b\n");
}

#[test]
fn unexpand_all_blanks() {
    // -a converts all space groups; 8 spaces between a and b at col 1 -> tab
    Command::cargo_bin("unexpand").unwrap()
        .arg("-a")
        .write_stdin("a        b\n")
        .assert().success()
        .stdout("a\tb\n");
}

#[test]
fn dispatch_distinct_binaries() {
    // expand and unexpand should produce different output for appropriate inputs
    let expand_out = Command::cargo_bin("expand").unwrap()
        .write_stdin("\tx\n")
        .assert().success()
        .get_output()
        .stdout
        .clone();
    let unexpand_out = Command::cargo_bin("unexpand").unwrap()
        .write_stdin("        x\n")
        .assert().success()
        .get_output()
        .stdout
        .clone();
    assert_ne!(expand_out, unexpand_out, "expand and unexpand produced identical outputs");
}
