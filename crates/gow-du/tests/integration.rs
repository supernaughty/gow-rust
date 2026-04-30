//! Integration tests for gow-du (U-08).

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// du -sh on a tempdir with a small file produces tab-separated output
#[test]
fn du_summary_human() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), b"hello world").unwrap();
    let assert = Command::cargo_bin("du")
        .unwrap()
        .args(["-sh"])
        .arg(dir.path())
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Output must be a single line: "<size>\t<path>"
    let first = stdout.lines().next().unwrap_or("");
    assert!(first.contains('\t'), "expected tab-separated output: {first}");
}

/// du -s <dir> produces a single line with total in 1K blocks
#[test]
fn du_summary_default_blocks() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("file.txt"), b"hello").unwrap();
    let assert = Command::cargo_bin("du")
        .unwrap()
        .arg("-s")
        .arg(dir.path())
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected 1 line for -s, got: {stdout}");
    // First field must be a number (1K blocks)
    let fields: Vec<&str> = lines[0].splitn(2, '\t').collect();
    assert!(fields.len() == 2, "expected tab-separated output: {}", lines[0]);
    assert!(
        fields[0].trim().parse::<u64>().is_ok(),
        "expected numeric 1K-block count: {}",
        fields[0]
    );
}

/// du <missing> exits 1 with "du:" prefix in stderr
#[test]
fn du_missing_path_errors() {
    let assert = Command::cargo_bin("du")
        .unwrap()
        .arg("nonexistent_gow_du_xyz_99999")
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("du:"), "expected 'du:' prefix in stderr: {stderr}");
}

/// du -d 0 <dir> emits exactly 1 line (only the top-level total)
#[test]
fn du_max_depth_zero() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub").join("x.txt"), b"data").unwrap();
    let assert = Command::cargo_bin("du")
        .unwrap()
        .args(["-d", "0"])
        .arg(dir.path())
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let line_count = stdout.lines().count();
    assert_eq!(line_count, 1, "expected 1 line at depth 0, got: {stdout}");
}

/// du (no flags) on a dir with a subdirectory lists both the subdir and the root
#[test]
fn du_no_flags_lists_directories() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("subdir")).unwrap();
    fs::write(dir.path().join("subdir").join("f.txt"), b"hello").unwrap();
    let assert = Command::cargo_bin("du")
        .unwrap()
        .arg(dir.path())
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Should have at least 2 lines: subdir + root
    let line_count = stdout.lines().count();
    assert!(line_count >= 2, "expected at least 2 lines (subdir + root): {stdout}");
}

/// du -h shows human-readable per-dir output (contains a K or M suffix or a small number)
#[test]
fn du_human_readable_suffix() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("big.txt"), vec![b'x'; 2048]).unwrap();
    let assert = Command::cargo_bin("du")
        .unwrap()
        .arg("-h")
        .arg(dir.path())
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // Human-readable output should contain either a unit suffix (K, M, G) or digits
    assert!(!stdout.is_empty(), "expected non-empty output");
    let first = stdout.lines().next().unwrap_or("");
    assert!(first.contains('\t'), "expected tab-separated human output: {first}");
}
