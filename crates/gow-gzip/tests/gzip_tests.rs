// Integration tests for gow-gzip (R019 coverage).
// Tests cover: compress/decompress round-trip, stdout mode, keep flag, zcat, -d flag, error handling.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ── Helper functions ──────────────────────────────────────────────────────────

fn gzip_cmd() -> Command {
    Command::cargo_bin("gzip").expect("gzip binary not found — run `cargo build -p gow-gzip` first")
}

fn gunzip_cmd() -> Command {
    Command::cargo_bin("gunzip")
        .expect("gunzip binary not found — run `cargo build -p gow-gzip` first")
}

fn zcat_cmd() -> Command {
    Command::cargo_bin("zcat")
        .expect("zcat binary not found — run `cargo build -p gow-gzip` first")
}

fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// gzip file.txt → file.txt.gz (original removed), gunzip file.txt.gz → file.txt matches original.
#[test]
fn compress_decompress_roundtrip() {
    let tmp = tempdir().unwrap();
    let original = b"hello world\nline 2\n";
    let input_path = write_fixture(tmp.path(), "input.txt", original);
    let compressed_path = tmp.path().join("input.txt.gz");
    let output_path = tmp.path().join("input.txt");

    // Compress: gzip input.txt → input.txt.gz, removes input.txt
    gzip_cmd()
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    assert!(
        compressed_path.exists(),
        "compressed file input.txt.gz should exist after gzip"
    );
    assert!(
        !input_path.exists(),
        "original input.txt should be removed after gzip (no -k)"
    );

    // Decompress: gunzip input.txt.gz → input.txt, removes .gz
    gunzip_cmd()
        .arg(compressed_path.to_str().unwrap())
        .assert()
        .success();

    assert!(
        output_path.exists(),
        "decompressed input.txt should exist after gunzip"
    );
    assert!(
        !compressed_path.exists(),
        "input.txt.gz should be removed after gunzip (no -k)"
    );

    let recovered = fs::read(&output_path).unwrap();
    assert_eq!(
        recovered, original,
        "round-trip: decompressed bytes must match original"
    );
}

/// gzip -c file.txt → stdout is non-empty, original file still exists.
#[test]
fn compress_to_stdout_keeps_original() {
    let tmp = tempdir().unwrap();
    let original = b"compress stdout test\n";
    let input_path = write_fixture(tmp.path(), "input.txt", original);

    let output = gzip_cmd()
        .arg("-c")
        .arg(input_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success(), "gzip -c should exit 0");
    assert!(
        !output.stdout.is_empty(),
        "gzip -c should write compressed bytes to stdout"
    );
    assert!(
        input_path.exists(),
        "original file must remain when using -c/--stdout"
    );
}

/// gzip -k file.txt → file.txt.gz created AND file.txt still exists.
#[test]
fn keep_flag_preserves_original() {
    let tmp = tempdir().unwrap();
    let original = b"keep flag test data\n";
    let input_path = write_fixture(tmp.path(), "input.txt", original);
    let compressed_path = tmp.path().join("input.txt.gz");

    gzip_cmd()
        .arg("-k")
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    assert!(
        compressed_path.exists(),
        "input.txt.gz should be created by gzip -k"
    );
    assert!(
        input_path.exists(),
        "original input.txt must still exist with -k/--keep"
    );
}

/// zcat file.txt.gz → stdout equals original content, .gz file still exists.
#[test]
fn zcat_decompresses_to_stdout() {
    let tmp = tempdir().unwrap();
    let original = b"zcat output test\nwith two lines\n";
    let input_path = write_fixture(tmp.path(), "input.txt", original);
    let compressed_path = tmp.path().join("input.txt.gz");

    // Create .gz file using gzip -k (keep original)
    gzip_cmd()
        .arg("-k")
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    assert!(compressed_path.exists(), "input.txt.gz must exist before zcat test");

    // zcat decompresses to stdout without removing the file
    let output = zcat_cmd()
        .arg(compressed_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success(), "zcat should exit 0");
    assert_eq!(
        output.stdout, original,
        "zcat stdout must match original bytes"
    );
    assert!(
        compressed_path.exists(),
        "zcat must not remove the .gz file"
    );
}

/// gzip -d file.txt.gz → same result as gunzip (decompress flag acts as gunzip).
#[test]
fn decompress_flag_acts_as_gunzip() {
    let tmp = tempdir().unwrap();
    let original = b"decompress flag test data\n";
    let input_path = write_fixture(tmp.path(), "file.txt", original);
    let compressed_path = tmp.path().join("file.txt.gz");
    let output_path = tmp.path().join("file.txt");

    // Compress first
    gzip_cmd()
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    assert!(compressed_path.exists());

    // Decompress using gzip -d
    gzip_cmd()
        .arg("-d")
        .arg(compressed_path.to_str().unwrap())
        .assert()
        .success();

    assert!(
        output_path.exists(),
        "file.txt should be recovered after gzip -d"
    );
    assert!(
        !compressed_path.exists(),
        "file.txt.gz should be removed after gzip -d"
    );

    let recovered = fs::read(&output_path).unwrap();
    assert_eq!(
        recovered, original,
        "gzip -d must recover original bytes"
    );
}

/// gunzip on a plain text (non-gzip) file → exits with code != 0.
#[test]
fn gunzip_rejects_non_gzip_file() {
    let tmp = tempdir().unwrap();
    // Write a plain text file (not gzip compressed)
    let bad_path = write_fixture(tmp.path(), "notgzip.txt.gz", b"this is not gzip\n");

    gunzip_cmd()
        .arg(bad_path.to_str().unwrap())
        .assert()
        .failure()
        .code(predicate::ne(0));
}

/// gzip stdin → stdout is a valid gzip stream (non-empty, starts with gzip magic bytes).
#[test]
fn gzip_compress_stdin_to_stdout() {
    let input_data = b"stdin compression test data\n";

    let output = gzip_cmd()
        .write_stdin(input_data.as_ref())
        .output()
        .unwrap();

    assert!(output.status.success(), "gzip (stdin) should exit 0");
    assert!(
        !output.stdout.is_empty(),
        "gzip stdin should produce compressed output"
    );
    // Gzip magic bytes: 0x1f 0x8b
    assert_eq!(
        &output.stdout[..2],
        &[0x1f, 0x8b],
        "gzip output must start with gzip magic bytes 1f 8b"
    );
}

/// gzip -k with multiple files: all compressed, all originals preserved.
#[test]
fn keep_flag_with_multiple_files() {
    let tmp = tempdir().unwrap();
    let file1 = write_fixture(tmp.path(), "a.txt", b"file one\n");
    let file2 = write_fixture(tmp.path(), "b.txt", b"file two\n");

    gzip_cmd()
        .arg("-k")
        .arg(file1.to_str().unwrap())
        .arg(file2.to_str().unwrap())
        .assert()
        .success();

    assert!(tmp.path().join("a.txt.gz").exists(), "a.txt.gz should exist");
    assert!(tmp.path().join("b.txt.gz").exists(), "b.txt.gz should exist");
    assert!(file1.exists(), "a.txt should still exist with -k");
    assert!(file2.exists(), "b.txt should still exist with -k");
}
