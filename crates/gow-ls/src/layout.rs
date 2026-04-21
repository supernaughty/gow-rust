//! Column layout for short-form ls output.
//!
//! Uses `terminal_size` to detect the current terminal width. When stdout is
//! not a tty (piped / redirected), returns `None` so callers can fall back to
//! single-column output (GNU `ls` convention).

/// Determine the column width available for output. Returns `None` when stdout
/// is not a tty (piped / redirected).
pub fn detect_width() -> Option<usize> {
    use terminal_size::{terminal_size, Width};
    match terminal_size() {
        Some((Width(w), _)) if w > 0 => Some(w as usize),
        _ => None,
    }
}

/// Compute the number of columns given the available width, the longest
/// entry name length, and padding between columns (2 spaces).
pub fn compute_columns(width: usize, max_name: usize) -> usize {
    if max_name == 0 {
        return 1;
    }
    let col_width = max_name + 2;
    std::cmp::max(1, width / col_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_columns_basic() {
        // 80-col terminal, 10-char names + 2 pad = 12 col width → 6 cols.
        assert_eq!(compute_columns(80, 10), 6);
    }

    #[test]
    fn compute_columns_floor_one() {
        // Single name wider than terminal — always at least 1 col.
        assert_eq!(compute_columns(10, 50), 1);
    }

    #[test]
    fn compute_columns_zero_name() {
        // Defensive: no entries → 1 col (prevents divide-by-zero).
        assert_eq!(compute_columns(80, 0), 1);
    }

    #[test]
    fn compute_columns_large_terminal() {
        assert_eq!(compute_columns(200, 10), 16);
    }
}
