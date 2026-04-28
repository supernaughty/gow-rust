use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_name_matches_basename_only() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("foo.txt"), "").unwrap();
    fs::write(tmp.path().join("bar.rs"), "").unwrap();
    let sub = tmp.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("nested.txt"), "").unwrap();

    Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-name")
        .arg("*.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("foo.txt"))
        .stdout(predicate::str::contains("nested.txt"))
        .stdout(predicate::str::contains("bar.rs").not());
}

#[test]
fn test_name_is_case_sensitive() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("foo.TXT"), "").unwrap();
    fs::write(tmp.path().join("bar.txt"), "").unwrap();

    Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-name")
        .arg("*.txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("bar.txt"))
        .stdout(predicate::str::contains("foo.TXT").not());
}

#[test]
fn test_iname_is_case_insensitive() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("foo.TXT"), "").unwrap();
    fs::write(tmp.path().join("bar.txt"), "").unwrap();

    Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-iname")
        .arg("*.TXT")
        .assert()
        .success()
        .stdout(predicate::str::contains("bar.txt"))
        .stdout(predicate::str::contains("foo.TXT"));
}

#[test]
fn test_type_filter_files() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("file.txt"), "").unwrap();
    let subdir = tmp.path().join("subdir");
    fs::create_dir(&subdir).unwrap();

    // With -type f: subdir directory should NOT appear, file.txt should appear
    let output = Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-type")
        .arg("f")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file.txt"), "expected file.txt in output, got: {stdout}");
    // The directory itself should not appear in -type f output
    // (subdir is a directory, not a file)
    let lines: Vec<&str> = stdout.lines().collect();
    for line in &lines {
        // No line should end with the subdir path component (it's a dir, not a file)
        assert!(
            !line.ends_with("subdir"),
            "expected no directory entries but found: {line}"
        );
    }
}

#[test]
fn test_type_filter_directories() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("file.txt"), "").unwrap();
    let subdir = tmp.path().join("mysubdir");
    fs::create_dir(&subdir).unwrap();

    Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-type")
        .arg("d")
        .assert()
        .success()
        .stdout(predicate::str::contains("mysubdir"))
        .stdout(predicate::str::contains("file.txt").not());
}

#[test]
fn test_size_greater_than() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("small.bin"), vec![0u8; 100]).unwrap();
    fs::write(tmp.path().join("large.bin"), vec![0u8; 5000]).unwrap();

    Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-size")
        .arg("+1k") // > 1024 bytes
        .assert()
        .success()
        .stdout(predicate::str::contains("large.bin"))
        .stdout(predicate::str::contains("small.bin").not());
}

#[test]
fn test_maxdepth_zero_lists_root_only() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("a.txt"), "").unwrap();
    let sub = tmp.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("b.txt"), "").unwrap();

    // -maxdepth 0 should only list the root directory itself — no children
    Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-maxdepth")
        .arg("0")
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt").not())
        .stdout(predicate::str::contains("b.txt").not());
}

#[test]
fn test_maxdepth_one_skips_subdir_contents() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("a.txt"), "").unwrap();
    let sub = tmp.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("b.txt"), "").unwrap();

    // -maxdepth 1 lists root + direct children but NOT deeper contents
    Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-maxdepth")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt"))
        .stdout(predicate::str::contains("b.txt").not());
}

#[test]
fn test_mtime_recent_files() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("recent.txt"), "x").unwrap();

    // -mtime -1 means modified less than 1 day ago — the file we just wrote should match
    Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-mtime")
        .arg("-1")
        .arg("-type")
        .arg("f")
        .assert()
        .success()
        .stdout(predicate::str::contains("recent.txt"));
}

#[test]
fn test_exec_runs_command_per_match() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("a.txt"), "").unwrap();
    fs::write(tmp.path().join("b.txt"), "").unwrap();

    // Use cmd /C echo on Windows so we don't depend on echo.exe in PATH.
    let bin = if cfg!(windows) { "cmd" } else { "echo" };
    let mut c = Command::cargo_bin("find").unwrap();
    c.arg(tmp.path())
        .arg("-name")
        .arg("*.txt")
        .arg("-type")
        .arg("f")
        .arg("-exec");
    if cfg!(windows) {
        c.arg(bin)
            .arg("/C")
            .arg("echo")
            .arg("FOUND:")
            .arg("{}")
            .arg(";");
    } else {
        c.arg(bin).arg("FOUND:").arg("{}").arg(";");
    }
    c.assert()
        .success()
        .stdout(predicate::str::contains("FOUND:"))
        .stdout(predicate::str::contains("a.txt"))
        .stdout(predicate::str::contains("b.txt"));
}

#[test]
fn test_exec_handles_paths_with_spaces() {
    // Regression: GOW issue #209 — paths with spaces must be passed as a single argument
    // (not requoted by a shell). std::process::Command achieves this directly via CreateProcessW.
    let tmp = tempdir().unwrap();
    let dir_with_space = tmp.path().join("has space");
    fs::create_dir(&dir_with_space).unwrap();
    let file_with_space = dir_with_space.join("file with space.txt");
    fs::write(&file_with_space, "").unwrap();

    let bin = if cfg!(windows) { "cmd" } else { "echo" };
    let mut c = Command::cargo_bin("find").unwrap();
    c.arg(tmp.path()).arg("-name").arg("*.txt").arg("-exec");
    if cfg!(windows) {
        c.arg(bin)
            .arg("/C")
            .arg("echo")
            .arg("MATCH:")
            .arg("{}")
            .arg(";");
    } else {
        c.arg(bin).arg("MATCH:").arg("{}").arg(";");
    }
    c.assert()
        .success()
        .stdout(predicate::str::contains("file with space.txt"));
}

#[test]
fn test_print0_emits_null_separated_paths() {
    let tmp = tempdir().unwrap();
    fs::write(tmp.path().join("alpha.txt"), "").unwrap();
    fs::write(tmp.path().join("beta.txt"), "").unwrap();

    let out = Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-name")
        .arg("*.txt")
        .arg("-type")
        .arg("f")
        .arg("-print0")
        .output()
        .unwrap();
    assert!(out.status.success(), "find -print0 should exit 0");
    // At least one NUL byte present (output is NUL-separated)
    assert!(
        out.stdout.contains(&0u8),
        "stdout must contain NUL separators"
    );
    // No trailing newlines expected — paths are NUL-separated only
    let nul_count = out.stdout.iter().filter(|&&b| b == 0).count();
    assert!(
        nul_count >= 2,
        "expected at least 2 NUL separators for 2 files, got {}",
        nul_count
    );
}

#[test]
fn test_multiple_predicates_and_together() {
    // All predicates AND: only files matching -name, -type f, and -size > 100 bytes should match
    let tmp = tempdir().unwrap();
    // This file matches -name "*.txt" and -type f but NOT -size +100c
    fs::write(tmp.path().join("small.txt"), "x").unwrap();
    // This file matches all three predicates
    fs::write(tmp.path().join("large.txt"), vec![b'y'; 200]).unwrap();
    // This file matches -size +100c and -type f but NOT -name "*.txt"
    fs::write(tmp.path().join("large.rs"), vec![b'z'; 200]).unwrap();

    Command::cargo_bin("find")
        .unwrap()
        .arg(tmp.path())
        .arg("-name")
        .arg("*.txt")
        .arg("-type")
        .arg("f")
        .arg("-size")
        .arg("+100c")
        .assert()
        .success()
        .stdout(predicate::str::contains("large.txt"))
        .stdout(predicate::str::contains("small.txt").not())
        .stdout(predicate::str::contains("large.rs").not());
}
