//! `uu_find`: GNU find — Windows port (R015 / FIND-01, FIND-02, FIND-03).
//!
//! Implements predicates -name / -iname / -type / -size / -mtime / -atime / -ctime /
//! -maxdepth / -mindepth, plus actions -print, -print0, -exec cmd {} \;.
//! Per CONTEXT.md D-01..D-06.

use anyhow::{anyhow, Result};
use clap::{CommandFactory, FromArgMatches, Parser};
use globset::{GlobBuilder, GlobMatcher};
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::{DirEntry, WalkDir};

#[derive(Parser, Debug)]
#[command(
    name = "find",
    about = "GNU find — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true,
    trailing_var_arg = false,
)]
struct Cli {
    /// Paths to search (default: current directory)
    paths: Vec<PathBuf>,

    /// Case-sensitive glob match against the basename
    #[arg(long = "name", value_name = "PATTERN", num_args = 1)]
    name: Option<String>,

    /// Case-insensitive glob match against the basename
    #[arg(long = "iname", value_name = "PATTERN", num_args = 1)]
    iname: Option<String>,

    /// Filter by entry type: f=file, d=directory, l=symlink
    #[arg(long = "type", value_name = "TYPE", num_args = 1)]
    type_: Option<String>,

    /// Filter by file size (+N/-N/N with optional c/k/M/G suffix)
    /// allow_hyphen_values: value may start with '-' (e.g. -size -1k)
    #[arg(long = "size", value_name = "SPEC", num_args = 1, allow_hyphen_values = true)]
    size: Option<String>,

    /// Filter by mtime in days (+N/-N/N)
    /// allow_hyphen_values: value may start with '-' (e.g. -mtime -1)
    #[arg(long = "mtime", value_name = "SPEC", num_args = 1, allow_hyphen_values = true)]
    mtime: Option<String>,

    /// Filter by atime in days (+N/-N/N) — see D-04 NTFS caveat
    /// allow_hyphen_values: value may start with '-' (e.g. -atime -1)
    #[arg(long = "atime", value_name = "SPEC", num_args = 1, allow_hyphen_values = true)]
    atime: Option<String>,

    /// Filter by ctime in days (+N/-N/N)
    /// allow_hyphen_values: value may start with '-' (e.g. -ctime -1)
    #[arg(long = "ctime", value_name = "SPEC", num_args = 1, allow_hyphen_values = true)]
    ctime: Option<String>,

    /// Maximum traversal depth (0 = path itself only)
    #[arg(long = "maxdepth", value_name = "N", num_args = 1)]
    maxdepth: Option<usize>,

    /// Minimum traversal depth (0 includes root)
    #[arg(long = "mindepth", value_name = "N", num_args = 1)]
    mindepth: Option<usize>,

    /// Null-delimited output (binary-safe)
    #[arg(long = "print0", action = clap::ArgAction::SetTrue)]
    print0: bool,

    /// Execute command per match: -exec cmd args... \;
    /// Collect all remaining args until literal ";"
    #[arg(long = "exec", value_name = "CMD", num_args = 1.., value_terminator = ";")]
    exec: Option<Vec<String>>,
}

/// Normalize GNU find's single-dash long flags to double-dash for clap.
///
/// GNU find uses single-dash long options (`-name`, `-type`, etc.) which clap
/// does not natively support. We rewrite them to `--name`, `--type`, etc.
/// before passing to clap. The literal `;` terminator for `-exec` and all
/// non-flag arguments (paths) are left untouched.
fn normalize_find_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    // The set of single-dash long flags that find uses
    const SINGLE_DASH_FLAGS: &[&str] = &[
        "-name", "-iname", "-type", "-size",
        "-mtime", "-atime", "-ctime",
        "-maxdepth", "-mindepth",
        "-print0", "-exec",
    ];

    args.into_iter()
        .map(|arg| {
            if let Some(s) = arg.to_str() {
                for flag in SINGLE_DASH_FLAGS {
                    if s == *flag {
                        // Rewrite -name → --name
                        let double = format!("-{}", s);
                        return OsString::from(double);
                    }
                }
            }
            arg
        })
        .collect()
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    // Normalize single-dash long flags to double-dash for clap compatibility.
    let normalized = normalize_find_args(args);

    let matches = gow_core::args::parse_gnu(Cli::command(), normalized);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("find: {}", e);
            return 2;
        }
    };

    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("find: {}", e);
            2
        }
    }
}

fn run(mut cli: Cli) -> Result<i32> {
    // Default search path to "." if none provided
    if cli.paths.is_empty() {
        cli.paths.push(PathBuf::from("."));
    }

    // Set stdout to binary mode when -print0 is active (must be done before any writes)
    if cli.print0 {
        set_stdout_binary_mode();
    }

    // Build glob matchers — if both -name and -iname are given, -iname wins (GNU behavior)
    let name_matcher: Option<GlobMatcher> = if let Some(ref pat) = cli.iname {
        Some(build_name_matcher(pat, true)?)
    } else if let Some(ref pat) = cli.name {
        Some(build_name_matcher(pat, false)?)
    } else {
        None
    };

    // Parse size predicate
    let size_pred: Option<(CmpOp, u64)> = cli.size.as_deref()
        .map(parse_size_spec)
        .transpose()?;

    // Parse time predicates
    let mtime_pred: Option<(CmpOp, u64)> = cli.mtime.as_deref()
        .map(parse_time_spec)
        .transpose()?;
    let atime_pred: Option<(CmpOp, u64)> = cli.atime.as_deref()
        .map(parse_time_spec)
        .transpose()?;
    let ctime_pred: Option<(CmpOp, u64)> = cli.ctime.as_deref()
        .map(parse_time_spec)
        .transpose()?;

    // Resolve "now" once for all comparisons
    let now = SystemTime::now();

    let min_depth = cli.mindepth.unwrap_or(0);
    let max_depth = cli.maxdepth.unwrap_or(usize::MAX);

    let mut any_exec_failed = false;

    for root in &cli.paths {
        // Verify path exists
        if !root.exists() {
            let err = std::io::Error::last_os_error();
            eprintln!("find: {}: {}", root.display(), err);
            continue;
        }

        let walker = WalkDir::new(root)
            .min_depth(min_depth)
            .max_depth(max_depth)
            .follow_links(false);

        for entry_result in walker.into_iter() {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("find: {}", e);
                    continue;
                }
            };

            // Run predicate chain — all predicates AND together
            if !matches_predicates(
                &entry,
                &name_matcher,
                cli.type_.as_deref(),
                size_pred,
                mtime_pred,
                atime_pred,
                ctime_pred,
                now,
            ) {
                continue;
            }

            // Perform action
            let path = entry.path();
            if let Some(ref cmd_parts) = cli.exec {
                match exec_for_entry(cmd_parts, path) {
                    Ok(code) if code != 0 => {
                        eprintln!("find: -exec failed (exit {})", code);
                        any_exec_failed = true;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("find: {}", e);
                        any_exec_failed = true;
                    }
                }
            } else if cli.print0 {
                // Write path bytes followed by NUL, no newline
                let path_bytes = path.as_os_str().to_string_lossy();
                let mut stdout = std::io::stdout().lock();
                stdout.write_all(path_bytes.as_bytes()).ok();
                stdout.write_all(b"\0").ok();
            } else {
                // Default action: -print
                println!("{}", path.display());
            }
        }
    }

    Ok(if any_exec_failed { 1 } else { 0 })
}

/// Evaluate all active predicates against a directory entry.
/// Returns true only if every active predicate matches.
#[allow(clippy::too_many_arguments)]
fn matches_predicates(
    entry: &DirEntry,
    name_matcher: &Option<GlobMatcher>,
    type_filter: Option<&str>,
    size_pred: Option<(CmpOp, u64)>,
    mtime_pred: Option<(CmpOp, u64)>,
    atime_pred: Option<(CmpOp, u64)>,
    ctime_pred: Option<(CmpOp, u64)>,
    now: SystemTime,
) -> bool {
    // -name / -iname: match basename only (GNU find semantics — NOT full path)
    if let Some(matcher) = name_matcher {
        if !match_glob_basename(matcher, entry.file_name()) {
            return false;
        }
    }

    // -type f|d|l
    if let Some(t) = type_filter {
        if !matches_type(entry, t) {
            return false;
        }
    }

    // -size: requires metadata
    if let Some((op, target)) = size_pred {
        match entry.metadata() {
            Ok(meta) => {
                if !cmp_size(op, meta.len(), target) {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }

    // -mtime / -atime / -ctime: requires metadata
    if mtime_pred.is_some() || atime_pred.is_some() || ctime_pred.is_some() {
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => return false,
        };

        if let Some((op, target)) = mtime_pred {
            match meta.modified() {
                Ok(t) => {
                    let days = days_since(now, t);
                    if !cmp_time(op, days, target) {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }

        if let Some((op, target)) = atime_pred {
            match meta.accessed() {
                Ok(t) => {
                    // D-04: atime may equal mtime on Windows NTFS when last-access
                    // tracking is disabled (NtfsDisableLastAccessUpdate registry key).
                    let days = days_since(now, t);
                    if !cmp_time(op, days, target) {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }

        if let Some((op, target)) = ctime_pred {
            match meta.created() {
                Ok(t) => {
                    // Windows: created() maps to ftCreationTime (not Unix inode ctime).
                    let days = days_since(now, t);
                    if !cmp_time(op, days, target) {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }
    }

    true
}

// ─── Helper functions ────────────────────────────────────────────────────────

fn build_name_matcher(pattern: &str, case_insensitive: bool) -> Result<GlobMatcher> {
    let glob = GlobBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .literal_separator(false) // POSIX find: * matches across separators in basename
        .build()
        .map_err(|e| anyhow!("invalid glob '{}': {}", pattern, e))?;
    Ok(glob.compile_matcher())
}

fn match_glob_basename(matcher: &GlobMatcher, name: &OsStr) -> bool {
    matcher.is_match(name)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CmpOp {
    Greater,
    Less,
    Equal,
}

/// Parse a size spec like `+10k`, `-1M`, `5`, `100c`.
///
/// Supported suffixes: c (bytes, default), k (1024), M (1024^2), G (1024^3).
/// Prefix `+` = Greater, `-` = Less, none = Equal.
fn parse_size_spec(spec: &str) -> Result<(CmpOp, u64)> {
    let (op, rest) = match spec.as_bytes().first() {
        Some(b'+') => (CmpOp::Greater, &spec[1..]),
        Some(b'-') => (CmpOp::Less, &spec[1..]),
        _ => (CmpOp::Equal, spec),
    };
    // Trailing unit char: c (bytes, default), k (1024), M (1024^2), G (1024^3)
    let (num_str, mult): (&str, u64) = match rest.chars().last() {
        Some('c') => (&rest[..rest.len() - 1], 1),
        Some('k') => (&rest[..rest.len() - 1], 1024),
        Some('M') => (&rest[..rest.len() - 1], 1024 * 1024),
        Some('G') => (&rest[..rest.len() - 1], 1024 * 1024 * 1024),
        _ => (rest, 1),
    };
    let n: u64 = num_str
        .parse()
        .map_err(|e| anyhow!("invalid -size '{}': {}", spec, e))?;
    Ok((op, n.saturating_mul(mult)))
}

/// Parse a time spec like `+7`, `-1`, `0`.
///
/// Value is in days. Prefix `+` = Greater, `-` = Less, none = Equal.
fn parse_time_spec(spec: &str) -> Result<(CmpOp, u64)> {
    let (op, rest) = match spec.as_bytes().first() {
        Some(b'+') => (CmpOp::Greater, &spec[1..]),
        Some(b'-') => (CmpOp::Less, &spec[1..]),
        _ => (CmpOp::Equal, spec),
    };
    let n: u64 = rest
        .parse()
        .map_err(|e| anyhow!("invalid time spec '{}': {}", spec, e))?;
    Ok((op, n))
}

fn cmp_size(op: CmpOp, actual: u64, target: u64) -> bool {
    match op {
        CmpOp::Greater => actual > target,
        CmpOp::Less => actual < target,
        CmpOp::Equal => actual == target,
    }
}

/// Compute the number of whole days since timestamp `t` relative to `now`.
///
/// Uses floor((now - t) / 86400) — matches GNU find -mtime semantics.
fn days_since(now: SystemTime, t: SystemTime) -> u64 {
    let now_s = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let t_s = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    now_s.saturating_sub(t_s) / 86400
}

/// Compare actual_days against target_days using the given operator.
///
/// +N → actual > N, -N → actual < N, N → actual == N (exact day match).
fn cmp_time(op: CmpOp, actual_days: u64, target_days: u64) -> bool {
    match op {
        CmpOp::Greater => actual_days > target_days,
        CmpOp::Less => actual_days < target_days,
        CmpOp::Equal => actual_days == target_days,
    }
}

/// Match entry type against the -type flag character: f, d, l.
fn matches_type(entry: &DirEntry, type_char: &str) -> bool {
    let ft = entry.file_type();
    match type_char {
        "f" => ft.is_file(),
        "d" => ft.is_dir(),
        "l" => ft.is_symlink(),
        _ => false, // unknown type: never matches
    }
}

/// Execute a command once for a matched entry (the -exec action).
///
/// Substitutes every literal `{}` token in cmd_parts[1..] with the matched path.
/// Invokes via std::process::Command (CreateProcessW on Windows) — no shell intermediary.
/// This fixes GOW #209 (paths with spaces): each argument is a separate Win32 string,
/// never concatenated into a shell command line.
fn exec_for_entry(cmd_parts: &[String], path: &Path) -> Result<i32> {
    if cmd_parts.is_empty() {
        return Err(anyhow!("-exec requires a command"));
    }
    let path_str = path.to_string_lossy().into_owned();
    let args: Vec<String> = cmd_parts[1..]
        .iter()
        .map(|a| {
            if a == "{}" {
                path_str.clone()
            } else {
                a.clone()
            }
        })
        .collect();
    let status = Command::new(&cmd_parts[0])
        .args(&args)
        .status()
        .map_err(|e| anyhow!("-exec failed to spawn '{}': {}", cmd_parts[0], e))?;
    Ok(status.code().unwrap_or(1))
}

/// Set stdout to binary mode on Windows to prevent CRT text-mode translation.
///
/// Required for -print0: text mode translates 0x0A → 0x0D 0x0A and treats
/// 0x1A as EOF, corrupting the null-byte stream. Uses `_setmode` (cdecl CRT
/// function, NOT a Win32 API — must use extern "C", not extern "system").
/// Per RESEARCH.md Pattern 3 and T-05-find-05 mitigation.
#[cfg(target_os = "windows")]
fn set_stdout_binary_mode() {
    unsafe extern "C" {
        fn _setmode(fd: i32, flags: i32) -> i32;
    }
    const _O_BINARY: i32 = 0x8000;
    unsafe {
        _setmode(1, _O_BINARY); // 1 = stdout
    }
}

#[cfg(not(target_os = "windows"))]
fn set_stdout_binary_mode() {
    // No-op on Unix/Linux — stdout is already binary-clean
}

// ─── Inline unit tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_plus_k() {
        assert_eq!(parse_size_spec("+10k").unwrap(), (CmpOp::Greater, 10240));
    }

    #[test]
    fn test_parse_size_minus_m() {
        assert_eq!(parse_size_spec("-1M").unwrap(), (CmpOp::Less, 1048576));
    }

    #[test]
    fn test_parse_size_equal_bytes() {
        assert_eq!(parse_size_spec("5").unwrap(), (CmpOp::Equal, 5));
    }

    #[test]
    fn test_parse_size_explicit_c() {
        assert_eq!(parse_size_spec("100c").unwrap(), (CmpOp::Equal, 100));
    }

    #[test]
    fn test_parse_time_plus() {
        assert_eq!(parse_time_spec("+7").unwrap(), (CmpOp::Greater, 7));
    }

    #[test]
    fn test_parse_time_minus() {
        assert_eq!(parse_time_spec("-1").unwrap(), (CmpOp::Less, 1));
    }

    #[test]
    fn test_parse_time_equal() {
        assert_eq!(parse_time_spec("0").unwrap(), (CmpOp::Equal, 0));
    }

    #[test]
    fn test_glob_basename_match() {
        let m = build_name_matcher("*.txt", false).unwrap();
        assert!(match_glob_basename(&m, OsStr::new("foo.txt")));
        assert!(!match_glob_basename(&m, OsStr::new("foo.rs")));
    }

    #[test]
    fn test_glob_iname_case_insensitive() {
        let m = build_name_matcher("*.TXT", true).unwrap();
        assert!(match_glob_basename(&m, OsStr::new("foo.txt")));
        assert!(match_glob_basename(&m, OsStr::new("FOO.TXT")));
    }

    #[test]
    fn test_glob_name_case_sensitive() {
        let m = build_name_matcher("*.TXT", false).unwrap();
        assert!(!match_glob_basename(&m, OsStr::new("foo.txt")));
        assert!(match_glob_basename(&m, OsStr::new("FOO.TXT")));
    }

    #[test]
    fn test_cmp_size_ops() {
        assert!(cmp_size(CmpOp::Greater, 100, 50));
        assert!(cmp_size(CmpOp::Less, 30, 50));
        assert!(cmp_size(CmpOp::Equal, 50, 50));
        assert!(!cmp_size(CmpOp::Greater, 50, 50));
    }

    #[test]
    fn test_cmp_time_ops() {
        assert!(cmp_time(CmpOp::Greater, 10, 5));
        assert!(cmp_time(CmpOp::Less, 2, 5));
        assert!(cmp_time(CmpOp::Equal, 7, 7));
        assert!(!cmp_time(CmpOp::Less, 7, 7));
    }

    #[test]
    fn test_normalize_find_args_single_dash() {
        let args: Vec<OsString> = vec![
            OsString::from("find"),
            OsString::from("."),
            OsString::from("-name"),
            OsString::from("*.txt"),
            OsString::from("-type"),
            OsString::from("f"),
        ];
        let normalized = normalize_find_args(args);
        assert_eq!(normalized[2], OsString::from("--name"));
        assert_eq!(normalized[4], OsString::from("--type"));
        // Path "." should be unchanged
        assert_eq!(normalized[1], OsString::from("."));
    }

    #[test]
    fn test_normalize_find_args_exec_semicolon_unchanged() {
        // The literal ";" terminator must not be mangled
        let args: Vec<OsString> = vec![
            OsString::from("find"),
            OsString::from("-exec"),
            OsString::from("echo"),
            OsString::from("{}"),
            OsString::from(";"),
        ];
        let normalized = normalize_find_args(args);
        assert_eq!(normalized[1], OsString::from("--exec"));
        assert_eq!(normalized[4], OsString::from(";"));
    }

    #[test]
    fn test_parse_size_g() {
        assert_eq!(
            parse_size_spec("+2G").unwrap(),
            (CmpOp::Greater, 2 * 1024 * 1024 * 1024)
        );
    }
}
