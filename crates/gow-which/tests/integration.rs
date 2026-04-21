//! Integration tests for `which` (WHICH-01, GOW #276).
//! All tests use isolated PATH via tempdir + GOW_PATHEXT override (D-18d).

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

fn which_cmd() -> Command {
    Command::cargo_bin("which")
        .expect("which binary not found — run `cargo build -p gow-which` first")
}

fn make_executable_file(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(b"fake").unwrap();
    path
}

/// Case-insensitive contains — Windows filesystems are case-insensitive so
/// `.EXE` (PATHEXT-cased) and `.exe` (disk-cased) refer to the same file.
/// Our resolver echoes the PATHEXT casing we constructed the candidate with
/// (D-18 literal-first then expand — we do NOT canonicalize per D-18e), so
/// assertions on filename substrings must fold case.
fn stdout_ci_contains(needle: &'static str) -> impl predicates::Predicate<str> {
    predicate::function(move |out: &str| out.to_lowercase().contains(&needle.to_lowercase()))
}

#[test]
fn test_literal_match_found() {
    let tmp = tempfile::tempdir().unwrap();
    let _target = make_executable_file(tmp.path(), "foo");

    which_cmd()
        .env("PATH", tmp.path())
        .env("GOW_PATHEXT", ".EXE")
        .arg("foo")
        .assert()
        .success()
        .stdout(predicate::str::contains("foo"));
}

#[test]
fn test_literal_beats_pathext_expansion() {
    // Both `foo` and `foo.exe` exist — literal must win (D-18).
    let tmp = tempfile::tempdir().unwrap();
    let _foo = make_executable_file(tmp.path(), "foo");
    let _foo_exe = make_executable_file(tmp.path(), "foo.exe");

    which_cmd()
        .env("PATH", tmp.path())
        .env("GOW_PATHEXT", ".EXE")
        .arg("foo")
        .assert()
        .success()
        .stdout(predicate::function(|out: &str| {
            // Output ends with "foo" (not "foo.exe"). Since \n follows, check for "\\foo\n" or "/foo\n".
            out.trim_end().ends_with("foo")
        }));
}

#[test]
fn test_pathext_expansion_fallback() {
    // Only `foo.exe` exists; `which foo` must expand via PATHEXT (D-18).
    let tmp = tempfile::tempdir().unwrap();
    let _foo_exe = make_executable_file(tmp.path(), "foo.exe");

    which_cmd()
        .env("PATH", tmp.path())
        .env("GOW_PATHEXT", ".EXE;.BAT")
        .arg("foo")
        .assert()
        .success()
        .stdout(stdout_ci_contains("foo.exe"));
}

#[test]
fn test_not_found_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    which_cmd()
        .env("PATH", tmp.path())
        .env("GOW_PATHEXT", ".EXE")
        .arg("definitely_not_there_xyzzy_9999")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("no definitely_not_there_xyzzy_9999"));
}

#[test]
fn test_a_returns_all_matches_across_dirs() {
    let tmp_a = tempfile::tempdir().unwrap();
    let tmp_b = tempfile::tempdir().unwrap();
    let _a = make_executable_file(tmp_a.path(), "sharedname.exe");
    let _b = make_executable_file(tmp_b.path(), "sharedname.exe");

    // PATH = tmp_a;tmp_b  →  -a should return both hits.
    let combined = format!("{};{}", tmp_a.path().display(), tmp_b.path().display());
    which_cmd()
        .env("PATH", combined)
        .env("GOW_PATHEXT", ".EXE")
        .args(["-a", "sharedname"])
        .assert()
        .success()
        .stdout(predicate::function(|out: &str| {
            // Expect two lines.
            out.lines().filter(|l| l.contains("sharedname")).count() == 2
        }));
}

#[test]
fn test_a_includes_literal_and_pathext() {
    // -a should return literal AND PATHEXT-expanded matches in the same dir.
    let tmp = tempfile::tempdir().unwrap();
    let _literal = make_executable_file(tmp.path(), "bothname");
    let _expanded = make_executable_file(tmp.path(), "bothname.exe");

    which_cmd()
        .env("PATH", tmp.path())
        .env("GOW_PATHEXT", ".EXE")
        .args(["-a", "bothname"])
        .assert()
        .success()
        .stdout(predicate::function(|out: &str| {
            let lines: Vec<&str> = out.lines().collect();
            lines.len() >= 2 // at least literal + .exe
        }));
}

#[test]
fn test_default_pathext_when_gow_pathext_unset() {
    // Without GOW_PATHEXT, which should use system PATHEXT; if PATHEXT is also missing,
    // fall back to .COM;.EXE;.BAT;.CMD.
    let tmp = tempfile::tempdir().unwrap();
    let _exe = make_executable_file(tmp.path(), "default.cmd");

    which_cmd()
        .env("PATH", tmp.path())
        .env_remove("GOW_PATHEXT")
        .env_remove("PATHEXT")
        .arg("default")
        .assert()
        .success()
        .stdout(stdout_ci_contains("default.cmd"));
}

#[test]
fn test_cwd_not_auto_searched() {
    // Create an executable in a directory that is NOT on PATH. Even if it's the cwd,
    // which should NOT find it (Unix convention per D-18c).
    let tmp = tempfile::tempdir().unwrap();
    let _exe = make_executable_file(tmp.path(), "inCwd.exe");

    which_cmd()
        .current_dir(tmp.path())
        .env("PATH", "/no/such/dir") // tempdir NOT on PATH
        .env("GOW_PATHEXT", ".EXE")
        .arg("inCwd")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_multiple_names_in_one_invocation() {
    let tmp = tempfile::tempdir().unwrap();
    let _alpha = make_executable_file(tmp.path(), "alpha.exe");
    let _beta = make_executable_file(tmp.path(), "beta.exe");

    which_cmd()
        .env("PATH", tmp.path())
        .env("GOW_PATHEXT", ".EXE")
        .args(["alpha", "beta"])
        .assert()
        .success()
        .stdout(stdout_ci_contains("alpha.exe"))
        .stdout(stdout_ci_contains("beta.exe"));
}

#[test]
fn test_one_not_found_fails_overall() {
    // When multi-name invocation has one miss, overall exit = 1 (but found names still print).
    let tmp = tempfile::tempdir().unwrap();
    let _exe = make_executable_file(tmp.path(), "present.exe");

    which_cmd()
        .env("PATH", tmp.path())
        .env("GOW_PATHEXT", ".EXE")
        .args(["present", "missing_xyzzy"])
        .assert()
        .failure()
        .code(1)
        .stdout(stdout_ci_contains("present.exe"))
        .stderr(predicate::str::contains("no missing_xyzzy"));
}

#[test]
fn test_no_args_error() {
    which_cmd()
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("missing argument"));
}

#[test]
fn test_bad_flag_exits_1() {
    which_cmd()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_utf8_name_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    // Make `안녕.exe` executable placeholder.
    let _exe = make_executable_file(tmp.path(), "안녕.exe");

    which_cmd()
        .env("PATH", tmp.path())
        .env("GOW_PATHEXT", ".EXE")
        .arg("안녕")
        .assert()
        .success()
        .stdout(stdout_ci_contains("안녕.exe"));
}
