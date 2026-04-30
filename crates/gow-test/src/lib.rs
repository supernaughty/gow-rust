use std::ffi::OsString;
use std::fs;
use std::path::Path;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();

    let mut expr_args: Vec<String> = args_vec[1..]
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    // Bracket mode detection: extras/bin/[.bat calls test.exe --_bracket_ <args> ]
    // The shim inserts --_bracket_ as the first argument so test.exe knows it was invoked as [
    let bracket_mode = expr_args.first().map(|s| s == "--_bracket_").unwrap_or(false);
    if bracket_mode {
        expr_args.remove(0); // strip the --_bracket_ sentinel
        // In bracket mode, require and strip the trailing ] argument
        match expr_args.last().map(String::as_str) {
            Some("]") => {
                expr_args.pop();
            }
            _ => {
                eprintln!("[: missing ']'");
                return 2;
            }
        }
    }

    evaluate_test(&expr_args)
}

/// Evaluate a test expression given as a slice of string arguments.
/// Returns exit code: 0 = true, 1 = false, 2 = usage/syntax error.
pub fn evaluate_test(args: &[String]) -> i32 {
    if args.is_empty() {
        return 1; // empty test → false (NOT error, per POSIX/GNU)
    }
    let mut pos = 0usize;
    match parse_expr(args, &mut pos) {
        Ok(result) => {
            if pos != args.len() {
                eprintln!("test: too many arguments");
                2
            } else if result {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("test: {}", e);
            2
        }
    }
}

// --- Recursive descent parser ---
// Precedence (lowest to highest): -o, -a, !, primary

fn parse_expr(args: &[String], pos: &mut usize) -> Result<bool, String> {
    let lhs = parse_and_expr(args, pos)?;
    if *pos < args.len() && args[*pos] == "-o" {
        *pos += 1;
        let rhs = parse_expr(args, pos)?;
        return Ok(lhs || rhs);
    }
    Ok(lhs)
}

fn parse_and_expr(args: &[String], pos: &mut usize) -> Result<bool, String> {
    let lhs = parse_not_expr(args, pos)?;
    if *pos < args.len() && args[*pos] == "-a" {
        *pos += 1;
        let rhs = parse_and_expr(args, pos)?;
        return Ok(lhs && rhs);
    }
    Ok(lhs)
}

fn parse_not_expr(args: &[String], pos: &mut usize) -> Result<bool, String> {
    if *pos < args.len() && args[*pos] == "!" {
        *pos += 1;
        let inner = parse_not_expr(args, pos)?; // right-associative
        return Ok(!inner);
    }
    parse_primary(args, pos)
}

fn parse_primary(args: &[String], pos: &mut usize) -> Result<bool, String> {
    if *pos >= args.len() {
        return Err("missing argument".to_string());
    }

    // Parenthesized expression: ( EXPR )
    if args[*pos] == "(" {
        *pos += 1;
        let inner = parse_expr(args, pos)?;
        if *pos >= args.len() || args[*pos] != ")" {
            return Err("missing ')'".to_string());
        }
        *pos += 1;
        return Ok(inner);
    }

    let tok = &args[*pos];

    // File predicates: -f -d -e -r -w -x -s -L
    if matches!(
        tok.as_str(),
        "-f" | "-d" | "-e" | "-r" | "-w" | "-x" | "-s" | "-L"
    ) {
        let op = tok.clone();
        *pos += 1;
        if *pos >= args.len() {
            return Err(format!("{}: missing file argument", op));
        }
        let path = &args[*pos];
        *pos += 1;
        return Ok(file_test(&op, path));
    }

    // String unary: -z STRING (zero length → true), -n STRING (non-zero → true)
    if tok == "-z" {
        *pos += 1;
        if *pos >= args.len() {
            return Err("-z: missing argument".to_string());
        }
        let s = &args[*pos];
        *pos += 1;
        return Ok(s.is_empty());
    }
    if tok == "-n" {
        *pos += 1;
        if *pos >= args.len() {
            return Err("-n: missing argument".to_string());
        }
        let s = &args[*pos];
        *pos += 1;
        return Ok(!s.is_empty());
    }

    // Binary expressions: these require at least pos, pos+1, pos+2
    // Check for 3-token pattern: ARG1 OP ARG2
    if *pos + 2 <= args.len() {
        // Need at least 3 tokens starting at pos
        let op = args.get(*pos + 1).map(String::as_str).unwrap_or("");
        match op {
            // String binary comparisons
            "=" | "!=" | "<" | ">" => {
                let a = args[*pos].clone();
                let b = args[*pos + 2].clone();
                *pos += 3;
                return Ok(match op {
                    "=" => a == b,
                    "!=" => a != b,
                    "<" => a < b,
                    ">" => a > b,
                    _ => unreachable!(),
                });
            }
            // Integer binary comparisons
            "-eq" | "-ne" | "-lt" | "-le" | "-gt" | "-ge" => {
                let ai = args[*pos]
                    .parse::<i64>()
                    .map_err(|_| format!("integer expression expected: '{}'", args[*pos]))?;
                let bi = args[*pos + 2]
                    .parse::<i64>()
                    .map_err(|_| format!("integer expression expected: '{}'", args[*pos + 2]))?;
                *pos += 3;
                return Ok(match op {
                    "-eq" => ai == bi,
                    "-ne" => ai != bi,
                    "-lt" => ai < bi,
                    "-le" => ai <= bi,
                    "-gt" => ai > bi,
                    "-ge" => ai >= bi,
                    _ => unreachable!(),
                });
            }
            _ => {}
        }
    }

    // Single string argument: non-empty string = true, empty string = false
    let s = args[*pos].clone();
    *pos += 1;
    Ok(!s.is_empty())
}

/// Test file predicates using std::fs::metadata.
/// Converts MSYS/Unix-style paths to Windows paths before stat.
fn file_test(op: &str, path: &str) -> bool {
    let converted = gow_core::path::try_convert_msys_path(path);
    match op {
        "-e" => fs::metadata(&converted).is_ok(),
        "-f" => fs::metadata(&converted)
            .map(|m| m.is_file())
            .unwrap_or(false),
        "-d" => fs::metadata(&converted)
            .map(|m| m.is_dir())
            .unwrap_or(false),
        "-s" => fs::metadata(&converted)
            .map(|m| m.len() > 0)
            .unwrap_or(false),
        "-r" => fs::metadata(&converted).is_ok(), // Windows: readable if accessible
        "-w" => fs::metadata(&converted)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false),
        "-L" => fs::symlink_metadata(&converted)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false),
        "-x" => {
            // Windows: executable = file exists with .exe/.bat/.com/.cmd extension
            let p = Path::new(&converted);
            p.exists()
                && matches!(
                    p.extension().and_then(|e| e.to_str()),
                    Some("exe") | Some("bat") | Some("com") | Some("cmd")
                )
        }
        _ => false,
    }
}
