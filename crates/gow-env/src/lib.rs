//! `uu_env`: GNU `env` ported to Windows with UTF-8 + long-path support.
//!
//! Supports the full flag set from CONTEXT.md D-19a:
//!   `-i` / `--ignore-environment`   start with an empty env
//!   `-u` / `--unset=NAME`           remove NAME from env (may repeat)
//!   `-C` / `--chdir=DIR`            cwd to DIR before spawning
//!   `-S` / `--split-string=STRING`  parse STRING into extra argv per GNU `env -S`
//!   `-0` / `--null`                 NUL-separate output (listing mode)
//!   `-v` / `--debug`                print exec trace to stderr
//!   `--`                            end-of-options
//!
//! Spawns children via `std::process::Command` with an argv array — NEVER a
//! shell string (D-19b, Phase 1 Pitfall #5). This guarantees the `-S` parser
//! cannot be used to inject shell meta-characters; the planner's threat
//! register (T-02-09-01) is satisfied by grep-able invariants in this file.
//!
//! Exit codes (GNU convention):
//!   0   child succeeded (status propagated)
//!   1   clap argument error (via gow_core::args::parse_gnu)
//!   125 env's own setup failure (bad -S, missing -C directory, etc.)
//!   127 command not found / spawn error

mod split;

use std::ffi::OsString;
use std::io::Write;
use std::process::Command as StdCommand;

use clap::{Arg, ArgAction, Command};

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    // Preprocess argv: rewrite the bare short flag `-0` to its long form
    // `--null`. Rationale — `gow_core::args::parse_gnu` enables clap's
    // `allow_negative_numbers(true)` globally so that utilities like `head -5`
    // can parse numeric shorthand (D-05). As a side effect, `-0` looks like
    // the negative integer zero and clap routes it to the nearest positional
    // argument instead of our `null` flag. Rewriting to `--null` sidesteps the
    // ambiguity without touching the shared parser. We stop rewriting at the
    // first non-flag or `--` so tokens meant for the child command are
    // preserved verbatim.
    let args: Vec<OsString> = args.into_iter().collect();
    let args = rewrite_short_null_flag(args);

    let matches = gow_core::args::parse_gnu(uu_app(), args);

    let ignore_env = matches.get_flag("ignore-environment");
    let null_sep = matches.get_flag("null");
    let verbose = matches.get_flag("debug");
    let chdir: Option<String> = matches.get_one::<String>("chdir").cloned();

    let unset_names: Vec<String> = matches
        .get_many::<String>("unset")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    let split_string: Option<String> = matches.get_one::<String>("split-string").cloned();

    let mut raw_operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    // Apply -S: expand STRING into tokens and prepend to operands.
    // Per Pitfall 6, `${VAR}` substitution reads the CURRENT (pre-clear) env, so
    // `env -iS 'foo=${USER}'` expands `${USER}` against the outer environment
    // BEFORE `-i` wipes it for the child. We therefore look up against
    // `std::env::var` here, regardless of `ignore_env`.
    if let Some(s) = split_string.as_ref() {
        let lookup = |n: &str| std::env::var(n).ok();
        match split::split(s, lookup) {
            Ok(mut tokens) => {
                tokens.append(&mut raw_operands);
                raw_operands = tokens;
            }
            Err(e) => {
                eprintln!("env: {e}");
                return 125;
            }
        }
    }

    // Split operands into NAME=VALUE assignments vs COMMAND + ARGS.
    // First token containing '=' whose name is a valid identifier start
    // (alpha or '_') is an assignment; anything else is the command.
    let mut assignments: Vec<(String, String)> = Vec::new();
    let mut cmd_idx: Option<usize> = None;
    for (i, op) in raw_operands.iter().enumerate() {
        if let Some(eq) = op.find('=') {
            let (name, value_with_eq) = op.split_at(eq);
            let first_ok = name
                .chars()
                .next()
                .map(|c| c.is_alphabetic() || c == '_')
                .unwrap_or(false);
            if !name.is_empty() && first_ok {
                // value_with_eq starts with '=' — skip that one byte.
                assignments.push((name.to_string(), value_with_eq[1..].to_string()));
                continue;
            }
        }
        cmd_idx = Some(i);
        break;
    }

    // Build the child environment.
    let mut child_env: Vec<(String, String)> = if ignore_env {
        Vec::new()
    } else {
        std::env::vars().collect()
    };
    for name in &unset_names {
        child_env.retain(|(k, _)| k != name);
    }
    for (k, v) in &assignments {
        if let Some(entry) = child_env.iter_mut().find(|(key, _)| key == k) {
            entry.1 = v.clone();
        } else {
            child_env.push((k.clone(), v.clone()));
        }
    }

    match cmd_idx {
        Some(idx) => spawn_child(&raw_operands, idx, &child_env, chdir.as_deref(), verbose),
        None => {
            // No command → print the environment.
            print_env(&child_env, null_sep);
            0
        }
    }
}

/// Spawn the trailing COMMAND with the built child env.
///
/// INVARIANT: we pass `command_name` directly to `StdCommand::new` and
/// `command_args` as a `&[String]` slice to `.args(...)`. No shell string
/// interpolation happens anywhere on this path (D-19b). An integration
/// test greps this file to ensure no shell-spawn pathway ever appears in
/// source (see tests/integration.rs::test_no_shell_spawn_in_source).
fn spawn_child(
    raw_operands: &[String],
    idx: usize,
    child_env: &[(String, String)],
    chdir: Option<&str>,
    verbose: bool,
) -> i32 {
    let command_name = &raw_operands[idx];
    let command_args = &raw_operands[idx + 1..];

    if verbose {
        eprintln!("env: executing: {command_name}");
        for a in command_args {
            eprintln!("env:           {a}");
        }
    }

    let mut cmd = StdCommand::new(command_name);
    cmd.args(command_args); // argv array — NO shell (D-19b)
    cmd.env_clear();
    for (k, v) in child_env {
        cmd.env(k, v);
    }
    if let Some(dir) = chdir {
        if !std::path::Path::new(dir).is_dir() {
            eprintln!("env: cannot change directory to '{dir}': not a directory");
            return 125;
        }
        cmd.current_dir(dir);
    }

    match cmd.status() {
        Ok(status) => status.code().unwrap_or(127),
        Err(e) => {
            eprintln!("env: {command_name}: {e}");
            127
        }
    }
}

fn print_env(child_env: &[(String, String)], null_sep: bool) {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let terminator: &[u8] = if null_sep { b"\0" } else { b"\n" };
    for (k, v) in child_env {
        let line = format!("{k}={v}");
        let _ = out.write_all(line.as_bytes());
        let _ = out.write_all(terminator);
    }
}

/// Rewrite the GNU short flag `-0` to its long form `--null` until we hit a
/// non-flag token or the `--` end-of-options marker. Leaves everything past
/// that point untouched so the trailing command+args can contain literal
/// `-0` without being clobbered.
fn rewrite_short_null_flag(args: Vec<OsString>) -> Vec<OsString> {
    let mut out = Vec::with_capacity(args.len());
    let mut past_options = false;
    for (i, arg) in args.into_iter().enumerate() {
        if i == 0 {
            // argv[0] is the program name — never rewrite.
            out.push(arg);
            continue;
        }
        if past_options {
            out.push(arg);
            continue;
        }
        // `--` ends option parsing; emit verbatim and stop rewriting.
        if arg == *"--" {
            past_options = true;
            out.push(arg);
            continue;
        }
        if arg == *"-0" {
            out.push(OsString::from("--null"));
            continue;
        }
        // Any non-flag token (first positional: assignment or command) also
        // ends option parsing for our purposes: anything after belongs to the
        // child and must not be mutated.
        if let Some(s) = arg.to_str() {
            if !s.starts_with('-') {
                past_options = true;
            }
        } else {
            past_options = true;
        }
        out.push(arg);
    }
    out
}

fn uu_app() -> Command {
    Command::new("env")
        .about("Set each NAME to VALUE in the environment and run COMMAND.")
        .arg(
            Arg::new("ignore-environment")
                .short('i')
                .long("ignore-environment")
                .action(ArgAction::SetTrue)
                .help("start with an empty environment"),
        )
        .arg(
            Arg::new("unset")
                .short('u')
                .long("unset")
                .action(ArgAction::Append)
                .num_args(1)
                .value_name("NAME")
                .help("remove variable from the environment"),
        )
        .arg(
            Arg::new("chdir")
                .short('C')
                .long("chdir")
                .num_args(1)
                .value_name("DIR")
                .help("change working directory to DIR before running COMMAND"),
        )
        .arg(
            Arg::new("split-string")
                .short('S')
                .long("split-string")
                .num_args(1)
                .value_name("STRING")
                .help("parse STRING as GNU env -S and splice into argv"),
        )
        .arg(
            Arg::new("null")
                .short('0')
                .long("null")
                .action(ArgAction::SetTrue)
                .help("end each output line with NUL, not newline"),
        )
        .arg(
            Arg::new("debug")
                .short('v')
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("print verbose information for each processing step"),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true)
                .allow_hyphen_values(true),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uu_app_builds_without_panic() {
        // Smoke test: construct the command graph. Individual flag behaviour
        // is covered by the integration suite.
        let _ = uu_app();
    }
}
