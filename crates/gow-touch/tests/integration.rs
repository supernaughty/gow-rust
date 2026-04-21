//! Integration tests for `touch` (FILE-08). Covers every flag per D-19c.

use assert_cmd::Command;
use predicates::prelude::*;
use std::time::{Duration, SystemTime};

fn touch() -> Command {
    Command::cargo_bin("touch")
        .expect("touch binary not found — run `cargo build -p gow-touch` first")
}

#[test]
fn test_creates_missing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("newfile.txt");
    touch().arg(&target).assert().success();
    assert!(target.exists());
}

#[test]
fn test_c_flag_does_not_create() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("absent.txt");
    touch().args(["-c"]).arg(&target).assert().success();
    assert!(
        !target.exists(),
        "expected file NOT to be created under -c"
    );
}

#[test]
fn test_default_updates_mtime_to_now() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("ts.txt");
    std::fs::write(&target, b"x").unwrap();
    // Set mtime to a long-past date first, then touch.
    let past = filetime::FileTime::from_unix_time(1_000_000_000, 0); // 2001
    filetime::set_file_times(&target, past, past).unwrap();

    touch().arg(&target).assert().success();

    let md = std::fs::metadata(&target).unwrap();
    let now = SystemTime::now();
    let mtime = md.modified().unwrap();
    let diff = now.duration_since(mtime).unwrap_or(Duration::ZERO);
    assert!(diff < Duration::from_secs(60), "mtime should be near now");
}

#[test]
fn test_r_flag_copies_timestamps() {
    let tmp = tempfile::tempdir().unwrap();
    let ref_file = tmp.path().join("ref.txt");
    let target = tmp.path().join("target.txt");
    std::fs::write(&ref_file, b"r").unwrap();
    std::fs::write(&target, b"t").unwrap();

    let fixed = filetime::FileTime::from_unix_time(1_500_000_000, 0); // 2017
    filetime::set_file_times(&ref_file, fixed, fixed).unwrap();

    touch()
        .args(["-r"])
        .arg(&ref_file)
        .arg(&target)
        .assert()
        .success();

    let md = std::fs::metadata(&target).unwrap();
    let mtime = filetime::FileTime::from_last_modification_time(&md);
    assert_eq!(mtime.unix_seconds(), 1_500_000_000);
}

#[test]
fn test_d_flag_yesterday() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("ydayfile.txt");
    std::fs::write(&target, b"y").unwrap();

    touch()
        .args(["-d", "yesterday"])
        .arg(&target)
        .assert()
        .success();

    let md = std::fs::metadata(&target).unwrap();
    let mtime = filetime::FileTime::from_last_modification_time(&md);
    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let expected_yesterday = now_secs - 86_400;
    // Allow ±1 day slop for "yesterday" semantics (some parsers snap to midnight).
    assert!(
        (mtime.unix_seconds() - expected_yesterday).abs() < 2 * 86_400,
        "expected mtime near {expected_yesterday}, got {}",
        mtime.unix_seconds()
    );
}

#[test]
fn test_t_flag_strict_stamp() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("tflag.txt");
    std::fs::write(&target, b"t").unwrap();

    touch()
        .args(["-t", "202001010000"])
        .arg(&target)
        .assert()
        .success();

    let md = std::fs::metadata(&target).unwrap();
    let mtime = filetime::FileTime::from_last_modification_time(&md);
    // 2020-01-01T00:00 — approximate (local TZ shifts by ±24h).
    let expected = 1_577_836_800_i64;
    assert!(
        (mtime.unix_seconds() - expected).abs() < 86_400,
        "expected ~2020-01-01 local ({expected}), got {}",
        mtime.unix_seconds()
    );
}

#[test]
fn test_a_flag_updates_only_atime() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("aonly.txt");
    std::fs::write(&target, b"a").unwrap();

    let fixed = filetime::FileTime::from_unix_time(1_500_000_000, 0);
    filetime::set_file_times(&target, fixed, fixed).unwrap();

    touch().args(["-a"]).arg(&target).assert().success();

    let md = std::fs::metadata(&target).unwrap();
    let mtime = filetime::FileTime::from_last_modification_time(&md);
    // mtime should NOT have been updated — still near 1_500_000_000
    assert!(
        (mtime.unix_seconds() - 1_500_000_000).abs() < 10,
        "mtime should not have changed under -a; got {}",
        mtime.unix_seconds()
    );
}

#[test]
fn test_m_flag_updates_only_mtime() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("monly.txt");
    std::fs::write(&target, b"m").unwrap();

    let fixed = filetime::FileTime::from_unix_time(1_500_000_000, 0);
    filetime::set_file_times(&target, fixed, fixed).unwrap();

    touch().args(["-m"]).arg(&target).assert().success();

    let md = std::fs::metadata(&target).unwrap();
    let atime = filetime::FileTime::from_last_access_time(&md);
    // atime should NOT have been updated under -m
    assert!(
        (atime.unix_seconds() - 1_500_000_000).abs() < 10,
        "atime should not have changed under -m; got {}",
        atime.unix_seconds()
    );
}

#[test]
fn test_missing_operand_exits_1() {
    touch()
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("missing file operand"));
}

#[test]
fn test_bad_date_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("bad.txt");
    std::fs::write(&target, b"b").unwrap();

    touch()
        .args(["-d", "not a date xyzzy"])
        .arg(&target)
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_bad_flag_exits_1() {
    touch()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_gnu_error_format_on_bad_flag() {
    touch()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("touch:"));
}

#[test]
fn test_utf8_filename() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("안녕.txt");
    touch().arg(&target).assert().success();
    assert!(target.exists());
}

#[test]
#[cfg_attr(not(windows), ignore)]
fn test_h_flag_modifies_symlink_self() {
    // Windows requires Developer Mode or SeCreateSymbolicLinkPrivilege for
    // symlink_file. Skip gracefully if unprivileged.
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("realfile.txt");
    let link = tmp.path().join("link.txt");
    std::fs::write(&target, b"real").unwrap();

    #[cfg(windows)]
    let created = std::os::windows::fs::symlink_file(&target, &link).is_ok();
    #[cfg(not(windows))]
    let created = std::os::unix::fs::symlink(&target, &link).is_ok();

    if !created {
        eprintln!("[skip] symlink creation requires Developer Mode / SeCreateSymbolicLinkPrivilege");
        return;
    }

    // Set target mtime to a known baseline.
    let baseline = filetime::FileTime::from_unix_time(1_600_000_000, 0);
    filetime::set_file_times(&target, baseline, baseline).unwrap();

    // touch -h -d 2020-01-01 link.txt → only the LINK's mtime should change.
    touch()
        .args(["-h", "-d", "2020-01-01T00:00:00Z"])
        .arg(&link)
        .assert()
        .success();

    let target_md = std::fs::metadata(&target).unwrap();
    let target_mtime = filetime::FileTime::from_last_modification_time(&target_md);
    assert_eq!(
        target_mtime.unix_seconds(),
        1_600_000_000,
        "target mtime must be unchanged under -h"
    );
}
