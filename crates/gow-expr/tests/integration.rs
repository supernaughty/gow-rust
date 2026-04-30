use assert_cmd::Command;

#[test]
fn expr_addition() {
    Command::cargo_bin("expr").unwrap()
        .args(["3", "+", "4"])
        .assert().code(0).stdout("7\n");
}

#[test]
fn expr_zero_result_exits_1() {
    // CRITICAL: zero result must exit 1, not 0
    Command::cargo_bin("expr").unwrap()
        .args(["3", "-", "3"])
        .assert().code(1).stdout("0\n");
}

#[test]
fn expr_no_args_exits_2() {
    Command::cargo_bin("expr").unwrap()
        .assert().code(2);
}

#[test]
fn expr_incomplete_exits_2() {
    Command::cargo_bin("expr").unwrap()
        .args(["3", "+"])
        .assert().code(2);
}

#[test]
fn expr_nonempty_string_exits_0() {
    Command::cargo_bin("expr").unwrap()
        .args(["foo"])
        .assert().code(0).stdout("foo\n");
}

#[test]
fn expr_empty_string_exits_1() {
    Command::cargo_bin("expr").unwrap()
        .args([""])
        .assert().code(1).stdout("\n");
}

#[test]
fn expr_multiply() {
    Command::cargo_bin("expr").unwrap()
        .args(["2", "*", "3"])
        .assert().code(0).stdout("6\n");
}

#[test]
fn expr_integer_division() {
    Command::cargo_bin("expr").unwrap()
        .args(["10", "/", "3"])
        .assert().code(0).stdout("3\n");
}

#[test]
fn expr_modulo() {
    Command::cargo_bin("expr").unwrap()
        .args(["10", "%", "3"])
        .assert().code(0).stdout("1\n");
}

#[test]
fn expr_comparison_true() {
    Command::cargo_bin("expr").unwrap()
        .args(["5", "=", "5"])
        .assert().code(0).stdout("1\n");
}

#[test]
fn expr_comparison_false_exits_1() {
    // comparison false: result is "0" → exit 1
    Command::cargo_bin("expr").unwrap()
        .args(["5", "=", "6"])
        .assert().code(1).stdout("0\n");
}

#[test]
fn expr_greater_than() {
    Command::cargo_bin("expr").unwrap()
        .args(["5", ">", "3"])
        .assert().code(0).stdout("1\n");
}

#[test]
fn expr_colon_match_length() {
    Command::cargo_bin("expr").unwrap()
        .args(["hello", ":", "hel"])
        .assert().code(0).stdout("3\n");
}
