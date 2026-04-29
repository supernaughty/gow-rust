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
fn join_basic() {
    let f1 = write_temp("a x\nb y\n");
    let f2 = write_temp("a 1\nb 2\n");
    Command::cargo_bin("join")
        .unwrap()
        .arg(f1.path())
        .arg(f2.path())
        .assert()
        .success()
        .stdout("a x 1\nb y 2\n");
}

#[test]
fn join_custom_fields() {
    // file1: join on field 2; file2: join on field 1
    let f1 = write_temp("x a\ny b\n");
    let f2 = write_temp("a 1\nb 2\n");
    Command::cargo_bin("join")
        .unwrap()
        .args(["-1", "2", "-2", "1"])
        .arg(f1.path())
        .arg(f2.path())
        .assert()
        .success()
        .stdout("a x 1\nb y 2\n");
}

#[test]
fn join_colon_separator() {
    let f1 = write_temp("a:x\nb:y\n");
    let f2 = write_temp("a:1\nb:2\n");
    Command::cargo_bin("join")
        .unwrap()
        .arg("-t:")
        .arg(f1.path())
        .arg(f2.path())
        .assert()
        .success()
        .stdout("a:x:1\nb:y:2\n");
}

#[test]
fn join_print_unmatched_a1() {
    // file1 has extra "c z" with no match in file2 — -a 1 prints it
    let f1 = write_temp("a x\nb y\nc z\n");
    let f2 = write_temp("a 1\nb 2\n");
    let out = Command::cargo_bin("join")
        .unwrap()
        .args(["-a", "1"])
        .arg(f1.path())
        .arg(f2.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("a x 1"), "matched line must appear: {:?}", s);
    assert!(s.contains("c z"), "unmatched file1 line must appear: {:?}", s);
}

#[test]
fn join_missing_file_exits_1() {
    let f2 = write_temp("a 1\n");
    Command::cargo_bin("join")
        .unwrap()
        .arg("nonexistent_xyz_join_98765.txt")
        .arg(f2.path())
        .assert()
        .failure()
        .code(1)
        .stderr(contains("join:"));
}
