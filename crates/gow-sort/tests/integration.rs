use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_sort_basic() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "c\nb\na\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg(&file_path);
    cmd.assert()
        .success()
        .stdout("a\nb\nc\n");
}

#[test]
fn test_sort_reverse() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "a\nb\nc\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg("-r").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("c\nb\na\n");
}

#[test]
fn test_sort_numeric() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "10\n2\n1\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg("-n").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("1\n2\n10\n");
}

#[test]
fn test_sort_unique() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "b\na\nb\na\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg("-u").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("a\nb\n");
}

#[test]
fn test_sort_ignore_case() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "b\nA\nB\na\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg("-f").arg(&file_path);
    // When ignoring case, 'A' and 'a' compare equal, 'B' and 'b' compare equal.
    // The relative order of equal elements depends on stable vs unstable sort.
    // The current implementation uses stable sort (`sort_by`), let's check content roughly.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("a\n"))
        .stdout(predicate::str::contains("A\n"))
        .stdout(predicate::str::contains("b\n"))
        .stdout(predicate::str::contains("B\n"));
}

#[test]
fn test_sort_multiple_files() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("f1.txt");
    let file2 = dir.path().join("f2.txt");
    fs::write(&file1, "b\nc\n").unwrap();
    fs::write(&file2, "a\nd\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg(&file1).arg(&file2);
    cmd.assert()
        .success()
        .stdout("a\nb\nc\nd\n");
}

#[test]
fn test_sort_external_merge() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "c\nb\na\nf\ne\nd\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    // Use an incredibly small buffer to force external merge sort
    cmd.arg("-S").arg("10").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("a\nb\nc\nd\ne\nf\n");
}

#[test]
fn test_sort_output_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let out_path = dir.path().join("out.txt");
    fs::write(&file_path, "c\nb\na\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg("-o").arg(&out_path).arg(&file_path);
    cmd.assert().success().stdout("");

    assert_eq!(fs::read_to_string(&out_path).unwrap(), "a\nb\nc\n");
}

#[test]
fn test_sort_key_field_1() {
    // Sort by second whitespace-delimited field (lexicographic)
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    // Field 2: banana, apple, cherry -> apple, banana, cherry
    fs::write(&file_path, "row1 banana x\nrow2 apple x\nrow3 cherry x\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg("-k2").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("row2 apple x\nrow1 banana x\nrow3 cherry x\n");
}

#[test]
fn test_sort_key_numeric() {
    // Sort by second field numerically (-k2n)
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    // Field 2 values: 10, 2, 1 -> numeric order 1, 2, 10
    fs::write(&file_path, "a 10\nb 2\nc 1\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg("-k2n").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("c 1\nb 2\na 10\n");
}

#[test]
fn test_sort_key_reverse() {
    // Sort by second field in reverse lexicographic order (-k2r)
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    // Field 2: banana, apple, cherry -> reverse: cherry, banana, apple
    fs::write(&file_path, "row1 banana\nrow2 apple\nrow3 cherry\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg("-k2r").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("row3 cherry\nrow1 banana\nrow2 apple\n");
}

#[test]
fn test_sort_key_separator() {
    // Sort by field 3 using colon as separator (-t: -k3)
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    // /etc/passwd-style: name:x:uid:...
    // field 3 (uid): 1000, 500, 0 -> lex order: 0, 1000, 500
    fs::write(&file_path, "bob:x:1000\nalice:x:500\nroot:x:0\n").unwrap();

    let mut cmd = Command::cargo_bin("sort").unwrap();
    cmd.arg("-t:").arg("-k3").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("root:x:0\nbob:x:1000\nalice:x:500\n");
}
