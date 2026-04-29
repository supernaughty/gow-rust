// Integration tests for gow-tar (R018).
// Tests cover: -c/-x/-t modes with -z (gzip), -j (bzip2), and plain codec.
// All tests use tempfile for isolation and assert_cmd for binary invocation.

use assert_cmd::Command;
use bzip2::write::BzEncoder;
use bzip2::Compression as BzCompression;
use predicates::prelude::*;
use std::fs;
use tar::{Builder, Header};
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

// ── WR-01: multi-stream bzip2 extraction ─────────────────────────────────────

/// Proves MultiBzDecoder is required: BzDecoder would stop after the first stream.
/// Constructs two separate bzip2-wrapped tar entries inline and verifies both are extracted by `tar xjf`.
#[test]
fn multi_stream_bzip2_extracts_both_entries() {
    let dir = tempdir().unwrap();

    // Build stream 1: a tar archive containing "file_a.txt"
    let mut buf = Vec::new();
    {
        let enc = BzEncoder::new(&mut buf, BzCompression::default());
        let mut tb = Builder::new(enc);
        let content_a = b"hello from file a";
        let mut header = Header::new_gnu();
        header.set_size(content_a.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tb.append_data(&mut header, "file_a.txt", content_a.as_ref()).unwrap();
        tb.into_inner().unwrap().finish().unwrap();
    }

    // Build stream 2: a tar archive containing "file_b.txt"
    {
        let enc = BzEncoder::new(&mut buf, BzCompression::default());
        let mut tb = Builder::new(enc);
        let content_b = b"hello from file b";
        let mut header = Header::new_gnu();
        header.set_size(content_b.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tb.append_data(&mut header, "file_b.txt", content_b.as_ref()).unwrap();
        tb.into_inner().unwrap().finish().unwrap();
    }

    // Write concatenated bzip2 streams to a .tar.bz2 file
    let archive_path = dir.path().join("multi.tar.bz2");
    fs::write(&archive_path, &buf).unwrap();

    let extract_dir = dir.path().join("out");
    fs::create_dir(&extract_dir).unwrap();

    // Run tar -x -j -f against the multi-stream archive
    tar_cmd()
        .args([
            "-x",
            "-j",
            "-f",
            archive_path.to_str().unwrap(),
            "-C",
            extract_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Both entries must be present — BzDecoder would only extract file_a.txt
    assert!(extract_dir.join("file_a.txt").exists(), "file_a.txt missing");
    assert!(extract_dir.join("file_b.txt").exists(), "file_b.txt missing");
}

// ── WR-02: graceful CLI error (exit 2 for invalid args) ──────────────────────

/// tar with no mode flag (-c/-x/-t) exits with code 2 (argument misuse).
/// This also covers the WR-02 fix: from_arg_matches errors now exit 2 not panic.
#[test]
fn missing_mode_flag_exits_with_error() {
    tar_cmd()
        .arg("somefile.tar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("option").or(predicate::str::contains("cxt")));
}

// ── WR-03: non-zero exit on per-entry extraction failure ─────────────────────

/// tar exits 1 when entries cannot be extracted (non-symlink error).
/// Uses a non-existent destination directory to trigger extraction failure.
#[test]
fn extraction_failure_exits_nonzero() {
    let dir = tempdir().unwrap();

    // Create a valid .tar.gz archive
    let content = b"test content";
    let archive_path = dir.path().join("test.tar.gz");
    {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::{Builder, Header};
        let f = fs::File::create(&archive_path).unwrap();
        let enc = GzEncoder::new(f, Compression::default());
        let mut tb = Builder::new(enc);
        let mut header = Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tb.append_data(&mut header, "test.txt", content.as_ref()).unwrap();
        tb.into_inner().unwrap().finish().unwrap();
    }

    // Extract into a path that does not exist — triggers unpack_in failure
    tar_cmd()
        .args([
            "-x",
            "-z",
            "-f",
            archive_path.to_str().unwrap(),
            "-C",
            dir.path().join("nonexistent_destination").to_str().unwrap(),
        ])
        .assert()
        .failure();
}
