use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;
use tempfile::NamedTempFile;

fn write_temp(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

#[test]
fn paste_two_files() {
    let f1 = write_temp("a\nb\n");
    let f2 = write_temp("c\nd\n");
    Command::cargo_bin("paste")
        .unwrap()
        .arg(f1.path())
        .arg(f2.path())
        .assert()
        .success()
        .stdout("a\tc\nb\td\n");
}

#[test]
fn paste_comma_delimiter() {
    let f1 = write_temp("a\nb\n");
    let f2 = write_temp("c\nd\n");
    Command::cargo_bin("paste")
        .unwrap()
        .arg("-d,")
        .arg(f1.path())
        .arg(f2.path())
        .assert()
        .success()
        .stdout("a,c\nb,d\n");
}

#[test]
fn paste_stdin_double_dash() {
    // paste - - splits stdin into 2 columns, alternating lines
    Command::cargo_bin("paste")
        .unwrap()
        .arg("-")
        .arg("-")
        .write_stdin("1\n2\n3\n4\n")
        .assert()
        .success()
        .stdout("1\t2\n3\t4\n");
}

#[test]
fn paste_unequal_line_counts() {
    // file1 has 3 lines, file2 has 2 — third row: "c\t\n" (empty for exhausted file2)
    let f1 = write_temp("a\nb\nc\n");
    let f2 = write_temp("x\ny\n");
    let out = Command::cargo_bin("paste")
        .unwrap()
        .arg(f1.path())
        .arg(f2.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.starts_with("a\tx\nb\ty\n"), "first two rows: {:?}", s);
    // Third row must have "c" and an empty second column
    assert!(s.contains("c\t"), "third row must contain 'c\\t': {:?}", s);
}

#[test]
fn paste_missing_file_exits_1() {
    Command::cargo_bin("paste")
        .unwrap()
        .arg("nonexistent_xyz_paste_98765.txt")
        .assert()
        .failure()
        .code(1)
        .stderr(contains("paste:"));
}

#[test]
fn paste_stdin_single_column_passthrough() {
    // paste with no file args = read stdin as single column (no join, no delimiter added)
    Command::cargo_bin("paste")
        .unwrap()
        .write_stdin("hello\nworld\n")
        .assert()
        .success()
        .stdout("hello\nworld\n");
}
