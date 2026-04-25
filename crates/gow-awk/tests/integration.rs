use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// Helper to run awk with stdin input
fn awk_stdin(program: &str, input: &str) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("awk").unwrap();
    cmd.arg(program).write_stdin(input).assert()
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_print_field: {print $2} on "hello world" → "world\n"
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_print_field() {
    awk_stdin("{print $2}", "hello world\n")
        .success()
        .stdout("world\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_nf: {print NF} on "hello world" → "2\n"
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_nf() {
    awk_stdin("{print NF}", "hello world\n")
        .success()
        .stdout("2\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_nr_end: END{print NR} on 3-line input → "3\n"
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_nr_end() {
    awk_stdin("END{print NR}", "a\nb\nc\n")
        .success()
        .stdout("3\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_sum: sum fields with for loop on "1 2 3" → "6\n"
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_sum() {
    awk_stdin("{sum=0; for(i=1;i<=NF;i++) sum+=$i; print sum}", "1 2 3\n")
        .success()
        .stdout("6\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_associative_array: count[word]++ pattern → count "a" 2, "b" 1
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_associative_array() {
    let result = Command::cargo_bin("awk")
        .unwrap()
        .arg("{count[$1]++} END{for(k in count) print k, count[k]}")
        .write_stdin("a\na\nb\n")
        .assert()
        .success();

    // Order may vary, check both lines are present
    result
        .stdout(predicate::str::contains("a 2"))
        .stdout(predicate::str::contains("b 1"));
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_field_separator: -F: on "a:b:c" → print $2 = "b\n"
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_field_separator() {
    Command::cargo_bin("awk")
        .unwrap()
        .args(["-F:", "{print $2}"])
        .write_stdin("a:b:c\n")
        .assert()
        .success()
        .stdout("b\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_printf: printf "%05d\n" on 5 → "00005\n"
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_printf() {
    awk_stdin(r#"{printf "%05d\n", $1}"#, "5\n")
        .success()
        .stdout("00005\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_pattern_match: /foo/ pattern prints only matching lines
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_pattern_match() {
    awk_stdin("/foo/{print $0}", "foo bar\nhello\nfoo baz\n")
        .success()
        .stdout("foo bar\nfoo baz\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_variable: -v x=10 adds x to first field
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_variable() {
    Command::cargo_bin("awk")
        .unwrap()
        .args(["-v", "x=10", "{print $1 + x}"])
        .write_stdin("5\n")
        .assert()
        .success()
        .stdout("15\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_begin_end: BEGIN and END both print
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_begin_end() {
    awk_stdin("BEGIN{print \"start\"} END{print \"done\"}", "line\n")
        .success()
        .stdout("start\ndone\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_nr_filter: NR==2 prints only second line
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_nr_filter() {
    awk_stdin("NR==2{print}", "first\nsecond\nthird\n")
        .success()
        .stdout("second\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_file_input: read from a file
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_file_input() {
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("data.txt");
    fs::write(&file_path, "alpha\nbeta\ngamma\n").unwrap();

    Command::cargo_bin("awk")
        .unwrap()
        .args(["{print $1}", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout("alpha\nbeta\ngamma\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_print_no_args: print with no args prints $0
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_print_no_args() {
    awk_stdin("{print}", "hello world\n")
        .success()
        .stdout("hello world\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// test_awk_string_concat: concatenation in print
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_awk_string_concat() {
    awk_stdin(r#"{print $1 "!" $2}"#, "hello world\n")
        .success()
        .stdout("hello!world\n");
}
