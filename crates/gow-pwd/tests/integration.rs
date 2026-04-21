//! Integration tests for `pwd` (UTIL-02).
//! Covers Dimensions 1 (GNU compat), 3 (Windows UNC strip), 4 (error path).

use assert_cmd::Command;
use predicates::prelude::*;

fn pwd() -> Command {
    Command::cargo_bin("pwd")
        .expect("pwd binary not found — run `cargo build -p gow-pwd` first")
}

#[test]
fn test_default_prints_cwd() {
    // Run in a known directory — assert_cmd defaults to repo root.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().to_path_buf();
    pwd()
        .current_dir(&path)
        .env_remove("PWD") // ensure we don't override with stale PWD
        .assert()
        .success()
        .stdout(predicate::function(move |out: &str| {
            // The stdout should contain the tempdir path. Default -L mode returns
            // current_dir() which is typically the un-canonicalized form (no \\?\).
            // We accept multiple possible shapes because tempdir on Windows may be
            // a short-name form; the robust check is "contains the final component".
            let line = out.trim_end();
            let basename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            !basename.is_empty() && line.contains(basename)
        }));
}

#[test]
fn test_p_flag_strips_unc_prefix() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().to_path_buf();
    pwd()
        .current_dir(&path)
        .arg("-P")
        .assert()
        .success()
        // Output must NOT start with `\\?\` per D-24 UNC strip.
        .stdout(predicate::str::starts_with(r"\\?\").not());
}

#[test]
fn test_p_flag_returns_drive_letter_form() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().to_path_buf();
    pwd()
        .current_dir(&path)
        .arg("-P")
        .assert()
        .success()
        // Output should start with a drive letter (e.g., C:\) — on Windows this is
        // guaranteed because canonicalize returns an absolute path and
        // simplify_canonical strips the \\?\ prefix from drive-letter paths.
        .stdout(predicate::str::is_match(r"(?i)^[A-Z]:\\").unwrap());
}

#[test]
fn test_l_flag_same_as_default() {
    let tmp = tempfile::tempdir().unwrap();
    let default_out = pwd()
        .current_dir(tmp.path())
        .env_remove("PWD")
        .output()
        .unwrap();
    let l_out = pwd()
        .current_dir(tmp.path())
        .env_remove("PWD")
        .arg("-L")
        .output()
        .unwrap();
    assert_eq!(
        default_out.stdout, l_out.stdout,
        "-L and default must produce identical stdout"
    );
}

#[test]
fn test_bad_flag_exits_1_not_2() {
    pwd()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_gnu_error_format_on_bad_flag() {
    pwd()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("pwd:"));
}

#[test]
fn test_output_has_trailing_newline() {
    let tmp = tempfile::tempdir().unwrap();
    pwd()
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::ends_with("\n"));
}

#[test]
fn test_pwd_env_used_when_valid() {
    // When PWD == current_dir canonical path, the default mode should prefer PWD verbatim.
    // Harder to test precisely because canonicalize smooths symlinks; accept that if PWD is
    // set to the current tempdir, the output contains that tempdir's basename.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path();
    pwd()
        .current_dir(path)
        .env("PWD", path.to_string_lossy().to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            path.file_name().unwrap().to_string_lossy().to_string(),
        ));
}
