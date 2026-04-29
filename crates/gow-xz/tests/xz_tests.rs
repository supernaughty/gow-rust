//! Integration tests for gow-xz (R019: xz/unxz coverage).
//!
//! Tests verify the round-trip compress/decompress using the xz binary.
//! All tests use tempdir to avoid polluting the filesystem.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn xz_cmd() -> Command {
    Command::cargo_bin("xz").unwrap()
}

fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Basic compress → decompress round-trip.
/// xz file.txt produces file.txt.xz (original gone),
/// then xz -d file.txt.xz recovers the original content.
#[test]
fn compress_decompress_roundtrip() {
    let tmp = tempdir().unwrap();
    let original_content = b"hello xz world\nline 2\nline 3\n";
    let input_path = write_fixture(tmp.path(), "input.txt", original_content);

    // Compress: xz input.txt → input.txt.xz, input.txt removed
    xz_cmd()
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    let compressed = tmp.path().join("input.txt.xz");
    assert!(compressed.exists(), "compressed file should exist");
    assert!(!input_path.exists(), "original should have been removed");

    // Decompress: xz -d input.txt.xz → input.txt
    xz_cmd()
        .arg("-d")
        .arg(compressed.to_str().unwrap())
        .assert()
        .success();

    assert!(input_path.exists(), "recovered file should exist");
    let recovered = fs::read(&input_path).unwrap();
    assert_eq!(recovered, original_content, "round-trip must preserve bytes exactly");
}

/// -c/--stdout mode: xz -c input.txt writes compressed bytes to stdout,
/// leaves the original file intact.
#[test]
fn compress_to_stdout() {
    let tmp = tempdir().unwrap();
    let input_path = write_fixture(tmp.path(), "data.txt", b"compress me to stdout\n");

    let output = xz_cmd()
        .arg("-c")
        .arg(input_path.to_str().unwrap())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(!output.is_empty(), "stdout should contain compressed bytes");
    // xz stream magic bytes: \xfd7zXZ\x00
    assert_eq!(&output[..6], b"\xfd7zXZ\x00", "xz stream magic should be present in stdout");
    assert!(input_path.exists(), "original must still exist when -c is used");
}

/// -k/--keep mode: xz -k file.txt produces file.txt.xz AND keeps file.txt.
#[test]
fn keep_flag_preserves_original() {
    let tmp = tempdir().unwrap();
    let input_path = write_fixture(tmp.path(), "keep_me.txt", b"keep this file\n");

    xz_cmd()
        .arg("-k")
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    let compressed = tmp.path().join("keep_me.txt.xz");
    assert!(compressed.exists(), "compressed file should exist");
    assert!(input_path.exists(), "original must still exist with -k flag");
}

/// -d/--decompress flag: decompress an .xz file back to original.
/// Equivalent to unxz behavior.
#[test]
fn decompress_flag() {
    let tmp = tempdir().unwrap();
    let original = b"decompress me back\n";
    let input_path = write_fixture(tmp.path(), "data.txt", original);

    // First compress to get an .xz file
    xz_cmd()
        .arg("-k")
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    let compressed = tmp.path().join("data.txt.xz");
    assert!(compressed.exists());

    // Now decompress using -d flag
    xz_cmd()
        .arg("-d")
        .arg(compressed.to_str().unwrap())
        .assert()
        .success();

    // data.txt should be restored (original kept because -k was used before)
    // The decompressed output path is derived from stripping .xz
    // Since we kept original, we need to verify content matches
    let recovered = fs::read(&input_path).unwrap();
    assert_eq!(recovered, original);
}

/// xz -d on a plain text file (not a valid xz stream) should exit with code 1.
#[test]
fn xz_rejects_non_xz_file() {
    let tmp = tempdir().unwrap();
    // Create a file with .xz extension but invalid content
    let bad_xz = write_fixture(tmp.path(), "notreal.xz", b"this is not xz data at all\n");

    xz_cmd()
        .arg("-d")
        .arg(bad_xz.to_str().unwrap())
        .assert()
        .failure()
        .code(1);
}

/// Binary content round-trip: compress/decompress preserves arbitrary binary bytes.
#[test]
fn roundtrip_binary_content() {
    let tmp = tempdir().unwrap();
    let binary_content: &[u8] = b"\x00\x01\x02\xff\xfe\xfd\x80\x7f\r\n\x00";
    let input_path = write_fixture(tmp.path(), "binary.bin", binary_content);

    // Compress
    xz_cmd()
        .arg(input_path.to_str().unwrap())
        .assert()
        .success();

    let compressed = tmp.path().join("binary.bin.xz");
    assert!(compressed.exists());
    assert!(!input_path.exists());

    // Decompress
    xz_cmd()
        .arg("-d")
        .arg(compressed.to_str().unwrap())
        .assert()
        .success();

    assert!(input_path.exists());
    let recovered = fs::read(&input_path).unwrap();
    assert_eq!(
        recovered, binary_content,
        "binary content round-trip must be byte-exact"
    );
}

/// xz on a file without .xz extension when decompressing should fail with error message.
#[test]
fn decompress_missing_xz_extension_fails() {
    let tmp = tempdir().unwrap();
    let plain_file = write_fixture(tmp.path(), "notxz.txt", b"plain text\n");

    xz_cmd()
        .arg("-d")
        .arg(plain_file.to_str().unwrap())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("xz"));
}

// ── WR-04: concatenated xz stream decompression ──────────────────────────────

/// Proves new_multi_decoder is required: XzDecoder::new() would only decode
/// the first stream and silently ignore all subsequent streams.
///
/// Constructs two concatenated xz streams inline (no binary fixture files per D-02),
/// writes them to a temp .xz file, then verifies xz -d -c outputs data from both streams.
#[test]
fn concatenated_xz_streams_decompress_fully() {
    use liblzma::write::XzEncoder;
    use std::io::Write;

    let dir = tempdir().unwrap();

    // Build two concatenated xz streams in memory
    let mut buf = Vec::new();

    // Stream 1
    {
        let mut enc = XzEncoder::new(&mut buf, 6);
        enc.write_all(b"xz stream one\n").unwrap();
        enc.finish().unwrap();
    }

    // Stream 2
    {
        let mut enc = XzEncoder::new(&mut buf, 6);
        enc.write_all(b"xz stream two\n").unwrap();
        enc.finish().unwrap();
    }

    // Write concatenated streams to a .xz file
    let xz_path = dir.path().join("multi.xz");
    fs::write(&xz_path, &buf).unwrap();

    // Decompress to stdout (-c flag) — new_multi_decoder should decode both streams
    let output = xz_cmd()
        .args(["-d", "-c", xz_path.to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let decoded = String::from_utf8(output).unwrap();
    assert!(
        decoded.contains("xz stream one"),
        "stream 1 data missing from output; got: {decoded:?}"
    );
    assert!(
        decoded.contains("xz stream two"),
        "stream 2 data missing from output — XzDecoder::new() would cause this; got: {decoded:?}"
    );
}
