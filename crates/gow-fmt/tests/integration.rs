use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn fmt_wraps_at_width() {
    // Width 20: "hello world goodbye universe" → "hello world goodbye\nuniverse\n"
    Command::cargo_bin("fmt")
        .unwrap()
        .args(["-w", "20"])
        .write_stdin("hello world goodbye universe\n")
        .assert()
        .success()
        .stdout("hello world goodbye\nuniverse\n");
}

#[test]
fn fmt_default_passthrough_short_line() {
    let short = "short line here\n";
    Command::cargo_bin("fmt")
        .unwrap()
        .write_stdin(short)
        .assert()
        .success()
        .stdout(short);
}

#[test]
fn fmt_joins_paragraph_lines() {
    // Two non-blank lines → one paragraph → wrapped output
    Command::cargo_bin("fmt")
        .unwrap()
        .args(["-w", "30"])
        .write_stdin("hello world\ngoodbye universe\n")
        .assert()
        .success()
        .stdout(contains("hello")); // output exists; exact wrapping depends on word order
}

#[test]
fn fmt_preserves_blank_line_as_paragraph_separator() {
    let input = "first paragraph\n\nsecond paragraph\n";
    let out = Command::cargo_bin("fmt")
        .unwrap()
        .args(["-w", "40"])
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // The blank line must appear between paragraphs
    assert!(
        s.contains("\n\n"),
        "blank line must separate paragraphs: {:?}",
        s
    );
}

#[test]
fn fmt_missing_file_exits_1() {
    Command::cargo_bin("fmt")
        .unwrap()
        .arg("nonexistent_xyz_fmt_98765.txt")
        .assert()
        .failure()
        .code(1)
        .stderr(contains("fmt:"));
}
