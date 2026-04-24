use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn rm() -> Command {
    Command::cargo_bin("rm").unwrap()
}

#[test]
fn test_rm_file() {
    let ts = TempDir::new().unwrap();
    let file = ts.path().join("a.txt");
    fs::write(&file, "hello").unwrap();

    rm().arg(file.to_str().unwrap())
        .assert()
        .success();

    assert!(!file.exists());
}

#[test]
fn test_rm_multiple_files() {
    let ts = TempDir::new().unwrap();
    let a = ts.path().join("a.txt");
    let b = ts.path().join("b.txt");
    fs::write(&a, "a").unwrap();
    fs::write(&b, "b").unwrap();

    rm().arg(a.to_str().unwrap())
        .arg(b.to_str().unwrap())
        .assert()
        .success();

    assert!(!a.exists());
    assert!(!b.exists());
}

#[test]
fn test_rm_nonexistent_fails() {
    rm().arg("this_file_does_not_exist_9f3k2j")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));
}

#[test]
fn test_rm_force_nonexistent_success() {
    rm().arg("-f")
        .arg("this_file_does_not_exist_9f3k2j")
        .assert()
        .success();
}

#[test]
fn test_rm_dir_fails_without_recursive() {
    let ts = TempDir::new().unwrap();
    let dir = ts.path().join("mydir");
    fs::create_dir(&dir).unwrap();

    rm().arg(dir.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Is a directory"));
}

#[test]
fn test_rm_recursive() {
    let ts = TempDir::new().unwrap();
    let dir = ts.path().join("mydir");
    fs::create_dir(&dir).unwrap();
    let file = dir.join("a.txt");
    fs::write(&file, "data").unwrap();

    rm().arg("-r")
        .arg(dir.to_str().unwrap())
        .assert()
        .success();

    assert!(!dir.exists());
}

#[test]
fn test_rm_verbose() {
    let ts = TempDir::new().unwrap();
    let file = ts.path().join("a.txt");
    fs::write(&file, "hello").unwrap();

    rm().arg("-v")
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"));
}

#[test]
fn test_rm_preserve_root() {
    // We can't really remove C:\ in a test, but we can check if it refuses
    rm().arg("C:\\")
        .assert()
        .failure()
        .stderr(predicate::str::contains("it is dangerous to operate recursively"));
}

#[test]
fn test_rm_force_readonly() {
    let ts = TempDir::new().unwrap();
    let file = ts.path().join("readonly.txt");
    fs::write(&file, "data").unwrap();

    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&file, perms).unwrap();

    // Without -f, it might prompt (which fails in non-interactive shell) or just fail.
    // In our implementation, if it's not interactive, it will try to prompt and fail to read from stdin.
    
    rm().arg("-f")
        .arg(file.to_str().unwrap())
        .assert()
        .success();

    assert!(!file.exists());
}

#[test]
fn test_rm_symlink() {
    let ts = TempDir::new().unwrap();
    let target = ts.path().join("target.txt");
    fs::write(&target, "target content").unwrap();
    let link = ts.path().join("link.txt");

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::symlink_file;
        if symlink_file(&target, &link).is_ok() {
            rm().arg(link.to_str().unwrap())
                .assert()
                .success();
            assert!(!link.exists());
            assert!(target.exists());
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&target, &link).unwrap();
        rm().arg(link.to_str().unwrap())
            .assert()
            .success();
        assert!(!link.exists());
        assert!(target.exists());
    }
}

#[test]
fn test_rm_recursive_with_readonly() {
    let ts = TempDir::new().unwrap();
    let dir = ts.path().join("mydir");
    fs::create_dir(&dir).unwrap();
    let file = dir.join("readonly.txt");
    fs::write(&file, "data").unwrap();

    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&file, perms).unwrap();

    rm().arg("-rf")
        .arg(dir.to_str().unwrap())
        .assert()
        .success();

    assert!(!dir.exists());
}

#[test]
fn test_rm_no_args_fails() {
    rm().assert()
        .failure()
        .stderr(predicate::str::contains("missing operand"));
}

#[test]
fn test_rm_recursive_nested() {
    let ts = TempDir::new().unwrap();
    let dir1 = ts.path().join("dir1");
    let dir2 = dir1.join("dir2");
    fs::create_dir_all(&dir2).unwrap();
    fs::write(dir1.join("a.txt"), "a").unwrap();
    fs::write(dir2.join("b.txt"), "b").unwrap();

    rm().arg("-r")
        .arg(dir1.to_str().unwrap())
        .assert()
        .success();

    assert!(!dir1.exists());
}

#[test]
fn test_rm_recursive_empty_dir() {
    let ts = TempDir::new().unwrap();
    let dir = ts.path().join("empty");
    fs::create_dir(&dir).unwrap();

    rm().arg("-r")
        .arg(dir.to_str().unwrap())
        .assert()
        .success();

    assert!(!dir.exists());
}

#[test]
fn test_rm_recursive_verbose() {
    let ts = TempDir::new().unwrap();
    let dir = ts.path().join("dir");
    fs::create_dir(&dir).unwrap();
    let file = dir.join("a.txt");
    fs::write(&file, "a").unwrap();

    rm().arg("-rv")
        .arg(dir.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("removed directory"))
        .stdout(predicate::str::contains("removed '"));
}

#[test]
fn test_rm_force_dir_still_fails_without_recursive() {
    let ts = TempDir::new().unwrap();
    let dir = ts.path().join("dir");
    fs::create_dir(&dir).unwrap();

    rm().arg("-f")
        .arg(dir.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Is a directory"));
}

#[test]
fn test_rm_interactive_yes() {
    let ts = TempDir::new().unwrap();
    let file = ts.path().join("a.txt");
    fs::write(&file, "a").unwrap();

    rm().arg("-i")
        .arg(file.to_str().unwrap())
        .write_stdin("y\n")
        .assert()
        .success();

    assert!(!file.exists());
}

#[test]
fn test_rm_interactive_no() {
    let ts = TempDir::new().unwrap();
    let file = ts.path().join("a.txt");
    fs::write(&file, "a").unwrap();

    rm().arg("-i")
        .arg(file.to_str().unwrap())
        .write_stdin("n\n")
        .assert()
        .success();

    assert!(file.exists());
}
