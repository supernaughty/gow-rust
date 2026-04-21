//! GNU `env -S STRING` split-string parser.
//!
//! State machine with 4 top states (delimiter, unquoted, single_quoted,
//! double_quoted) plus inline backslash handling. Supports `${VAR}` expansion
//! via a caller-provided closure (no direct env access — keeps the function
//! pure and unit-testable).
//!
//! Reference: RESEARCH.md Q7 lines 590-672 and CONTEXT.md D-19a. Security: input
//! bounded to MAX_INPUT_LEN (10 KiB) to prevent DoS via giant `-S` strings
//! (threat T-02-09-02); no recursion; 1-pass expansion (expanded values are
//! emitted verbatim and NOT re-parsed, per Pitfall 6).

use std::fmt;

pub(crate) const MAX_INPUT_LEN: usize = 10 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum SplitError {
    MissingBrace { at: usize },
    UnterminatedQuote { quote: char },
    InputTooLong,
}

impl fmt::Display for SplitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SplitError::MissingBrace { at } => write!(
                f,
                "only ${{VARNAME}} expansion is supported (missing brace at byte {at})"
            ),
            SplitError::UnterminatedQuote { quote } => {
                write!(f, "unterminated quote: {quote}")
            }
            SplitError::InputTooLong => {
                write!(f, "input too long (> {MAX_INPUT_LEN} bytes)")
            }
        }
    }
}

impl std::error::Error for SplitError {}

/// Tokenize a GNU env -S string. `env_lookup` resolves `${VAR}` expansions.
///
/// The parser walks `input` byte-by-byte through four states:
/// - `Delim`  — between tokens: whitespace, comments, and token-start sentinels
/// - `Unquoted` — inside a bare token: escapes and `${VAR}` expand, whitespace ends it
/// - `Single` — inside `'…'`: literal except `\\` and `\'`
/// - `Double` — inside `"…"`: escapes and `${VAR}` expand, no `\c`
///
/// Returns `SplitError::MissingBrace` when `$` is not followed by `{NAME}`
/// (GNU requires curly braces), `SplitError::UnterminatedQuote` when a quote
/// is never closed, and `SplitError::InputTooLong` on inputs > 10 KiB.
pub fn split<F: Fn(&str) -> Option<String>>(
    input: &str,
    env_lookup: F,
) -> Result<Vec<String>, SplitError> {
    if input.len() > MAX_INPUT_LEN {
        return Err(SplitError::InputTooLong);
    }

    #[derive(Clone, Copy)]
    enum S {
        Delim,
        Unquoted,
        Single,
        Double,
    }

    let bytes = input.as_bytes();
    let mut tokens: Vec<String> = Vec::new();
    let mut cur = String::new();
    // `has_token` is true once we've committed to emitting a token (possibly empty,
    // e.g. from an unset ${MISSING}). `cur.is_empty()` alone cannot distinguish
    // "no token yet" from "empty token in progress".
    let mut has_token = false;
    let mut state = S::Delim;
    let mut i: usize = 0;

    while i < bytes.len() {
        let b = bytes[i];
        match state {
            S::Delim => match b {
                b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C => {
                    i += 1;
                }
                b'#' => {
                    // Comment: skip until newline or EOS.
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                }
                b'\\' => {
                    // `\c` at a delimiter position ends parsing with no extra token.
                    if i + 1 < bytes.len() && bytes[i + 1] == b'c' {
                        return Ok(tokens);
                    }
                    // Otherwise: start a new unquoted token and reprocess at same index.
                    state = S::Unquoted;
                    has_token = true;
                }
                b'\'' => {
                    state = S::Single;
                    has_token = true;
                    i += 1;
                }
                b'"' => {
                    state = S::Double;
                    has_token = true;
                    i += 1;
                }
                b'$' => {
                    i = expand_var(bytes, i, &mut cur, &env_lookup)?;
                    state = S::Unquoted;
                    has_token = true;
                }
                _ => {
                    cur.push(b as char);
                    has_token = true;
                    state = S::Unquoted;
                    i += 1;
                }
            },
            S::Unquoted => match b {
                b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C => {
                    tokens.push(std::mem::take(&mut cur));
                    has_token = false;
                    state = S::Delim;
                    i += 1;
                }
                b'\'' => {
                    state = S::Single;
                    i += 1;
                }
                b'"' => {
                    state = S::Double;
                    i += 1;
                }
                b'\\' => {
                    if i + 1 >= bytes.len() {
                        // Trailing backslash: emit literally.
                        cur.push('\\');
                        i += 1;
                        continue;
                    }
                    let n = bytes[i + 1];
                    match n {
                        b'c' => {
                            // \c ends parsing. Flush current token first.
                            if has_token {
                                tokens.push(std::mem::take(&mut cur));
                            }
                            return Ok(tokens);
                        }
                        b'_' => {
                            // Token delimiter when outside quotes.
                            tokens.push(std::mem::take(&mut cur));
                            has_token = false;
                            state = S::Delim;
                            i += 2;
                        }
                        b'f' => {
                            cur.push('\x0C');
                            i += 2;
                        }
                        b'n' => {
                            cur.push('\n');
                            i += 2;
                        }
                        b'r' => {
                            cur.push('\r');
                            i += 2;
                        }
                        b't' => {
                            cur.push('\t');
                            i += 2;
                        }
                        b'v' => {
                            cur.push('\x0B');
                            i += 2;
                        }
                        b'#' => {
                            cur.push('#');
                            i += 2;
                        }
                        b'$' => {
                            cur.push('$');
                            i += 2;
                        }
                        b'\\' => {
                            cur.push('\\');
                            i += 2;
                        }
                        b'"' => {
                            cur.push('"');
                            i += 2;
                        }
                        b'\'' => {
                            cur.push('\'');
                            i += 2;
                        }
                        _ => {
                            // Unknown escape: preserve backslash + char verbatim.
                            cur.push('\\');
                            cur.push(n as char);
                            i += 2;
                        }
                    }
                }
                b'#' => {
                    // Inside a token, `#` is a literal character, not a comment.
                    cur.push('#');
                    i += 1;
                }
                b'$' => {
                    i = expand_var(bytes, i, &mut cur, &env_lookup)?;
                }
                _ => {
                    cur.push(b as char);
                    i += 1;
                }
            },
            S::Single => {
                // Single quotes: almost everything is literal. Only \\ and \' escape.
                match b {
                    b'\'' => {
                        state = S::Unquoted;
                        i += 1;
                    }
                    b'\\' if i + 1 < bytes.len() => {
                        let n = bytes[i + 1];
                        if n == b'\\' || n == b'\'' {
                            cur.push(n as char);
                            i += 2;
                        } else {
                            cur.push('\\');
                            i += 1;
                        }
                    }
                    _ => {
                        cur.push(b as char);
                        i += 1;
                    }
                }
            }
            S::Double => match b {
                b'"' => {
                    state = S::Unquoted;
                    i += 1;
                }
                b'\\' => {
                    if i + 1 >= bytes.len() {
                        cur.push('\\');
                        i += 1;
                        continue;
                    }
                    let n = bytes[i + 1];
                    match n {
                        b'_' => {
                            // Inside double quotes \_ means a literal space.
                            cur.push(' ');
                            i += 2;
                        }
                        b'f' => {
                            cur.push('\x0C');
                            i += 2;
                        }
                        b'n' => {
                            cur.push('\n');
                            i += 2;
                        }
                        b'r' => {
                            cur.push('\r');
                            i += 2;
                        }
                        b't' => {
                            cur.push('\t');
                            i += 2;
                        }
                        b'v' => {
                            cur.push('\x0B');
                            i += 2;
                        }
                        b'#' => {
                            cur.push('#');
                            i += 2;
                        }
                        b'$' => {
                            cur.push('$');
                            i += 2;
                        }
                        b'\\' => {
                            cur.push('\\');
                            i += 2;
                        }
                        b'"' => {
                            cur.push('"');
                            i += 2;
                        }
                        _ => {
                            cur.push('\\');
                            cur.push(n as char);
                            i += 2;
                        }
                    }
                }
                b'$' => {
                    i = expand_var(bytes, i, &mut cur, &env_lookup)?;
                }
                _ => {
                    cur.push(b as char);
                    i += 1;
                }
            },
        }
    }

    // End-of-input: check for unterminated quotes.
    if matches!(state, S::Single) {
        return Err(SplitError::UnterminatedQuote { quote: '\'' });
    }
    if matches!(state, S::Double) {
        return Err(SplitError::UnterminatedQuote { quote: '"' });
    }

    if has_token {
        tokens.push(cur);
    }
    Ok(tokens)
}

/// Expand `${VAR}` starting at `i` (bytes[i] must be `b'$'`). Writes the
/// expansion into `cur` and returns the new index (just past the closing `}`).
/// Returns Err(MissingBrace) if no `{` follows `$` or no `}` terminates the name.
fn expand_var<F: Fn(&str) -> Option<String>>(
    bytes: &[u8],
    i: usize,
    cur: &mut String,
    env_lookup: F,
) -> Result<usize, SplitError> {
    debug_assert_eq!(bytes[i], b'$');
    if i + 1 >= bytes.len() || bytes[i + 1] != b'{' {
        return Err(SplitError::MissingBrace { at: i });
    }
    let mut j = i + 2;
    while j < bytes.len() && bytes[j] != b'}' {
        j += 1;
    }
    if j >= bytes.len() {
        return Err(SplitError::MissingBrace { at: i });
    }
    // Variable names in GNU env -S are ASCII (`[A-Za-z_][A-Za-z0-9_]*`); we do
    // not enforce the full grammar here — any UTF-8 slice is accepted and passed
    // to the lookup closure, which will simply return None for unknown names.
    let name = std::str::from_utf8(&bytes[i + 2..j]).unwrap_or("");
    let value = env_lookup(name).unwrap_or_default();
    cur.push_str(&value);
    Ok(j + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none_env(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn basic_two_tokens() {
        assert_eq!(split("foo bar", none_env).unwrap(), vec!["foo", "bar"]);
    }

    #[test]
    fn double_quoted_preserves_spaces() {
        assert_eq!(
            split(r#""hello world""#, none_env).unwrap(),
            vec!["hello world"]
        );
    }

    #[test]
    fn single_quoted_is_literal() {
        // \n inside single quotes stays as 2 chars \n (backslash + n)
        assert_eq!(split(r#"'a\n'"#, none_env).unwrap(), vec![r"a\n"]);
    }

    #[test]
    fn double_quoted_honors_escape() {
        assert_eq!(split(r#""a\tb""#, none_env).unwrap(), vec!["a\tb"]);
    }

    #[test]
    fn comment_strips_rest_of_line() {
        assert_eq!(
            split("#comment text", none_env).unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn trailing_comment_after_token() {
        // After a token and whitespace, `#` introduces a comment through EOL.
        assert_eq!(split("foo # trailing", none_env).unwrap(), vec!["foo"]);
    }

    #[test]
    fn backslash_c_early_exit() {
        assert_eq!(split(r"echo \c ignored", none_env).unwrap(), vec!["echo"]);
    }

    #[test]
    fn var_expansion_with_lookup() {
        let env = |n: &str| {
            if n == "VAR" {
                Some("expanded".to_string())
            } else {
                None
            }
        };
        assert_eq!(
            split("tok1 ${VAR} tok2", env).unwrap(),
            vec!["tok1", "expanded", "tok2"]
        );
    }

    #[test]
    fn bare_dollar_var_is_error() {
        let err = split("$VAR", none_env).unwrap_err();
        assert!(matches!(err, SplitError::MissingBrace { .. }));
    }

    #[test]
    fn missing_var_expands_to_empty() {
        // In delim context, ${MISSING} still starts a token (empty string).
        let result = split("${MISSING}", none_env).unwrap();
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn backslash_underscore_unquoted_delimits() {
        assert_eq!(split(r"a\_b", none_env).unwrap(), vec!["a", "b"]);
    }

    #[test]
    fn backslash_underscore_double_quoted_is_space() {
        assert_eq!(split(r#""a\_b""#, none_env).unwrap(), vec!["a b"]);
    }

    #[test]
    fn unterminated_double_quote_errors() {
        let err = split(r#"foo "bar"#, none_env).unwrap_err();
        assert_eq!(err, SplitError::UnterminatedQuote { quote: '"' });
    }

    #[test]
    fn unterminated_single_quote_errors() {
        let err = split(r#"foo 'bar"#, none_env).unwrap_err();
        assert_eq!(err, SplitError::UnterminatedQuote { quote: '\'' });
    }

    #[test]
    fn empty_input_yields_empty_vec() {
        assert_eq!(split("", none_env).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn input_too_long_rejected() {
        let long = "a".repeat(MAX_INPUT_LEN + 1);
        assert_eq!(split(&long, none_env), Err(SplitError::InputTooLong));
    }
}
