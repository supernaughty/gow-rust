// Integration tests for gow-bzip2 (R019: bzip2/bunzip2 coverage).
// Tests use assert_cmd to spawn the compiled binary and tempfile for isolation.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Helper: create a bzip2 Command pointing at the compiled binary.
fn bzip2_cmd() -> Command {
    Command::cargo_bin("bzip2").expect("bzip2 binary not found — run `cargo build -p gow-bzip2` first")
}

/// Helper: write a fixture file and return its path.
fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}

/// Round-trip: bzip2 compresses, bzip2 -d decompresses, original bytes recovered.
#[test]
fn compress_decompress_roundtrip() {
    let tmp = tempdir().unwrap();
    let original = b"hello bzip2 world\nline two\nthird line\n";
    let input_path = write_fixture(tmp.path(), "roundtrip.txt", original);
    let compressed_path = tmp.path().join("roundtrip.txt.bz2");
    let output_path = tmp.path().join("roundtrip.txt");

    // Compress: roundtrip.txt → roundtrip.txt.bz2, original removed
    bzip2_cmd()
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    assert!(compressed_path.exists(), ".bz2 output should exist after compress");
    assert!(!input_path.exists(), "original should be removed by default");

    // Decompress: roundtrip.txt.bz2 → roundtrip.txt
    bzip2_cmd()
        .arg("-d")
        .arg(compressed_path.to_str().unwrap())
        .assert()
        .success();

    assert!(output_path.exists(), "decompressed file should exist");
    assert!(!compressed_path.exists(), ".bz2 should be removed after decompress");

    let recovered = fs::read(&output_path).unwrap();
    assert_eq!(recovered, original, "round-trip must recover original bytes exactly");
}

/// -c / --stdout: compress to stdout, original file must still exist.
#[test]
fn compress_to_stdout() {
    let tmp = tempdir().unwrap();
    let contents = b"compress to stdout test content";
    let input_path = write_fixture(tmp.path(), "stdout_test.txt", contents);

    let output = bzip2_cmd()
        .arg("-c")
        .arg(input_path.to_str().unwrap())
        .output()
        .expect("failed to run bzip2 -c");

    assert!(output.status.success(), "bzip2 -c must exit 0");
    assert!(!output.stdout.is_empty(), "stdout must contain compressed bytes");
    assert!(
        input_path.exists(),
        "original must be preserved when -c is used"
    );

    // Verify the compressed output is valid bzip2 (starts with BZ magic bytes)
    assert_eq!(&output.stdout[0..2], b"BZ", "output must start with bzip2 magic BZh");
}

/// -k / --keep: compress but keep the original file alongside the .bz2.
#[test]
fn keep_flag_preserves_original() {
    let tmp = tempdir().unwrap();
    let contents = b"keep original after compression";
    let input_path = write_fixture(tmp.path(), "keep_test.txt", contents);
    let compressed_path = tmp.path().join("keep_test.txt.bz2");

    bzip2_cmd()
        .arg("-k")
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    assert!(compressed_path.exists(), ".bz2 output must exist");
    assert!(input_path.exists(), "original must still exist with -k flag");

    // Verify original content unchanged
    let original_after = fs::read(&input_path).unwrap();
    assert_eq!(original_after, contents, "original file content must be unchanged");
}

/// -d flag decompresses (equivalent to bunzip2): bzip2 -d file.bz2 → file.
#[test]
fn decompress_flag() {
    let tmp = tempdir().unwrap();
    let original = b"decompress via -d flag test";
    let input_path = write_fixture(tmp.path(), "decompress_test.txt", original);
    let compressed_path = tmp.path().join("decompress_test.txt.bz2");

    // Compress first (keeping original so we have both)
    bzip2_cmd()
        .arg("-k")
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    // Remove original so -d can recreate it
    fs::remove_file(&input_path).unwrap();

    // Decompress using -d flag
    bzip2_cmd()
        .arg("-d")
        .arg(compressed_path.to_str().unwrap())
        .assert()
        .success();

    assert!(input_path.exists(), "decompressed file must exist after bzip2 -d");
    let recovered = fs::read(&input_path).unwrap();
    assert_eq!(recovered, original, "bzip2 -d must recover original bytes");
}

/// Decompress to stdout with -d -c: bzip2 -d -c file.bz2 → stdout, .bz2 preserved.
#[test]
fn decompress_to_stdout() {
    let tmp = tempdir().unwrap();
    let original = b"decompress to stdout content\n";
    let input_path = write_fixture(tmp.path(), "dc_test.txt", original);
    let compressed_path = tmp.path().join("dc_test.txt.bz2");

    // Compress first
    bzip2_cmd()
        .arg("-k")
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    // Decompress to stdout
    let out = bzip2_cmd()
        .args(["-d", "-c", compressed_path.to_str().unwrap()])
        .output()
        .expect("failed to run bzip2 -d -c");

    assert!(out.status.success(), "bzip2 -d -c must exit 0");
    assert_eq!(out.stdout, original, "stdout must contain decompressed bytes");
    assert!(compressed_path.exists(), ".bz2 file must be preserved with -c");
}

/// Decompressing a file without .bz2 suffix must exit non-zero with error message.
#[test]
fn decompress_file_without_bz2_suffix_fails() {
    let tmp = tempdir().unwrap();
    let path = write_fixture(tmp.path(), "plain.txt", b"not compressed");

    bzip2_cmd()
        .arg("-d")
        .arg(path.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown suffix").or(predicate::str::contains("ignored")));
}

/// Decompressing a plain (non-bzip2) .bz2 file must exit non-zero.
#[test]
fn decompress_non_bzip2_data_fails() {
    let tmp = tempdir().unwrap();
    // Write a file with .bz2 extension but not actual bzip2 data
    let fake_bz2 = write_fixture(tmp.path(), "fake.bz2", b"this is not bzip2 data");

    bzip2_cmd()
        .arg("-d")
        .arg(fake_bz2.to_str().unwrap())
        .assert()
        .failure();
}

/// Multiple input files: each compressed independently.
#[test]
fn compress_multiple_files() {
    let tmp = tempdir().unwrap();
    let file1 = write_fixture(tmp.path(), "file1.txt", b"first file content");
    let file2 = write_fixture(tmp.path(), "file2.txt", b"second file content");
    let bz2_1 = tmp.path().join("file1.txt.bz2");
    let bz2_2 = tmp.path().join("file2.txt.bz2");

    bzip2_cmd()
        .arg(file1.to_str().unwrap())
        .arg(file2.to_str().unwrap())
        .assert()
        .success();

    assert!(bz2_1.exists(), "file1.txt.bz2 must exist");
    assert!(bz2_2.exists(), "file2.txt.bz2 must exist");
    assert!(!file1.exists(), "file1.txt original removed");
    assert!(!file2.exists(), "file2.txt original removed");
}
