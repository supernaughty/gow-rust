use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_unix2dos_inplace() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "line1\nline2\n").unwrap();

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg(file_path.to_str().unwrap());
    cmd.assert().success();

    let content = fs::read(&file_path).unwrap();
    assert_eq!(content, b"line1\r\nline2\r\n");
}

#[test]
fn test_unix2dos_newfile() {
    let dir = tempdir().unwrap();
    let src_path = dir.path().join("src.txt");
    let dst_path = dir.path().join("dst.txt");
    fs::write(&src_path, "line1\nline2\n").unwrap();

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg("-n").arg(src_path.to_str().unwrap()).arg(dst_path.to_str().unwrap());
    cmd.assert().success();

    let src_content = fs::read(&src_path).unwrap();
    assert_eq!(src_content, b"line1\nline2\n"); // src unchanged

    let dst_content = fs::read(&dst_path).unwrap();
    assert_eq!(dst_content, b"line1\r\nline2\r\n");
}

#[test]
fn test_unix2dos_keepdate() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "line1\nline2\n").unwrap();

    let metadata = fs::metadata(&file_path).unwrap();
    let original_mtime = filetime::FileTime::from_last_modification_time(&metadata);

    // Sleep a bit to ensure mtime would change if not preserved
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg("-k").arg(file_path.to_str().unwrap());
    cmd.assert().success();

    let new_metadata = fs::metadata(&file_path).unwrap();
    let new_mtime = filetime::FileTime::from_last_modification_time(&new_metadata);

    assert_eq!(original_mtime, new_mtime);
}

#[test]
fn test_unix2dos_skips_binary() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.bin");
    fs::write(&file_path, b"line1\n\0line2\n").unwrap();

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg(file_path.to_str().unwrap());
    cmd.assert().success();

    let content = fs::read(&file_path).unwrap();
    assert_eq!(content, b"line1\n\0line2\n"); // unchanged
}

#[test]
fn test_unix2dos_force_binary() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.bin");
    fs::write(&file_path, b"line1\n\0line2\n").unwrap();

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg("-f").arg(file_path.to_str().unwrap());
    cmd.assert().success();

    let content = fs::read(&file_path).unwrap();
    assert_eq!(content, b"line1\r\n\0line2\r\n");
}

#[test]
fn test_unix2dos_empty() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("empty.txt");
    fs::write(&file_path, "").unwrap();

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg(file_path.to_str().unwrap());
    cmd.assert().success();

    let content = fs::read(&file_path).unwrap();
    assert_eq!(content, b"");
}

#[test]
fn test_unix2dos_no_trailing_newline() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "abc").unwrap();

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg(file_path.to_str().unwrap());
    cmd.assert().success();

    let content = fs::read(&file_path).unwrap();
    assert_eq!(content, b"abc");
}

#[test]
fn test_unix2dos_already_dos() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("dos.txt");
    fs::write(&file_path, "line1\r\n").unwrap();

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg(file_path.to_str().unwrap());
    cmd.assert().success();

    let content = fs::read(&file_path).unwrap();
    assert_eq!(content, b"line1\r\n"); // unchanged
}

#[test]
fn test_unix2dos_multiple_operands() {
    let dir = tempdir().unwrap();
    let f1 = dir.path().join("f1.txt");
    let f2 = dir.path().join("f2.txt");
    fs::write(&f1, "a\n").unwrap();
    fs::write(&f2, "b\n").unwrap();

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg(f1.to_str().unwrap()).arg(f2.to_str().unwrap());
    cmd.assert().success();

    assert_eq!(fs::read(&f1).unwrap(), b"a\r\n");
    assert_eq!(fs::read(&f2).unwrap(), b"b\r\n");
}

#[test]
fn test_unix2dos_quiet() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "line\n").unwrap();

    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg("-q").arg(file_path.to_str().unwrap());
    let output = cmd.assert().success().get_output().stdout.clone();
    assert!(output.is_empty());
}

#[test]
fn test_unix2dos_non_existent() {
    let mut cmd = Command::cargo_bin("unix2dos").unwrap();
    cmd.arg("non_existent_file_999.txt");
    cmd.assert().failure();
}

#[test]
fn test_round_trip() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("trip.txt");
    fs::write(&file_path, "hello\nworld\n").unwrap();

    // unix2dos
    let mut u2d = Command::cargo_bin("unix2dos").unwrap();
    u2d.arg(file_path.to_str().unwrap());
    u2d.assert().success();
    assert_eq!(fs::read(&file_path).unwrap(), b"hello\r\nworld\r\n");

    // dos2unix - need to find it in target/debug
    // On Windows it's .exe
    let mut d2u_path = std::env::current_exe().unwrap();
    d2u_path.pop(); // deps
    d2u_path.pop(); // debug
    let d2u_exe = d2u_path.join("dos2unix.exe");

    let mut d2u = if d2u_exe.exists() {
        Command::new(d2u_exe)
    } else {
        // Fallback for Linux/macOS just in case, though target is Windows
        let d2u_bin = d2u_path.join("dos2unix");
        Command::new(d2u_bin)
    };

    d2u.arg(file_path.to_str().unwrap());
    d2u.assert().success();
    assert_eq!(fs::read(&file_path).unwrap(), b"hello\nworld\n");
}
