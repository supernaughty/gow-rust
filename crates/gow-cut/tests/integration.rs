use assert_cmd::Command;

#[test]
fn test_cut_bytes() {
    let mut cmd = Command::cargo_bin("cut").unwrap();
    cmd.arg("-b").arg("1-3,5")
        .write_stdin("abcdefg")
        .assert()
        .success()
        .stdout("abce\n");
}

#[test]
fn test_cut_fields() {
    let mut cmd = Command::cargo_bin("cut").unwrap();
    cmd.arg("-d").arg(",").arg("-f").arg("1,3")
        .write_stdin("a,b,c,d")
        .assert()
        .success()
        .stdout("a,c\n");
}

#[test]
fn test_cut_complement() {
    let mut cmd = Command::cargo_bin("cut").unwrap();
    cmd.arg("-b").arg("2").arg("--complement")
        .write_stdin("abc")
        .assert()
        .success()
        .stdout("ac\n");
}

#[test]
fn test_cut_chars_unicode() {
    let mut cmd = Command::cargo_bin("cut").unwrap();
    cmd.arg("-c").arg("1,3")
        .write_stdin("안녕세")
        .assert()
        .success()
        .stdout("안세\n");
}
