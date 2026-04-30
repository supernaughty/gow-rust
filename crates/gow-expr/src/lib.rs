//! `uu_expr`: GNU `expr` — recursive-descent arithmetic and string expression evaluator.
//!
//! Does NOT use clap. Raw argv parsing.
//!
//! EXIT CODE SEMANTICS (inverted from test/[):
//!   exit 0 = non-null result (non-empty AND not "0")
//!   exit 1 = null result (empty string OR exactly "0")
//!   exit 2 = syntax error

use std::ffi::OsString;
use regex::Regex;

/// Maximum recursion depth to prevent stack overflow (T-11-04-01 DoS mitigation).
const MAX_DEPTH: usize = 100;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<String> = args
        .into_iter()
        .skip(1) // skip argv[0]
        .map(|s| s.to_string_lossy().to_string())
        .collect();

    if args_vec.is_empty() {
        eprintln!("expr: missing operand");
        return 2;
    }

    match evaluate(&args_vec) {
        Ok(result) => {
            println!("{}", result);
            // INVERTED semantics: 0 = non-null (truthy), 1 = null (falsy), 2 = error
            let is_null = result.is_empty() || result == "0";
            if is_null { 1 } else { 0 }
        }
        Err(e) => {
            eprintln!("expr: {}", e);
            2
        }
    }
}

/// Evaluate an expression given as a slice of argument tokens.
pub fn evaluate(args: &[String]) -> Result<String, String> {
    let mut pos = 0usize;
    let result = parse_or(args, &mut pos, 0)?;
    if pos != args.len() {
        return Err(format!("extra operand '{}'", args[pos]));
    }
    Ok(result)
}

// ─── Helper ────────────────────────────────────────────────────────────────

fn is_null(s: &str) -> bool {
    s.is_empty() || s == "0"
}

fn to_int(s: &str, op: &str) -> Result<i64, String> {
    s.parse::<i64>()
        .map_err(|_| format!("non-integer argument for {}: '{}'", op, s))
}

fn cmp_values(a: &str, b: &str, op: &str) -> String {
    // Try numeric comparison first
    if let (Ok(ai), Ok(bi)) = (a.parse::<i64>(), b.parse::<i64>()) {
        let result = match op {
            "=" => ai == bi,
            "!=" => ai != bi,
            "<" => ai < bi,
            ">" => ai > bi,
            "<=" => ai <= bi,
            ">=" => ai >= bi,
            _ => false,
        };
        return if result { "1".to_string() } else { "0".to_string() };
    }
    // Fallback: string comparison
    let result = match op {
        "=" => a == b,
        "!=" => a != b,
        "<" => a < b,
        ">" => a > b,
        "<=" => a <= b,
        ">=" => a >= b,
        _ => false,
    };
    if result { "1".to_string() } else { "0".to_string() }
}

// ─── Recursive-descent parser (lowest precedence first) ────────────────────

/// Level 1: | (or) — lowest precedence
fn parse_or(args: &[String], pos: &mut usize, depth: usize) -> Result<String, String> {
    if depth > MAX_DEPTH {
        return Err("expression too deeply nested".to_string());
    }
    let mut lhs = parse_and(args, pos, depth + 1)?;
    while *pos < args.len() && args[*pos] == "|" {
        *pos += 1;
        let rhs = parse_and(args, pos, depth + 1)?;
        // | semantics: return lhs if lhs is non-null, else rhs
        lhs = if !is_null(&lhs) { lhs } else { rhs };
    }
    Ok(lhs)
}

/// Level 2: & (and)
fn parse_and(args: &[String], pos: &mut usize, depth: usize) -> Result<String, String> {
    if depth > MAX_DEPTH {
        return Err("expression too deeply nested".to_string());
    }
    let mut lhs = parse_comparison(args, pos, depth + 1)?;
    while *pos < args.len() && args[*pos] == "&" {
        *pos += 1;
        let rhs = parse_comparison(args, pos, depth + 1)?;
        // & semantics: return lhs if both non-null, else "0"
        lhs = if !is_null(&lhs) && !is_null(&rhs) {
            lhs
        } else {
            "0".to_string()
        };
    }
    Ok(lhs)
}

/// Level 3: = != < > <= >= (comparison)
fn parse_comparison(args: &[String], pos: &mut usize, depth: usize) -> Result<String, String> {
    if depth > MAX_DEPTH {
        return Err("expression too deeply nested".to_string());
    }
    let mut lhs = parse_additive(args, pos, depth + 1)?;
    while *pos < args.len() {
        let op = args[*pos].as_str();
        if !matches!(op, "=" | "!=" | "<" | ">" | "<=" | ">=") {
            break;
        }
        *pos += 1;
        let rhs = parse_additive(args, pos, depth + 1)?;
        lhs = cmp_values(&lhs, &rhs, op);
    }
    Ok(lhs)
}

/// Level 4: + - (additive)
fn parse_additive(args: &[String], pos: &mut usize, depth: usize) -> Result<String, String> {
    if depth > MAX_DEPTH {
        return Err("expression too deeply nested".to_string());
    }
    let mut lhs = parse_multiplicative(args, pos, depth + 1)?;
    while *pos < args.len() {
        let op = args[*pos].as_str();
        if !matches!(op, "+" | "-") {
            break;
        }
        *pos += 1;
        let rhs = parse_multiplicative(args, pos, depth + 1)?;
        let a = to_int(&lhs, op)?;
        let b = to_int(&rhs, op)?;
        let result = match op {
            "+" => a.checked_add(b).ok_or_else(|| "integer overflow".to_string())?,
            "-" => a.checked_sub(b).ok_or_else(|| "integer overflow".to_string())?,
            _ => unreachable!(),
        };
        lhs = result.to_string();
    }
    Ok(lhs)
}

/// Level 5: * / % (multiplicative)
fn parse_multiplicative(args: &[String], pos: &mut usize, depth: usize) -> Result<String, String> {
    if depth > MAX_DEPTH {
        return Err("expression too deeply nested".to_string());
    }
    let mut lhs = parse_colon(args, pos, depth + 1)?;
    while *pos < args.len() {
        let op = args[*pos].as_str();
        if !matches!(op, "*" | "/" | "%") {
            break;
        }
        *pos += 1;
        let rhs = parse_colon(args, pos, depth + 1)?;
        let a = to_int(&lhs, op)?;
        let b = to_int(&rhs, op)?;
        let result = match op {
            "*" => a.checked_mul(b).ok_or_else(|| "integer overflow".to_string())?,
            "/" => {
                if b == 0 {
                    return Err("division by zero".to_string());
                }
                a.checked_div(b).ok_or_else(|| "integer overflow".to_string())?
            }
            "%" => {
                if b == 0 {
                    return Err("division by zero".to_string());
                }
                a.checked_rem(b).ok_or_else(|| "integer overflow".to_string())?
            }
            _ => unreachable!(),
        };
        lhs = result.to_string();
    }
    Ok(lhs)
}

/// Level 6: : (regex match)
fn parse_colon(args: &[String], pos: &mut usize, depth: usize) -> Result<String, String> {
    if depth > MAX_DEPTH {
        return Err("expression too deeply nested".to_string());
    }
    let lhs = parse_atom(args, pos, depth + 1)?;
    if *pos < args.len() && args[*pos] == ":" {
        *pos += 1;
        if *pos >= args.len() {
            return Err("missing regexp after ':'".to_string());
        }
        let pattern = args[*pos].as_str();
        *pos += 1;
        // GNU expr anchors the regex at the start of the string implicitly.
        let anchored = format!("^(?:{})", pattern);
        // T-11-04-03: use `regex` crate which guarantees linear-time matching.
        let re = Regex::new(&anchored).map_err(|e| format!("invalid regexp: {e}"))?;
        if let Some(caps) = re.captures(&lhs) {
            if caps.len() > 1 {
                // Has capturing group — return the group match (or empty string if unmatched)
                return Ok(caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default());
            }
            // No capturing group — return match length in characters (not bytes),
            // consistent with GNU expr and with the `length` keyword below.
            return Ok(caps.get(0).map(|m| m.as_str().chars().count()).unwrap_or(0).to_string());
        }
        return Ok("0".to_string());
    }
    Ok(lhs)
}

/// Level 7: atom — literals, parentheses, string functions
fn parse_atom(args: &[String], pos: &mut usize, depth: usize) -> Result<String, String> {
    if depth > MAX_DEPTH {
        return Err("expression too deeply nested".to_string());
    }
    if *pos >= args.len() {
        return Err("missing operand".to_string());
    }

    // Parentheses
    if args[*pos] == "(" {
        *pos += 1;
        let inner = parse_or(args, pos, depth + 1)?;
        if *pos >= args.len() || args[*pos] != ")" {
            return Err("unmatched '('".to_string());
        }
        *pos += 1;
        return Ok(inner);
    }

    // String functions: length, substr, index, match
    match args[*pos].as_str() {
        "length" => {
            *pos += 1;
            let s = parse_atom(args, pos, depth + 1)?;
            return Ok(s.chars().count().to_string());
        }
        "substr" => {
            *pos += 1;
            let s = parse_atom(args, pos, depth + 1)?;
            let p = parse_atom(args, pos, depth + 1)?.parse::<usize>().unwrap_or(1).max(1);
            let n = parse_atom(args, pos, depth + 1)?.parse::<usize>().unwrap_or(0);
            let chars: Vec<char> = s.chars().collect();
            let start = (p - 1).min(chars.len());
            let end = (start + n).min(chars.len());
            return Ok(chars[start..end].iter().collect());
        }
        "index" => {
            *pos += 1;
            let s = parse_atom(args, pos, depth + 1)?;
            let chars_arg = parse_atom(args, pos, depth + 1)?;
            // Find first occurrence of any char in chars_arg in s (1-based position)
            let result = s
                .char_indices()
                .find(|(_, c)| chars_arg.contains(*c))
                .map(|(i, _)| {
                    // Convert byte index to character position (1-based)
                    s[..i].chars().count() + 1
                })
                .unwrap_or(0);
            return Ok(result.to_string());
        }
        "match" => {
            *pos += 1;
            let s = parse_atom(args, pos, depth + 1)?;
            let pattern = parse_atom(args, pos, depth + 1)?;
            let anchored = format!("^(?:{})", pattern);
            let re = Regex::new(&anchored).map_err(|e| format!("invalid regexp: {e}"))?;
            let result = if let Some(m) = re.find(&s) { m.as_str().chars().count() } else { 0 };
            return Ok(result.to_string());
        }
        _ => {}
    }

    // Literal token
    let token = args[*pos].clone();
    *pos += 1;
    Ok(token)
}
