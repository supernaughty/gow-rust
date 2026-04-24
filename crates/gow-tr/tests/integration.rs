use assert_cmd::Command;

#[test]
fn test_tr_translate() {
    let mut cmd = Command::cargo_bin("tr").unwrap();
    cmd.arg("a-z").arg("A-Z")
        .write_stdin("hello world")
        .assert()
        .success()
        .stdout("HELLO WORLD");
}

#[test]
fn test_tr_delete() {
    let mut cmd = Command::cargo_bin("tr").unwrap();
    cmd.arg("-d").arg("l")
        .write_stdin("hello world")
        .assert()
        .success()
        .stdout("heo word");
}

#[test]
fn test_tr_squeeze() {
    let mut cmd = Command::cargo_bin("tr").unwrap();
    cmd.arg("-s").arg("l")
        .write_stdin("hello world")
        .assert()
        .success()
        .stdout("helo world");
}

#[test]
fn test_tr_complement() {
    let mut cmd = Command::cargo_bin("tr").unwrap();
    cmd.arg("-c").arg("a-z").arg("X")
        .write_stdin("hello 123 WORLD")
        .assert()
        .success()
        .stdout("helloXXXXXXXXXX");
}
