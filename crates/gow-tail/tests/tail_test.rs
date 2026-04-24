use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use std::thread;
use std::time::Duration;

#[test]
fn test_tail_n_lines() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    for i in 1..=20 {
        writeln!(file, "line {}", i).unwrap();
    }

    let mut cmd = Command::cargo_bin("tail").unwrap();
    cmd.arg("-n").arg("5").arg(&file_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("line 16\nline 17\nline 18\nline 19\nline 20\n"));
}

#[test]
fn test_tail_n_plus_lines() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    for i in 1..=10 {
        writeln!(file, "line {}", i).unwrap();
    }

    let mut cmd = Command::cargo_bin("tail").unwrap();
    cmd.arg("-n").arg("+8").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("line 8\nline 9\nline 10\n");
}

#[test]
fn test_tail_c_bytes() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"1234567890").unwrap();

    let mut cmd = Command::cargo_bin("tail").unwrap();
    cmd.arg("-c").arg("3").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("890");
}

#[test]
fn test_tail_c_plus_bytes() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(b"1234567890").unwrap();

    let mut cmd = Command::cargo_bin("tail").unwrap();
    cmd.arg("-c").arg("+8").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("890");
}

#[test]
fn test_tail_multiple_files() {
    let dir = tempdir().unwrap();
    let f1 = dir.path().join("f1.txt");
    let f2 = dir.path().join("f2.txt");
    fs::write(&f1, "a\nb\n").unwrap();
    fs::write(&f2, "1\n2\n").unwrap();

    let mut cmd = Command::cargo_bin("tail").unwrap();
    cmd.arg(&f1).arg(&f2);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("==> ").and(predicate::str::contains("f1.txt")))
        .stdout(predicate::str::contains("a\nb\n"))
        .stdout(predicate::str::contains("==> ").and(predicate::str::contains("f2.txt")))
        .stdout(predicate::str::contains("1\n2\n"));
}

#[test]
fn test_tail_quiet() {
    let dir = tempdir().unwrap();
    let f1 = dir.path().join("f1.txt");
    let f2 = dir.path().join("f2.txt");
    fs::write(&f1, "a\n").unwrap();
    fs::write(&f2, "1\n").unwrap();

    let mut cmd = Command::cargo_bin("tail").unwrap();
    cmd.arg("-q").arg(&f1).arg(&f2);
    cmd.assert()
        .success()
        .stdout("a\n1\n")
        .stdout(predicate::str::contains("==>").not());
}

#[test]
fn test_tail_stdin() {
    let mut cmd = Command::cargo_bin("tail").unwrap();
    cmd.arg("-n").arg("2");
    cmd.write_stdin("line 1\nline 2\nline 3\n");
    cmd.assert()
        .success()
        .stdout("line 2\nline 3\n");
}

#[test]
fn test_tail_follow() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("follow.txt");
    fs::write(&file_path, "initial\n").unwrap();

    let mut cmd = std::process::Command::new(assert_cmd::cargo_bin!("tail"));
    cmd.arg("-f").arg("-s").arg("0.1").arg(&file_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().unwrap();

    // Give some time for tail to read initial content
    thread::sleep(Duration::from_millis(500));

    {
        let mut file = fs::OpenOptions::new().append(true).open(&file_path).unwrap();
        writeln!(file, "appended").unwrap();
    }

    // Give some time for notify to pick up the change
    thread::sleep(Duration::from_millis(500));

    child.kill().unwrap();

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("initial"));
    assert!(stdout.contains("appended"));
}

#[test]
fn test_tail_follow_truncation() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("truncate.txt");
    fs::write(&file_path, "long content that will be truncated\n").unwrap();

    let mut cmd = std::process::Command::new(assert_cmd::cargo_bin!("tail"));
    cmd.arg("-f").arg("-s").arg("0.1").arg(&file_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().unwrap();
    thread::sleep(Duration::from_millis(500));

    // Truncate file
    fs::write(&file_path, "new start\n").unwrap();
    thread::sleep(Duration::from_millis(500));

    child.kill().unwrap();
    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stderr.contains("file truncated"));
    assert!(stdout.contains("new start"));
}

#[test]
fn test_tail_z_zero_terminated() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("zero.txt");
    fs::write(&file_path, "a\0b\0c\0d\0").unwrap();

    let mut cmd = Command::cargo_bin("tail").unwrap();
    cmd.arg("-z").arg("-n").arg("2").arg(&file_path);
    cmd.assert()
        .success()
        .stdout("c\0d\0");
}

#[test]
fn test_tail_retry() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("retry.txt");

    let mut cmd = std::process::Command::new(assert_cmd::cargo_bin!("tail"));
    cmd.arg("-f").arg("--retry").arg("-s").arg("0.1").arg(&file_path);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().unwrap();
    thread::sleep(Duration::from_millis(500));

    // Create the file
    fs::write(&file_path, "created\n").unwrap();
    thread::sleep(Duration::from_millis(500));

    child.kill().unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("created"));
}

#[test]
fn test_tail_pid() {
    // This is hard to test reliably without spawning another process and getting its PID.
    // We can use the current process's PID for a quick "it terminates immediately" test if we use a finished process.
    // Or just spawn a dummy process.
    let dummy = std::process::Command::new("cmd").arg("/c").arg("exit 0").spawn().unwrap();
    let pid = dummy.id();
    // Wait for it to finish
    let _ = dummy.wait_with_output();

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("pid.txt");
    fs::write(&file_path, "content\n").unwrap();

    let mut cmd = Command::cargo_bin("tail").unwrap();
    cmd.arg("-f").arg("--pid").arg(pid.to_string()).arg("-s").arg("0.1").arg(&file_path);
    
    // It should terminate quickly because the PID is already dead
    let start = std::time::Instant::now();
    cmd.assert().success();
    assert!(start.elapsed() < Duration::from_secs(2));
}
