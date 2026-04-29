use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

fn write_temp(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

#[test]
fn split_by_lines() {
    let f = write_temp("a\nb\nc\nd\ne\nf\ng\n"); // 7 lines
    let dir = TempDir::new().unwrap();
    let prefix = dir.path().join("x").to_str().unwrap().to_string();
    Command::cargo_bin("split")
        .unwrap()
        .args(["-l", "3", f.path().to_str().unwrap(), &prefix])
        .assert()
        .success();
    let xaa = std::fs::read_to_string(format!("{prefix}aa")).unwrap();
    let xab = std::fs::read_to_string(format!("{prefix}ab")).unwrap();
    let xac = std::fs::read_to_string(format!("{prefix}ac")).unwrap();
    assert_eq!(xaa, "a\nb\nc\n");
    assert_eq!(xab, "d\ne\nf\n");
    assert_eq!(xac, "g\n");
}

#[test]
fn split_by_bytes() {
    let f = write_temp("0123456789"); // 10 bytes, no newline
    let dir = TempDir::new().unwrap();
    let prefix = dir.path().join("x").to_str().unwrap().to_string();
    Command::cargo_bin("split")
        .unwrap()
        .args(["-b", "5", f.path().to_str().unwrap(), &prefix])
        .assert()
        .success();
    let xaa = std::fs::read(format!("{prefix}aa")).unwrap();
    let xab = std::fs::read(format!("{prefix}ab")).unwrap();
    assert_eq!(xaa, b"01234");
    assert_eq!(xab, b"56789");
}

#[test]
fn split_default_1000_lines_single_chunk() {
    let f = write_temp("a\nb\nc\nd\ne\n"); // 5 lines < 1000
    let dir = TempDir::new().unwrap();
    let prefix = dir.path().join("x").to_str().unwrap().to_string();
    Command::cargo_bin("split")
        .unwrap()
        .args([f.path().to_str().unwrap(), &prefix])
        .assert()
        .success();
    let xaa = std::fs::read_to_string(format!("{prefix}aa")).unwrap();
    assert_eq!(xaa, "a\nb\nc\nd\ne\n");
}

#[test]
fn split_custom_prefix() {
    let f = write_temp("hello\nworld\n");
    let dir = TempDir::new().unwrap();
    let prefix = dir.path().join("part").to_str().unwrap().to_string();
    Command::cargo_bin("split")
        .unwrap()
        .args(["-l", "1", f.path().to_str().unwrap(), &prefix])
        .assert()
        .success();
    assert!(std::path::Path::new(&format!("{prefix}aa")).exists());
    assert!(std::path::Path::new(&format!("{prefix}ab")).exists());
}

#[test]
fn split_chunks_n() {
    let f = write_temp("0123456789"); // 10 bytes
    let dir = TempDir::new().unwrap();
    let prefix = dir.path().join("x").to_str().unwrap().to_string();
    Command::cargo_bin("split")
        .unwrap()
        .args(["-n", "2", f.path().to_str().unwrap(), &prefix])
        .assert()
        .success();
    let xaa = std::fs::read(format!("{prefix}aa")).unwrap();
    let xab = std::fs::read(format!("{prefix}ab")).unwrap();
    assert_eq!(xaa.len() + xab.len(), 10);
}

#[test]
fn split_zero_lines_exits_1() {
    let f = write_temp("a\n");
    let dir = TempDir::new().unwrap();
    let prefix = dir.path().join("x").to_str().unwrap().to_string();
    Command::cargo_bin("split")
        .unwrap()
        .args(["-l", "0", f.path().to_str().unwrap(), &prefix])
        .assert()
        .failure()
        .code(1)
        .stderr(contains("split:"));
}
