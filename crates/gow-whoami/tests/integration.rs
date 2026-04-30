use assert_cmd::Command;

#[test]
fn whoami_prints_username() {
    let out = Command::cargo_bin("whoami")
        .unwrap()
        .assert()
        .success()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // Output must be a single non-empty line
    assert!(!s.trim().is_empty(), "whoami output must not be empty");
    let lines: Vec<&str> = s.lines().collect();
    assert_eq!(lines.len(), 1, "whoami must print exactly one line: {:?}", s);
}

#[test]
fn whoami_exits_0() {
    Command::cargo_bin("whoami").unwrap().assert().code(0);
}
