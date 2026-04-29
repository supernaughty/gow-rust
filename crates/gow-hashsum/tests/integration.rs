use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

// ─── MD5 GNU known-answer vectors ──────────────────────────────────────────

#[test]
fn md5_empty_vector() {
    Command::cargo_bin("md5sum")
        .unwrap()
        .write_stdin("")
        .assert()
        .success()
        .stdout("d41d8cd98f00b204e9800998ecf8427e  -\n");
}

#[test]
fn md5_abc_vector() {
    Command::cargo_bin("md5sum")
        .unwrap()
        .write_stdin("abc")
        .assert()
        .success()
        .stdout("900150983cd24fb0d6963f7d28e17f72  -\n");
}

// ─── SHA-1 GNU known-answer vectors ────────────────────────────────────────

#[test]
fn sha1_empty_vector() {
    Command::cargo_bin("sha1sum")
        .unwrap()
        .write_stdin("")
        .assert()
        .success()
        .stdout("da39a3ee5e6b4b0d3255bfef95601890afd80709  -\n");
}

#[test]
fn sha1_abc_vector() {
    Command::cargo_bin("sha1sum")
        .unwrap()
        .write_stdin("abc")
        .assert()
        .success()
        .stdout("a9993e364706816aba3e25717850c26c9cd0d89d  -\n");
}

// ─── SHA-256 GNU known-answer vectors ──────────────────────────────────────

#[test]
fn sha256_empty_vector() {
    Command::cargo_bin("sha256sum")
        .unwrap()
        .write_stdin("")
        .assert()
        .success()
        .stdout("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  -\n");
}

#[test]
fn sha256_abc_vector() {
    Command::cargo_bin("sha256sum")
        .unwrap()
        .write_stdin("abc")
        .assert()
        .success()
        .stdout("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  -\n");
}

// ─── File input ────────────────────────────────────────────────────────────

#[test]
fn md5_file_input() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("data.txt");
    fs::write(&p, b"abc").unwrap();
    let out = Command::cargo_bin("md5sum")
        .unwrap()
        .arg(&p)
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.starts_with("900150983cd24fb0d6963f7d28e17f72"),
        "got: {stdout}"
    );
    assert!(stdout.contains("data.txt"), "got: {stdout}");
}

#[test]
fn md5_missing_file_errors() {
    Command::cargo_bin("md5sum")
        .unwrap()
        .arg("nonexistent_xyz_99999.bin")
        .assert()
        .failure()
        .code(1);
}

// ─── Check mode ─────────────────────────────────────────────────────────────

#[test]
fn md5_check_mode_pass() {
    let dir = TempDir::new().unwrap();
    let data = dir.path().join("data.txt");
    fs::write(&data, b"abc").unwrap();
    let check = dir.path().join("checks.md5");
    fs::write(&check, "900150983cd24fb0d6963f7d28e17f72  data.txt\n").unwrap();
    Command::cargo_bin("md5sum")
        .unwrap()
        .args(["-c"])
        .arg(&check)
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn md5_check_mode_fail() {
    let dir = TempDir::new().unwrap();
    let data = dir.path().join("data.txt");
    fs::write(&data, b"abc").unwrap();
    let check = dir.path().join("checks.md5");
    fs::write(&check, "00000000000000000000000000000000  data.txt\n").unwrap();
    Command::cargo_bin("md5sum")
        .unwrap()
        .args(["-c"])
        .arg(&check)
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1)
        .stdout(contains("FAILED"));
}

#[test]
fn md5_check_mode_binary_format() {
    let dir = TempDir::new().unwrap();
    let data = dir.path().join("data.txt");
    fs::write(&data, b"abc").unwrap();
    let check = dir.path().join("checks.md5");
    // Binary mode format: single space + asterisk before filename
    fs::write(&check, "900150983cd24fb0d6963f7d28e17f72 *data.txt\n").unwrap();
    Command::cargo_bin("md5sum")
        .unwrap()
        .args(["-c"])
        .arg(&check)
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn md5_check_mode_missing_file() {
    let dir = TempDir::new().unwrap();
    let check = dir.path().join("checks.md5");
    fs::write(&check, "00000000000000000000000000000000  missing.txt\n").unwrap();
    Command::cargo_bin("md5sum")
        .unwrap()
        .args(["-c"])
        .arg(&check)
        .current_dir(dir.path())
        .assert()
        .failure()
        .code(1);
}

// ─── Dispatch: distinct digest lengths ────────────────────────────────────

#[test]
fn dispatch_distinct_digest_lengths() {
    let md5_out = Command::cargo_bin("md5sum")
        .unwrap()
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let sha1_out = Command::cargo_bin("sha1sum")
        .unwrap()
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let sha256_out = Command::cargo_bin("sha256sum")
        .unwrap()
        .write_stdin("")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let md5_hex = String::from_utf8(md5_out).unwrap();
    let sha1_hex = String::from_utf8(sha1_out).unwrap();
    let sha256_hex = String::from_utf8(sha256_out).unwrap();

    assert_eq!(
        md5_hex.split_whitespace().next().unwrap().len(),
        32,
        "MD5 hex should be 32 chars"
    );
    assert_eq!(
        sha1_hex.split_whitespace().next().unwrap().len(),
        40,
        "SHA-1 hex should be 40 chars"
    );
    assert_eq!(
        sha256_hex.split_whitespace().next().unwrap().len(),
        64,
        "SHA-256 hex should be 64 chars"
    );
}
