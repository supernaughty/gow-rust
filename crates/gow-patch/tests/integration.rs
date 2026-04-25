use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

/// Create a unified diff patch string from old and new content.
/// Uses the diffy crate format (same as GNU diff output).
fn make_patch(original_path: &str, modified_path: &str, original: &str, modified: &str) -> String {
    // Manually construct a unified diff compatible with GNU patch
    // Format: --- a/file \n+++ b/file\n@@ ... @@\n lines
    let patch = diffy::create_patch(original, modified);
    // Replace the generic "original"/"modified" headers with actual paths
    let s = patch.to_string();
    s.replace("--- original\n", &format!("--- a/{original_path}\n"))
        .replace("+++ modified\n", &format!("+++ b/{modified_path}\n"))
}

#[test]
fn test_patch_apply_basic() {
    let tmp = tempdir().unwrap();
    let target = tmp.path().join("file.txt");
    fs::write(&target, "line1\nline2\nline3\n").unwrap();

    // Create patch that changes "line2" to "changed"
    let patch_content = make_patch("file.txt", "file.txt", "line1\nline2\nline3\n", "line1\nchanged\nline3\n");

    // Write patch to a file
    let patch_file = tmp.path().join("changes.patch");
    fs::write(&patch_file, &patch_content).unwrap();

    let mut cmd = Command::cargo_bin("patch").unwrap();
    cmd.arg("-p1")
        .arg("-i")
        .arg(patch_file.to_str().unwrap())
        .current_dir(tmp.path())
        .assert()
        .success()
        .code(0);

    let result = fs::read_to_string(&target).unwrap();
    assert_eq!(result, "line1\nchanged\nline3\n", "file should be patched");
}

#[test]
fn test_patch_strip_p1() {
    let tmp = tempdir().unwrap();
    let target = tmp.path().join("file.txt");
    fs::write(&target, "hello\nworld\n").unwrap();

    // Patch with a/file.txt prefix (standard git diff format)
    let patch_content = "--- a/file.txt\n+++ b/file.txt\n@@ -1,2 +1,2 @@\n hello\n-world\n+rust\n";
    let patch_file = tmp.path().join("changes.patch");
    fs::write(&patch_file, patch_content).unwrap();

    // -p1 should strip "a/" and "b/" prefix, making target "file.txt"
    let mut cmd = Command::cargo_bin("patch").unwrap();
    cmd.arg("-p1")
        .arg("-i")
        .arg(patch_file.to_str().unwrap())
        .current_dir(tmp.path())
        .assert()
        .success();

    let result = fs::read_to_string(&target).unwrap();
    assert_eq!(result, "hello\nrust\n", "file should be patched with p1 stripping");
}

#[test]
fn test_patch_dry_run() {
    let tmp = tempdir().unwrap();
    let target = tmp.path().join("file.txt");
    let original_content = "hello\nworld\n";
    fs::write(&target, original_content).unwrap();

    let patch_content = "--- a/file.txt\n+++ b/file.txt\n@@ -1,2 +1,2 @@\n hello\n-world\n+rust\n";
    let patch_file = tmp.path().join("changes.patch");
    fs::write(&patch_file, patch_content).unwrap();

    let mut cmd = Command::cargo_bin("patch").unwrap();
    cmd.arg("-p1")
        .arg("--dry-run")
        .arg("-i")
        .arg(patch_file.to_str().unwrap())
        .current_dir(tmp.path())
        .assert()
        .success();

    // File should be unchanged after dry-run
    let result = fs::read_to_string(&target).unwrap();
    assert_eq!(
        result, original_content,
        "--dry-run should not modify the file"
    );
}

#[test]
fn test_patch_reverse() {
    let tmp = tempdir().unwrap();
    let target = tmp.path().join("file.txt");

    // The file is in the PATCHED state; we want to revert to original
    fs::write(&target, "hello\nrust\n").unwrap();

    // Patch that would go from "world" -> "rust"; reversing it goes "rust" -> "world"
    let patch_content = "--- a/file.txt\n+++ b/file.txt\n@@ -1,2 +1,2 @@\n hello\n-world\n+rust\n";
    let patch_file = tmp.path().join("changes.patch");
    fs::write(&patch_file, patch_content).unwrap();

    let mut cmd = Command::cargo_bin("patch").unwrap();
    cmd.arg("-p1")
        .arg("-R")
        .arg("-i")
        .arg(patch_file.to_str().unwrap())
        .current_dir(tmp.path())
        .assert()
        .success();

    let result = fs::read_to_string(&target).unwrap();
    assert_eq!(result, "hello\nworld\n", "-R should reverse the patch");
}

#[test]
fn test_patch_input_file() {
    let tmp = tempdir().unwrap();
    let target = tmp.path().join("file.txt");
    fs::write(&target, "line1\nline2\n").unwrap();

    let patch_content = "--- a/file.txt\n+++ b/file.txt\n@@ -1,2 +1,2 @@\n line1\n-line2\n+patched\n";
    let patch_file = tmp.path().join("my.patch");
    fs::write(&patch_file, patch_content).unwrap();

    let mut cmd = Command::cargo_bin("patch").unwrap();
    cmd.arg("-p1")
        .arg("-i")
        .arg(patch_file.to_str().unwrap())
        .current_dir(tmp.path())
        .assert()
        .success();

    let result = fs::read_to_string(&target).unwrap();
    assert_eq!(result, "line1\npatched\n", "-i should read patch from named file");
}
