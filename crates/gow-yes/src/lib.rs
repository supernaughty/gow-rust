//! `uu_yes`: GNU `yes` — infinite repeat of a string (default `y`).
//!
//! Throughput-optimized via a 16 KiB prefill buffer + `write_all` loop on a
//! locked `stdout`. See RESEARCH.md Q4 for the performance rationale and the
//! original uutils reference pattern.
//!
//! - No args:        output `y\n` forever.
//! - One or more:    output `arg1 arg2 ... argN\n` forever (space-joined; D-23).
//! - `BrokenPipe`:   silently exit 0 — GNU behavior (`yes | head -1` exits 0).
//! - Other I/O err:  print `yes: {err}` to stderr and exit 1.
//!
//! UTIL-07, D-23.

use std::ffi::OsString;
use std::io::{self, Write};

/// 16 KiB is the documented sweet spot for throughput on modern Windows/Linux
/// page sizes (RESEARCH.md Q4 performance table: ~2 GB/s at 16 KiB). Stored in
/// a heap `Vec<u8>` rather than on the stack — 64 KiB would not fit on the
/// default test-thread stack, and 16 KiB is already plenty.
const BUF_SIZE: usize = 16 * 1024;

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    // Skip argv[0] (binary name) then collect remaining args.
    let mut iter = args.into_iter();
    let _argv0 = iter.next();
    let rest: Vec<OsString> = iter.collect();

    // Build the payload per D-23:
    //   no args          → "y\n"
    //   one or more args → "<arg1> <arg2> ... <argN>\n"  (space-joined)
    let payload: String = if rest.is_empty() {
        "y\n".to_owned()
    } else {
        let joined: Vec<String> = rest
            .iter()
            .map(|os| os.to_string_lossy().into_owned())
            .collect();
        let mut s = joined.join(" ");
        s.push('\n');
        s
    };

    run(payload.as_bytes())
}

/// Inner loop. Exposed at module scope (not `pub`) to keep the hot path small
/// and testable independently of argv parsing.
fn run(bytes: &[u8]) -> i32 {
    let mut buffer = vec![0u8; BUF_SIZE];
    let to_write = prepare_buffer(bytes, &mut buffer);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    loop {
        match out.write_all(to_write) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return 0,
            Err(e) => {
                eprintln!("yes: {e}");
                return 1;
            }
        }
    }
}

/// Fill `buffer` with as many copies of `input` as will fit when `input` is
/// short (less than half the buffer), returning a slice of the filled prefix.
/// For longer inputs, return `input` unchanged (no copy).
///
/// Pattern verbatim from uutils `yes.rs` (RESEARCH.md Q4 lines 313-324). The
/// dual-lifetime signature lets the caller reuse `buffer` across invocations;
/// in this utility `run` calls it exactly once.
pub(crate) fn prepare_buffer<'a>(input: &'a [u8], buffer: &'a mut [u8]) -> &'a [u8] {
    if input.len() < buffer.len() / 2 {
        let mut size = 0;
        while size + input.len() <= buffer.len() {
            buffer[size..size + input.len()].copy_from_slice(input);
            size += input.len();
        }
        &buffer[..size]
    } else {
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_buffer_fills_short_input() {
        // 2-byte input into a 16-byte buffer → 8 copies → full 16 bytes.
        let input = b"ab";
        let mut buf = [0u8; 16];
        let out = prepare_buffer(input, &mut buf);
        assert_eq!(out.len(), 16);
        assert_eq!(out, b"abababababababab");
    }

    #[test]
    fn prepare_buffer_fills_default_y_newline() {
        // Default `y\n` (2 bytes) behaves identically.
        let input = b"y\n";
        let mut buf = [0u8; 32];
        let out = prepare_buffer(input, &mut buf);
        assert_eq!(out.len(), 32);
        assert!(out.chunks(2).all(|c| c == b"y\n"));
    }

    #[test]
    fn prepare_buffer_returns_input_when_long() {
        // input length >= buffer/2 → return input as-is, not the buffer.
        let input = [1u8; 100];
        let mut buf = [0u8; 16];
        let out = prepare_buffer(&input, &mut buf);
        assert_eq!(out.as_ptr(), input.as_ptr());
        assert_eq!(out.len(), 100);
    }

    #[test]
    fn prepare_buffer_boundary_exactly_half() {
        // input.len() == buffer.len() / 2 → falls into `else` branch
        // (the condition is strictly `<`, not `<=`). Returns input unchanged.
        let input = [9u8; 8];
        let mut buf = [0u8; 16];
        let out = prepare_buffer(&input, &mut buf);
        assert_eq!(out.as_ptr(), input.as_ptr());
        assert_eq!(out.len(), 8);
    }
}
