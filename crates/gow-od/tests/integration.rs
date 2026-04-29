//! Integration tests for gow-od (U-05).
//! Covers -t type specifiers (o/x/d/u/c with various sizes),
//! -A address formats (o/x/d/n), -N byte limit, file reading, and error cases.

use assert_cmd::Command;

// ============================================================
// Address format tests
// ============================================================

#[test]
fn od_empty_input() {
    // Empty stdin → single address-only line
    Command::cargo_bin("od")
        .unwrap()
        .write_stdin("")
        .assert()
        .success()
        .stdout("0000000\n");
}

#[test]
fn od_default_octal_two_byte() {
    // "ab" = 0x61 0x62, LE u16 = 0x6261 = octal 061141
    Command::cargo_bin("od")
        .unwrap()
        .write_stdin("ab")
        .assert()
        .success()
        .stdout("0000000 061141\n0000002\n");
}

#[test]
fn od_default_four_bytes() {
    // "abcd" → two LE u16 words: 0x6261=061141, 0x6463=062143
    Command::cargo_bin("od")
        .unwrap()
        .write_stdin("abcd")
        .assert()
        .success()
        .stdout("0000000 061141 062143\n0000004\n");
}

#[test]
fn od_a_x_hex_address() {
    // 16 bytes: "0123456789abcdef"
    // First-row address = 0000000 (0 in hex)
    // Final address = 0000010 (16 in hex)
    let out = Command::cargo_bin("od")
        .unwrap()
        .args(["-A", "x"])
        .write_stdin("0123456789abcdef")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.starts_with("0000000 "), "expected hex addr 0000000, got: {s:?}");
    assert!(s.ends_with("0000010\n"), "expected hex final addr 0000010, got: {s:?}");
}

#[test]
fn od_a_d_decimal_address() {
    // 16 bytes: final address = 16 in decimal = "0000016" (7 digits)
    let out = Command::cargo_bin("od")
        .unwrap()
        .args(["-A", "d"])
        .write_stdin("0123456789abcdef")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.starts_with("0000000 "), "expected decimal addr 0000000, got: {s:?}");
    assert!(s.ends_with("0000016\n"), "expected decimal final addr 0000016, got: {s:?}");
}

#[test]
fn od_a_n_no_address() {
    // -A n suppresses address column; final line is also empty (just \n)
    Command::cargo_bin("od")
        .unwrap()
        .args(["-A", "n", "-t", "x1"])
        .write_stdin("A")
        .assert()
        .success()
        .stdout(" 41\n\n");
}

// ============================================================
// Type specifier tests
// ============================================================

#[test]
fn od_t_o1_octal_byte() {
    // "AB" = 0x41 0x42 = octal 101 102
    Command::cargo_bin("od")
        .unwrap()
        .args(["-t", "o1"])
        .write_stdin("AB")
        .assert()
        .success()
        .stdout("0000000 101 102\n0000002\n");
}

#[test]
fn od_t_x1_hex_byte() {
    // "AB" = 0x41 0x42
    Command::cargo_bin("od")
        .unwrap()
        .args(["-t", "x1"])
        .write_stdin("AB")
        .assert()
        .success()
        .stdout("0000000 41 42\n0000002\n");
}

#[test]
fn od_t_d1_signed_decimal() {
    // "AB" = 65 66 as signed i8 → formatted right-aligned in 4 chars
    Command::cargo_bin("od")
        .unwrap()
        .args(["-t", "d1"])
        .write_stdin("AB")
        .assert()
        .success()
        .stdout("0000000   65   66\n0000002\n");
}

#[test]
fn od_t_u1_unsigned_decimal() {
    // "AB" = 65 66 as unsigned u8 → formatted right-aligned in 3 chars
    Command::cargo_bin("od")
        .unwrap()
        .args(["-t", "u1"])
        .write_stdin("AB")
        .assert()
        .success()
        .stdout("0000000  65  66\n0000002\n");
}

#[test]
fn od_t_c_chars() {
    // "A\n" → '   A' '  \n' in 4-char fields
    Command::cargo_bin("od")
        .unwrap()
        .args(["-t", "c"])
        .write_stdin("A\n")
        .assert()
        .success()
        .stdout("0000000   A  \\n\n0000002\n");
}

#[test]
fn od_t_c_control_chars() {
    // Tab and carriage return
    Command::cargo_bin("od")
        .unwrap()
        .args(["-t", "c"])
        .write_stdin("\t\r")
        .assert()
        .success()
        .stdout("0000000  \\t  \\r\n0000002\n");
}

#[test]
fn od_t_x2_hex_word() {
    // "abcd" = LE u16: 0x6261=6261, 0x6463=6463
    Command::cargo_bin("od")
        .unwrap()
        .args(["-t", "x2"])
        .write_stdin("abcd")
        .assert()
        .success()
        .stdout("0000000 6261 6463\n0000004\n");
}

// ============================================================
// -N byte limit
// ============================================================

#[test]
fn od_n_limit_4_bytes() {
    // "0123456789" limited to 4 bytes: 0x30 0x31 0x32 0x33
    Command::cargo_bin("od")
        .unwrap()
        .args(["-N", "4", "-t", "x1"])
        .write_stdin("0123456789")
        .assert()
        .success()
        .stdout("0000000 30 31 32 33\n0000004\n");
}

#[test]
fn od_n_limit_2_bytes_default_type() {
    // 2 bytes of "abcd" → only "ab" → single o2 word 061141
    Command::cargo_bin("od")
        .unwrap()
        .args(["-N", "2"])
        .write_stdin("abcd")
        .assert()
        .success()
        .stdout("0000000 061141\n0000002\n");
}

// ============================================================
// 16-byte full row
// ============================================================

#[test]
fn od_full_16_byte_row() {
    // 16 bytes of 0x00 → 8 o2 words of 000000
    let input: Vec<u8> = vec![0u8; 16];
    let out = Command::cargo_bin("od")
        .unwrap()
        .write_stdin(input.as_slice())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // First line: address 0000000 + 8 zero words
    assert!(
        s.starts_with("0000000 000000 000000 000000 000000 000000 000000 000000 000000\n"),
        "full 16-byte row failed: {s:?}"
    );
    // Final address line: 0000020 (16 in octal)
    assert!(s.ends_with("0000020\n"), "final addr for 16 bytes should be 0000020, got: {s:?}");
}

// ============================================================
// File reading
// ============================================================

#[test]
fn od_reads_file() {
    use std::io::Write;
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    tmp.write_all(b"AB").unwrap();
    tmp.flush().unwrap();
    Command::cargo_bin("od")
        .unwrap()
        .args(["-t", "x1", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout("0000000 41 42\n0000002\n");
}

// ============================================================
// Error cases
// ============================================================

#[test]
fn od_missing_file_errors() {
    Command::cargo_bin("od")
        .unwrap()
        .arg("nonexistent_xyz_od_12345.bin")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn od_missing_file_stderr_prefix() {
    let output = Command::cargo_bin("od")
        .unwrap()
        .arg("nonexistent_xyz_od_12345.bin")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("od:"), "stderr should contain 'od:' prefix, got: {stderr:?}");
}
