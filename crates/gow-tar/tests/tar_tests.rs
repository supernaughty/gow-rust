// Integration tests for gow-tar (R018).
// Tests cover: -c/-x/-t modes with -z (gzip), -j (bzip2), and plain codec.
// All tests use tempfile for isolation and assert_cmd for binary invocation.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn tar_cmd() -> Command {
    Command::cargo_bin("tar").expect("tar binary not found — run `cargo build -p gow-tar` first")
}

fn write_fixture(dir: &std::path::Path, name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, contents).unwrap();
    path
}

// ─── Test 1: create and extract .tar.gz round-trip ───────────────────────────

#[test]
fn create_and_extract_tar_gz() {
    // Covers: tar -c -z -f creates .tar.gz; tar -x -z -f extracts; files match
    let tmp = tempdir().unwrap();
    let src_dir = tmp.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    let file_content = b"hello from tar gz test\nline 2\n";
    write_fixture(&src_dir, "file.txt", file_content);

    let archive = tmp.path().join("archive.tar.gz");

    // Create .tar.gz
    tar_cmd()
        .arg("-c")
        .arg("-z")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(tmp.path().to_str().unwrap())
        .arg("src")
        .assert()
        .success();

    assert!(archive.exists(), ".tar.gz archive was not created");

    // Extract .tar.gz into a different directory
    let extract_dir = tmp.path().join("extract");
    fs::create_dir(&extract_dir).unwrap();

    tar_cmd()
        .arg("-x")
        .arg("-z")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(extract_dir.to_str().unwrap())
        .assert()
        .success();

    // Verify extracted content matches original
    let extracted_file = extract_dir.join("src").join("file.txt");
    assert!(extracted_file.exists(), "extracted file.txt not found");
    let recovered = fs::read(&extracted_file).unwrap();
    assert_eq!(recovered, file_content, "extracted content mismatch");
}

// ─── Test 2: create and extract .tar.bz2 round-trip ─────────────────────────

#[test]
fn create_and_extract_tar_bz2() {
    // Covers: tar -c -j -f creates .tar.bz2; tar -x -j -f extracts; files match
    let tmp = tempdir().unwrap();
    let src_dir = tmp.path().join("bzsrc");
    fs::create_dir(&src_dir).unwrap();

    let file_content = b"bzip2 compressed tar content\n";
    write_fixture(&src_dir, "bz_file.txt", file_content);

    let archive = tmp.path().join("archive.tar.bz2");

    // Create .tar.bz2
    tar_cmd()
        .arg("-c")
        .arg("-j")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(tmp.path().to_str().unwrap())
        .arg("bzsrc")
        .assert()
        .success();

    assert!(archive.exists(), ".tar.bz2 archive was not created");

    // Extract .tar.bz2
    let extract_dir = tmp.path().join("bz_extract");
    fs::create_dir(&extract_dir).unwrap();

    tar_cmd()
        .arg("-x")
        .arg("-j")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(extract_dir.to_str().unwrap())
        .assert()
        .success();

    // Verify content
    let extracted_file = extract_dir.join("bzsrc").join("bz_file.txt");
    assert!(extracted_file.exists(), "extracted bz_file.txt not found");
    let recovered = fs::read(&extracted_file).unwrap();
    assert_eq!(recovered, file_content, "bzip2 extracted content mismatch");
}

// ─── Test 3: create and extract plain .tar round-trip ────────────────────────

#[test]
fn create_and_extract_plain_tar() {
    // Covers: tar -c -f creates uncompressed .tar; tar -x -f extracts
    let tmp = tempdir().unwrap();
    let src_dir = tmp.path().join("plain_src");
    fs::create_dir(&src_dir).unwrap();

    let file_content = b"plain tar content without compression\n";
    write_fixture(&src_dir, "plain.txt", file_content);

    let archive = tmp.path().join("archive.tar");

    // Create plain .tar
    tar_cmd()
        .arg("-c")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(tmp.path().to_str().unwrap())
        .arg("plain_src")
        .assert()
        .success();

    assert!(archive.exists(), ".tar archive was not created");

    // Extract plain .tar
    let extract_dir = tmp.path().join("plain_extract");
    fs::create_dir(&extract_dir).unwrap();

    tar_cmd()
        .arg("-x")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(extract_dir.to_str().unwrap())
        .assert()
        .success();

    // Verify content
    let extracted_file = extract_dir.join("plain_src").join("plain.txt");
    assert!(extracted_file.exists(), "extracted plain.txt not found");
    let recovered = fs::read(&extracted_file).unwrap();
    assert_eq!(recovered, file_content, "plain tar extracted content mismatch");
}

// ─── Test 4: list archive contents (.tar.gz) ─────────────────────────────────

#[test]
fn list_tar_gz() {
    // Covers: tar -t -z -f lists archive file names to stdout
    let tmp = tempdir().unwrap();
    let src_dir = tmp.path().join("list_src");
    fs::create_dir(&src_dir).unwrap();

    write_fixture(&src_dir, "alpha.txt", b"alpha");
    write_fixture(&src_dir, "beta.txt", b"beta");

    let archive = tmp.path().join("list.tar.gz");

    // Create archive
    tar_cmd()
        .arg("-c")
        .arg("-z")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(tmp.path().to_str().unwrap())
        .arg("list_src")
        .assert()
        .success();

    // List archive contents — should contain both filenames
    tar_cmd()
        .arg("-t")
        .arg("-z")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha.txt"))
        .stdout(predicate::str::contains("beta.txt"));
}

// ─── Test 5: round-trip byte-exact content preservation ──────────────────────

#[test]
fn roundtrip_preserves_content() {
    // Covers: byte-exact content check after .tar.gz create → extract
    let tmp = tempdir().unwrap();
    let src_dir = tmp.path().join("rtsrc");
    fs::create_dir(&src_dir).unwrap();

    // Use binary content (not just ASCII) to catch encoding issues
    let binary_content: Vec<u8> = (0u8..=255u8).collect();
    write_fixture(&src_dir, "binary.bin", &binary_content);

    let text_content = b"line1\nline2\nline3 with unicode: \xc3\xa9\n";
    write_fixture(&src_dir, "text.txt", text_content);

    let archive = tmp.path().join("rt.tar.gz");

    tar_cmd()
        .arg("-c")
        .arg("-z")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(tmp.path().to_str().unwrap())
        .arg("rtsrc")
        .assert()
        .success();

    let extract_dir = tmp.path().join("rtextract");
    fs::create_dir(&extract_dir).unwrap();

    tar_cmd()
        .arg("-x")
        .arg("-z")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(extract_dir.to_str().unwrap())
        .assert()
        .success();

    // Byte-exact binary check
    let recovered_bin = fs::read(extract_dir.join("rtsrc").join("binary.bin")).unwrap();
    assert_eq!(recovered_bin, binary_content, "binary content mismatch after round-trip");

    // Byte-exact text check
    let recovered_text = fs::read(extract_dir.join("rtsrc").join("text.txt")).unwrap();
    assert_eq!(recovered_text, text_content, "text content mismatch after round-trip");
}

// ─── Test 6: archive a directory tree with nested structure ───────────────────

#[test]
fn create_directory_archive() {
    // Covers: archive a directory tree; extract and verify nested structure
    let tmp = tempdir().unwrap();
    let src_dir = tmp.path().join("tree");
    let sub_dir = src_dir.join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();

    write_fixture(&src_dir, "root.txt", b"root file");
    write_fixture(&sub_dir, "nested.txt", b"nested file");

    let archive = tmp.path().join("tree.tar.gz");

    tar_cmd()
        .arg("-c")
        .arg("-z")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(tmp.path().to_str().unwrap())
        .arg("tree")
        .assert()
        .success();

    assert!(archive.exists(), "directory archive was not created");

    let extract_dir = tmp.path().join("tree_extract");
    fs::create_dir(&extract_dir).unwrap();

    tar_cmd()
        .arg("-x")
        .arg("-z")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(extract_dir.to_str().unwrap())
        .assert()
        .success();

    // Verify directory structure was preserved
    let root_file = extract_dir.join("tree").join("root.txt");
    let nested_file = extract_dir.join("tree").join("subdir").join("nested.txt");

    assert!(root_file.exists(), "root.txt not found after extraction");
    assert!(nested_file.exists(), "subdir/nested.txt not found after extraction");

    assert_eq!(fs::read(&root_file).unwrap(), b"root file");
    assert_eq!(fs::read(&nested_file).unwrap(), b"nested file");
}

// ─── Test 7: list plain tar contents ─────────────────────────────────────────

#[test]
fn list_plain_tar() {
    // Covers: tar -t -f lists uncompressed archive entries
    let tmp = tempdir().unwrap();
    let src_dir = tmp.path().join("lp_src");
    fs::create_dir(&src_dir).unwrap();

    write_fixture(&src_dir, "item.txt", b"item content");

    let archive = tmp.path().join("list_plain.tar");

    tar_cmd()
        .arg("-c")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .arg("-C")
        .arg(tmp.path().to_str().unwrap())
        .arg("lp_src")
        .assert()
        .success();

    tar_cmd()
        .arg("-t")
        .arg("-f")
        .arg(archive.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("item.txt"));
}

// ─── Test 8: error on no mode flag ───────────────────────────────────────────

#[test]
fn error_when_no_mode_specified() {
    // Covers: error handling when no -c/-x/-t is given
    tar_cmd()
        .arg("-f")
        .arg("some.tar")
        .assert()
        .failure();
}
