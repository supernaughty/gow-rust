//! GNU `env -S STRING` split-string parser.
//!
//! TDD RED phase: tests only, `split` returns placeholder. Implementation
//! arrives in the GREEN commit.
//!
//! Reference: RESEARCH.md Q7 lines 590-672. Security: input bounded by caller
//! (typical CLI max 10 KB); no recursion; 1-pass expansion (expanded values are
//! NOT re-parsed per Pitfall 6).

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
/// TDD RED stub — unconditionally panics so tests fail loudly. Real state machine
/// lands in the GREEN commit.
pub fn split<F: Fn(&str) -> Option<String>>(
    _input: &str,
    _env_lookup: F,
) -> Result<Vec<String>, SplitError> {
    unimplemented!("split(): GREEN phase implementation pending")
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
