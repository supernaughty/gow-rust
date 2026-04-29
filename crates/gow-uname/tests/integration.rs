use assert_cmd::Command;
use predicates::str::contains;
use predicates::prelude::*;

#[test]
fn uname_s_prints_windows_nt() {
    Command::cargo_bin("uname")
        .unwrap()
        .arg("-s")
        .assert()
        .success()
        .code(0)
        .stdout("Windows_NT\n");
}

#[test]
fn uname_r_version_format() {
    let out = Command::cargo_bin("uname")
        .unwrap()
        .arg("-r")
        .assert()
        .success()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap().trim().to_string();
    // Must match MAJOR.MINOR.BUILD format
    let parts: Vec<&str> = s.split('.').collect();
    assert_eq!(parts.len(), 3, "uname -r must be MAJOR.MINOR.BUILD: {:?}", s);
    for part in &parts {
        part.parse::<u32>()
            .unwrap_or_else(|_| panic!("uname -r part must be numeric: {:?}", part));
    }
    // Must NOT be 6.2.x (that's GetVersionExW lying on Windows 8.1+)
    assert!(s != "6.2", "uname -r must use RtlGetVersion, not GetVersionExW (got 6.2)");
}

#[test]
fn uname_m_valid_arch() {
    let out = Command::cargo_bin("uname")
        .unwrap()
        .arg("-m")
        .assert()
        .success()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let arch = String::from_utf8(out).unwrap().trim().to_string();
    assert!(
        matches!(arch.as_str(), "x86_64" | "i686" | "aarch64"),
        "uname -m must be x86_64/i686/aarch64, got: {:?}",
        arch
    );
}

#[test]
fn uname_n_hostname_nonempty() {
    let out = Command::cargo_bin("uname")
        .unwrap()
        .arg("-n")
        .assert()
        .success()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let hostname = String::from_utf8(out).unwrap().trim().to_string();
    assert!(!hostname.is_empty(), "uname -n must print a hostname");
}

#[test]
fn uname_a_contains_windows_nt() {
    Command::cargo_bin("uname")
        .unwrap()
        .arg("-a")
        .assert()
        .success()
        .code(0)
        .stdout(contains("Windows_NT"));
}

#[test]
fn uname_no_flags_prints_sysname() {
    // Default (no flags) = kernel name only = Windows_NT
    Command::cargo_bin("uname")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .stdout("Windows_NT\n");
}
