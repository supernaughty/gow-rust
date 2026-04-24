use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use filetime::{FileTime, set_file_times};

#[test]
fn test_cp_basic_file() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&src, "hello").unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&dst).unwrap(), "hello");
}

#[test]
fn test_cp_to_directory() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src.txt");
    let dst_dir = tmp.path().join("dst_dir");
    fs::create_dir(&dst_dir).unwrap();
    fs::write(&src, "hello").unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg(&src)
        .arg(&dst_dir)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dst_dir.join("src.txt")).unwrap(), "hello");
}

#[test]
fn test_cp_recursive() {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src_dir");
    let dst_dir = tmp.path().join("dst_dir");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("a.txt"), "a").unwrap();
    fs::create_dir(src_dir.join("sub")).unwrap();
    fs::write(src_dir.join("sub/b.txt"), "b").unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg("-r")
        .arg(&src_dir)
        .arg(&dst_dir)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dst_dir.join("a.txt")).unwrap(), "a");
    assert_eq!(fs::read_to_string(dst_dir.join("sub/b.txt")).unwrap(), "b");
}

#[test]
fn test_cp_preserve_timestamps() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&src, "hello").unwrap();

    // Set old timestamps
    let old_time = FileTime::from_unix_time(1000, 0);
    set_file_times(&src, old_time, old_time).unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg("-p")
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    let dst_meta = fs::metadata(&dst).unwrap();
    assert_eq!(FileTime::from_last_modification_time(&dst_meta), old_time);
}

#[test]
fn test_cp_force_readonly() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&src, "new").unwrap();
    fs::write(&dst, "old").unwrap();

    let mut perms = fs::metadata(&dst).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&dst, perms).unwrap();

    // Without -f it might fail (depends on OS/impl, but GNU cp fails)
    // Actually std::fs::copy might fail on Windows if dest is RO.
    
    Command::cargo_bin("cp")
        .unwrap()
        .arg("-f")
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&dst).unwrap(), "new");
}

#[test]
fn test_cp_verbose() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&src, "hello").unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg("-v")
        .arg(&src)
        .arg(&dst)
        .assert()
        .stdout(predicate::str::contains(format!("'{}' -> '{}'", src.display(), dst.display())));
}

#[test]
fn test_cp_error_same_file() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src.txt");
    fs::write(&src, "hello").unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg(&src)
        .arg(&src)
        .assert()
        .failure()
        .stderr(predicate::str::contains("are the same file"));
}

#[test]
fn test_cp_omitting_directory() {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src_dir");
    let dst_dir = tmp.path().join("dst_dir");
    fs::create_dir(&src_dir).unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg(&src_dir)
        .arg(&dst_dir)
        .assert()
        .success() // GNU cp exits 0 but prints warning
        .stderr(predicate::str::contains("omitting directory"));
}

#[cfg(target_os = "windows")]
#[test]
fn test_cp_symlink_no_dereference() {
    use std::os::windows::fs::symlink_file;
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("target.txt");
    let link = tmp.path().join("link.txt");
    let dst = tmp.path().join("dst_link.txt");
    fs::write(&target, "content").unwrap();
    
    if symlink_file(&target, &link).is_ok() {
        Command::cargo_bin("cp")
            .unwrap()
            .arg("-P")
            .arg(&link)
            .arg(&dst)
            .assert()
            .success();

        assert!(fs::symlink_metadata(&dst).unwrap().file_type().is_symlink());
        assert_eq!(fs::read_link(&dst).unwrap(), target);
    }
}

#[test]
fn test_cp_multiple_sources() {
    let tmp = TempDir::new().unwrap();
    let src1 = tmp.path().join("src1.txt");
    let src2 = tmp.path().join("src2.txt");
    let dst_dir = tmp.path().join("dst_dir");
    fs::create_dir(&dst_dir).unwrap();
    fs::write(&src1, "one").unwrap();
    fs::write(&src2, "two").unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg(&src1)
        .arg(&src2)
        .arg(&dst_dir)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dst_dir.join("src1.txt")).unwrap(), "one");
    assert_eq!(fs::read_to_string(dst_dir.join("src2.txt")).unwrap(), "two");
}

#[test]
fn test_cp_archive() {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src_dir");
    let dst_dir = tmp.path().join("dst_dir");
    fs::create_dir(&src_dir).unwrap();
    let file = src_dir.join("a.txt");
    fs::write(&file, "a").unwrap();
    
    let old_time = FileTime::from_unix_time(2000, 0);
    set_file_times(&file, old_time, old_time).unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg("-a")
        .arg(&src_dir)
        .arg(&dst_dir)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dst_dir.join("a.txt")).unwrap(), "a");
    let dst_meta = fs::metadata(dst_dir.join("a.txt")).unwrap();
    assert_eq!(FileTime::from_last_modification_time(&dst_meta), old_time);
}

#[test]
fn test_cp_symlink_follow_command_line() {
    use std::os::windows::fs::symlink_file;
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("target.txt");
    let link = tmp.path().join("link.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&target, "content").unwrap();
    
    if symlink_file(&target, &link).is_ok() {
        Command::cargo_bin("cp")
            .unwrap()
            .arg("-H")
            .arg("-r")
            .arg(&link)
            .arg(&dst)
            .assert()
            .success();

        // -H follows command line, so dst should be a regular file with "content"
        assert!(!fs::symlink_metadata(&dst).unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&dst).unwrap(), "content");
    }
}

#[test]
fn test_cp_symlink_dereference() {
    use std::os::windows::fs::symlink_file;
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("target.txt");
    let link = tmp.path().join("link.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&target, "content").unwrap();
    
    if symlink_file(&target, &link).is_ok() {
        Command::cargo_bin("cp")
            .unwrap()
            .arg("-L")
            .arg(&link)
            .arg(&dst)
            .assert()
            .success();

        assert!(!fs::symlink_metadata(&dst).unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&dst).unwrap(), "content");
    }
}

#[test]
fn test_cp_multiple_sources_fail_not_dir() {
    let tmp = TempDir::new().unwrap();
    let src1 = tmp.path().join("src1.txt");
    let src2 = tmp.path().join("src2.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&src1, "one").unwrap();
    fs::write(&src2, "two").unwrap();
    fs::write(&dst, "existing").unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg(&src1)
        .arg(&src2)
        .arg(&dst)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a directory"));
}

#[test]
fn test_cp_recursive_merge() {
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src_dir");
    let dst_dir = tmp.path().join("dst_dir");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("a.txt"), "new_a").unwrap();
    fs::create_dir(&dst_dir).unwrap();
    fs::write(dst_dir.join("b.txt"), "existing_b").unwrap();

    Command::cargo_bin("cp")
        .unwrap()
        .arg("-r")
        .arg(&src_dir)
        .arg(&dst_dir)
        .assert()
        .success();

    assert_eq!(fs::read_to_string(dst_dir.join("src_dir/a.txt")).unwrap(), "new_a");
    assert_eq!(fs::read_to_string(dst_dir.join("b.txt")).unwrap(), "existing_b");
}

#[cfg(target_os = "windows")]
#[test]
fn test_cp_recursive_with_symlink() {
    use std::os::windows::fs::symlink_file;
    let tmp = TempDir::new().unwrap();
    let src_dir = tmp.path().join("src_dir");
    let dst_dir = tmp.path().join("dst_dir");
    fs::create_dir(&src_dir).unwrap();
    let target = tmp.path().join("target.txt");
    fs::write(&target, "content").unwrap();
    let link = src_dir.join("link.txt");
    
    if symlink_file(&target, &link).is_ok() {
        Command::cargo_bin("cp")
            .unwrap()
            .arg("-r") // defaults to -P
            .arg(&src_dir)
            .arg(&dst_dir)
            .assert()
            .success();

        assert!(fs::symlink_metadata(dst_dir.join("link.txt")).unwrap().file_type().is_symlink());
    }
}
