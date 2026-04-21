//! Integration tests for `env` (UTIL-03).
//!
//! Per D-30b each crate has ≥4 baseline tests (default behavior, exit-code,
//! GNU error format, UTF-8 roundtrip) plus flag-specific coverage per D-19a.

use assert_cmd::Command;
use predicates::prelude::*;

fn env_cmd() -> Command {
    Command::cargo_bin("env").expect("env binary not found — run `cargo build -p gow-env` first")
}

/// UTIL-03 / D-19a: no args → print current environment as NAME=VALUE lines.
#[test]
fn test_no_args_prints_env() {
    env_cmd()
        .env("GOW_TEST_MARKER", "hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("GOW_TEST_MARKER=hello"));
}

/// D-19a `-i`: starts the child with an empty environment. With no command,
/// the empty env means stdout is effectively blank.
#[test]
fn test_i_flag_empty_env() {
    env_cmd()
        .arg("-i")
        .env("GOW_TEST_MARKER", "hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("GOW_TEST_MARKER").not());
}

/// D-19a `-u NAME`: removes NAME from the env; other vars survive.
#[test]
fn test_u_unsets_variable() {
    env_cmd()
        .args(["-u", "GOW_TEST_MARKER"])
        .env("GOW_TEST_MARKER", "should_be_gone")
        .env("OTHER_VAR_GOW", "kept")
        .assert()
        .success()
        .stdout(predicate::str::contains("GOW_TEST_MARKER").not())
        .stdout(predicate::str::contains("OTHER_VAR_GOW=kept"));
}

/// UTIL-03: `env NAME=value COMMAND` exports NAME=value for the child.
/// We exercise this by nesting gow-env: the outer sets TESTV, the inner
/// prints its env, and we grep for TESTV=ok.
#[test]
fn test_assignment_sets_var_and_spawns_command() {
    let inner = assert_cmd::cargo::cargo_bin("env");
    env_cmd()
        .arg("TESTV=ok")
        .arg(inner)
        .assert()
        .success()
        .stdout(predicate::str::contains("TESTV=ok"));
}

/// D-19a `-0`: NUL-separated listing instead of newline.
#[test]
fn test_0_flag_null_separator() {
    let inner = assert_cmd::cargo::cargo_bin("env");
    let out = env_cmd()
        .args(["-i"])
        .arg("A=1")
        .arg("B=2")
        .arg(&inner)
        .args(["-0"])
        .output()
        .expect("inner env call failed to spawn");
    assert!(
        out.stdout.contains(&0u8),
        "expected NUL byte in stdout with -0, got: {:?}",
        out.stdout
    );
}

/// D-19a `-S STRING` (+ RESEARCH.md Q7): split-string spec is parsed and
/// spliced into the child argv. `-i -S 'A=1 B=2 <bin>'` launches the inner
/// bin with a freshly-populated env containing A and B.
///
/// The path is wrapped in SINGLE quotes so the state machine treats `\t`,
/// `\x`, etc. as literal characters (required on Windows where paths like
/// `…\target\…` would otherwise trigger tab/escape expansion per the GNU
/// `env -S` escape table).
#[test]
fn test_s_split_string_expansion() {
    let bin = assert_cmd::cargo::cargo_bin("env");
    let bin_str = bin.to_string_lossy().to_string();
    env_cmd()
        .args(["-i", "-S", &format!("A=1 B=2 '{bin_str}'")])
        .assert()
        .success()
        .stdout(predicate::str::contains("A=1"))
        .stdout(predicate::str::contains("B=2"));
}

/// D-19a `-C DIR`: change cwd before exec. Verified by handing the inner
/// env call a valid tempdir — we only assert exit 0 here; deeper CWD
/// inspection would require a cwd-printing helper we don't have.
#[test]
fn test_chdir_flag_changes_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let inner = assert_cmd::cargo::cargo_bin("env");
    env_cmd()
        .arg("-C")
        .arg(tmp.path())
        .arg(inner)
        .assert()
        .success();
}

/// D-19a `-C`: non-existent directory aborts with the env-own failure code 125.
#[test]
fn test_chdir_missing_directory_fails_125() {
    let inner = assert_cmd::cargo::cargo_bin("env");
    env_cmd()
        .arg("-C")
        .arg("Z:/definitely/does/not/exist/gow-env-test")
        .arg(inner)
        .assert()
        .failure()
        .code(125);
}

/// RESEARCH.md Q7: `$VAR` without braces is a parse error → exit 125 + GNU
/// `env:` stderr prefix.
#[test]
fn test_bad_s_missing_brace_errors() {
    env_cmd()
        .args(["-S", "$VAR"])
        .assert()
        .failure()
        .code(125)
        .stderr(predicate::str::starts_with("env:"));
}

/// Non-existent command → exit 127 (GNU convention).
#[test]
fn test_nonexistent_command_exits_127() {
    env_cmd()
        .arg("/this/binary/definitely/does/not/exist/xyzzy")
        .assert()
        .failure()
        .code(127);
}

/// Per clap + GNU env semantics, an unknown token in the command slot is
/// treated as the command name (GNU env does not validate it against its own
/// flag list — everything after the recognised leading options becomes the
/// child invocation). Spawning the non-existent program yields exit 127,
/// matching `test_nonexistent_command_exits_127`. This test anchors that
/// contract so a future change to clap's grammar (e.g. making
/// `--completely-unknown-xyz` an error instead) surfaces here.
#[test]
fn test_unknown_long_flag_treated_as_command_exits_127() {
    env_cmd()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .code(127);
}

/// D-30b GNU-format error: `env: <message>` prefix on stderr for runtime
/// errors (here: spawn failure of an unknown command).
#[test]
fn test_gnu_error_format() {
    env_cmd()
        .arg("--completely-unknown-xyz")
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("env:"));
}

/// D-30b UTF-8 round-trip: non-ASCII values survive through assignment +
/// child-env printing without corruption.
#[test]
fn test_utf8_var_value_roundtrip() {
    let inner = assert_cmd::cargo::cargo_bin("env");
    env_cmd()
        .arg("-i")
        .arg("GREETING=안녕")
        .arg(inner)
        .assert()
        .success()
        .stdout(predicate::str::contains("GREETING=안녕"));
}

/// T-02-09-01 (threat model): the gow-env source must never spawn a shell.
/// Greps the lib.rs for `Command::new("sh" | "bash" | "cmd")`. This test
/// enforces D-19b at the source level — if a future refactor slips in a
/// shell pathway, CI fails here.
#[test]
fn test_no_shell_spawn_in_source() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"),
    )
    .expect("failed to read src/lib.rs");

    // Tolerate whitespace variants; reject any `Command::new("sh"|"bash"|"cmd")`.
    for shell in &["\"sh\"", "\"bash\"", "\"cmd\"", "\"cmd.exe\"", "\"powershell\""] {
        let needle = format!("Command::new({shell}");
        assert!(
            !src.contains(&needle),
            "forbidden shell spawn pattern found: {needle}"
        );
        // Also check with a space: Command::new( "sh" ...
        let needle_sp = format!("Command::new( {shell}");
        assert!(
            !src.contains(&needle_sp),
            "forbidden shell spawn pattern found: {needle_sp}"
        );
    }

    // Additional positive assertion: the canonical argv-array call site exists.
    assert!(
        src.contains("StdCommand::new(command_name)"),
        "expected argv-array spawn `StdCommand::new(command_name)` to remain in lib.rs"
    );
    assert!(
        src.contains(".args(command_args)"),
        "expected `.args(command_args)` (argv array passthrough) to remain in lib.rs"
    );
}
