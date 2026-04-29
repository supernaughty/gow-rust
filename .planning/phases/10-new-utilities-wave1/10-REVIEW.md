---
phase: 10-new-utilities-wave1
reviewed: 2026-04-29T00:00:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - Cargo.toml
  - build.bat
  - crates/gow-df/src/lib.rs
  - crates/gow-df/tests/integration.rs
  - crates/gow-du/src/lib.rs
  - crates/gow-du/tests/integration.rs
  - crates/gow-expand-unexpand/src/lib.rs
  - crates/gow-expand-unexpand/tests/integration.rs
  - crates/gow-fold/src/lib.rs
  - crates/gow-fold/tests/integration.rs
  - crates/gow-hashsum/src/lib.rs
  - crates/gow-hashsum/tests/integration.rs
  - crates/gow-nl/src/lib.rs
  - crates/gow-nl/tests/integration.rs
  - crates/gow-od/src/lib.rs
  - crates/gow-od/tests/integration.rs
  - crates/gow-seq/src/lib.rs
  - crates/gow-seq/tests/integration.rs
  - crates/gow-sleep/src/lib.rs
  - crates/gow-sleep/tests/integration.rs
  - crates/gow-tac/src/lib.rs
  - crates/gow-tac/tests/integration.rs
findings:
  critical: 1
  warning: 4
  info: 3
  total: 8
status: issues_found
---

# Phase 10: Code Review Report

**Reviewed:** 2026-04-29
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

Phase 10 introduces ten new utilities: `seq`, `sleep`, `tac`, `nl`, `od`, `fold`, `expand`/`unexpand`, `du`, `df`, and `hashsum` (md5sum/sha1sum/sha256sum). Overall the implementation quality is good — consistent patterns, proper error propagation, GNU-compatible exit codes, and strong test coverage. All crates follow the established `gow_core::init()` / `parse_gnu()` pattern.

Three areas need attention before this code ships:

1. **Critical** — `gow-df` `get_drives()` iterates up to `len` elements from a 256-element buffer; if `GetLogicalDriveStringsW` returns a required-length > 256 it will trigger an out-of-bounds panic.
2. **Warning** — `gow-seq` `seq_output()` calls `10_i64.pow(precision)` without overflow protection; a 19+ decimal-place input causes a panic in debug mode and silently wrong output in release.
3. **Warning** — `gow-fold` always wraps on byte boundaries regardless of the `-b` flag; multi-byte UTF-8 sequences are split mid-codepoint without `-b`, corrupting non-ASCII output.
4. **Warning** — `gow-nl` `-b n` outputs `<separator><line>` (e.g., `\ta`) rather than just `<line>`, diverging from GNU `nl` which omits both the number field and separator for unnumbered body lines.
5. **Warning** — `gow-tac` silently swallows stdin read errors and always exits 0 for the stdin path.

---

## Critical Issues

### CR-01: `df` out-of-bounds panic when `GetLogicalDriveStringsW` returns `len > buf.len()`

**File:** `crates/gow-df/src/lib.rs:37-50`

**Issue:** `buf` is a fixed `[0u16; 256]` stack array. The Win32 API `GetLogicalDriveStringsW` is called with `buf.len() as u32` (256) as the buffer capacity. Per MSDN, when the buffer is too small, the return value is the required size (greater than the capacity) and the buffer is left untouched. The loop then iterates `for i in 0..len as usize`, which indexes `buf[0]` through `buf[len-1]`. If `len > 256`, accesses starting at `buf[256]` are out-of-bounds and Rust will panic.

In practice, 26 standard drive letters need 26 × 4 = 104 u16 entries, well within 256. However systems with many mounted RAM drives, USB sticks, or SUBST drives can exceed this. The practical threshold is 64 drives (64 × 4 = 256 entries, leaving no room for the double-null terminator).

```rust
// BEFORE (crates/gow-df/src/lib.rs:36-51)
fn get_drives() -> Vec<String> {
    let mut buf = [0u16; 256];
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) };
    if len == 0 {
        return Vec::new();
    }
    let mut drives = Vec::new();
    let mut start = 0usize;
    for i in 0..len as usize {           // BUG: len may exceed buf.len()
        if buf[i] == 0 {
            ...
        }
    }
    drives
}

// AFTER — cap the iteration to the actual buffer length
fn get_drives() -> Vec<String> {
    let mut buf = [0u16; 512];           // increase size for safety
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) };
    if len == 0 {
        return Vec::new();
    }
    if len as usize > buf.len() {
        // Buffer was too small (extremely unlikely, >128 drives); return empty
        // Caller emits "df: no drives detected"
        return Vec::new();
    }
    let mut drives = Vec::new();
    let mut start = 0usize;
    for i in 0..len as usize {
        if buf[i] == 0 {
            if i > start {
                drives.push(String::from_utf16_lossy(&buf[start..i]));
            }
            start = i + 1;
        }
    }
    drives
}
```

---

## Warnings

### WR-01: `seq` — `10_i64.pow(precision)` overflows for 19+ decimal places

**File:** `crates/gow-seq/src/lib.rs:112-115`

**Issue:** `precision` is a `u32` derived by counting decimal digits in the input strings. For inputs like `seq 0.1234567890123456789 1 1` (19 decimal places), `10_i64.pow(19)` overflows `i64::MAX` (~9.2×10¹⁸). In debug builds this panics with no user-visible error message; in release builds it silently wraps and produces wrong output. The subsequent `(first * scale as f64).round() as i64` cast is safe since Rust 1.45 saturates on overflow, but the overflow in `pow` itself is not.

```rust
// BEFORE (crates/gow-seq/src/lib.rs:112)
let scale: i64 = 10_i64.pow(precision);

// AFTER — cap precision and use checked arithmetic
const MAX_PRECISION: u32 = 18; // 10^18 < i64::MAX
if precision > MAX_PRECISION {
    eprintln!("seq: precision too large (max {} decimal places)", MAX_PRECISION);
    return 0; // GNU seq silently ignores extreme precision; error is also acceptable
}
let scale: i64 = 10_i64.pow(precision);
```

Alternatively, use `i64::checked_pow` and treat `None` as a hard-break:

```rust
let scale = match 10_i64.checked_pow(precision) {
    Some(s) => s,
    None => {
        eprintln!("seq: precision overflow");
        return 1;
    }
};
```

---

### WR-02: `fold` — byte-boundary wrapping corrupts multi-byte UTF-8 without `-b`

**File:** `crates/gow-fold/src/lib.rs:42-87`

**Issue:** `wrap_line` receives `line.as_bytes()` and counts units in bytes. This is correct when `-b` (byte mode) is active, but GNU `fold` without `-b` wraps on character boundaries (Unicode code points). Since `wrap_reader` always passes raw bytes, multi-byte UTF-8 sequences (e.g., `é` = `\xc3\xa9`, 2 bytes) are split mid-codepoint, producing garbled output on any non-ASCII input.

The Cli struct comments this as "Phase 10 always counts bytes" — this should be explicitly documented as a known gap, and the `-b` flag should not silently claim character mode is available:

```rust
// Current Cli doc (line 31):
/// Count bytes rather than characters (accepted for compatibility; Phase 10 always counts bytes)
#[arg(short = 'b', long = "bytes", action = ArgAction::SetTrue)]
bytes: bool,

// The fix is to implement char-aware wrapping in wrap_line when !cli.bytes.
// Minimal fix for Phase 10: reject non-ASCII input or document the limitation
// prominently in --help so users know to always pass -b.
```

For Phase 10, the minimally correct fix is to add a note to `--help` that character mode is not yet implemented and that `-b` must be used for correct behavior.

---

### WR-03: `nl` — `-b n` emits separator prefix, deviating from GNU behavior

**File:** `crates/gow-nl/src/lib.rs:60-63`

**Issue:** When body-numbering style is `n` (number no lines), the implementation outputs `<separator><line>` (e.g., `\ta\n` with the default tab separator). GNU `nl -b n` outputs only the raw line text — neither a number nor a separator is prepended. The integration test `nl_b_n_numbers_no_lines` encodes and tests the non-compliant behavior, so this is a compatibility gap that scripts relying on GNU `nl -b n` output would fail.

```rust
// BEFORE (crates/gow-nl/src/lib.rs:60-63)
"n" => {
    // Number no lines — emit separator then line
    write!(writer, "{}{}\n", separator, line)?;
}

// AFTER — GNU nl -b n: no number, no separator, just the line
"n" => {
    writeln!(writer, "{}", line)?;
}
```

The integration test must also be updated from:
```rust
.stdout("\ta\n\tb\n");   // wrong: has separator
```
to:
```rust
.stdout("a\nb\n");       // correct: raw lines
```

---

### WR-04: `tac` — stdin read errors silently ignored; exit code stays 0

**File:** `crates/gow-tac/src/lib.rs:61`

**Issue:** The stdin path calls `io::stdin().read_to_end(&mut buf)` and discards the `Result` with `let _ = ...`. If stdin read fails (e.g., broken pipe mid-read), `tac` proceeds with a partial buffer and exits 0. File paths have correct error handling (lines 68-75). The stdin path should mirror that behavior.

```rust
// BEFORE (crates/gow-tac/src/lib.rs:61)
let _ = io::stdin().read_to_end(&mut buf);
reverse_and_write(&buf, &mut out);

// AFTER
if let Err(e) = io::stdin().read_to_end(&mut buf) {
    eprintln!("tac: stdin: {e}");
    exit_code = 1;
}
reverse_and_write(&buf, &mut out);
```

---

## Info

### IN-01: `od` — unreachable `_` arm in `format_value` is dead code

**File:** `crates/gow-od/src/lib.rs:199`

**Issue:** The `_` fallback arm `format!("{:?}", bytes)` can never be reached if `parse_type_spec` validates all inputs correctly. All valid `TypeSpec` variants are explicitly covered. This arm adds noise and exposes a `{:?}` debug format that would surprise users if ever triggered.

**Fix:** Replace with `unreachable!()` or `#[allow(unreachable_patterns)]` with a comment, to make the invariant explicit:

```rust
// Replace the _ arm with:
_ => unreachable!("format_value called with size not covered by parse_type_spec"),
```

---

### IN-02: `du` — double-walk for each directory entry is quadratic

**File:** `crates/gow-du/src/lib.rs:122-133`

**Issue:** In the non-`--summarize` path, the outer `WalkDir` traverses all directory entries and for each directory entry calls `dir_usage_recursive()` which does a full inner `WalkDir` starting from that directory. For a directory tree with N total entries, this performs O(N²) filesystem stat calls. This is a quality concern; it does not produce incorrect results.

**Fix (deferred to Phase 11 if desired):** Build a single-pass tally using a stack or depth-first traversal that accumulates sizes bottom-up, rather than re-walking each subdirectory from scratch.

---

### IN-03: `hashsum` — improperly-formatted checksum line counter not reported at end

**File:** `crates/gow-hashsum/src/lib.rs:179-186`

**Issue:** GNU `md5sum -c` reports a summary like `md5sum: WARNING: 1 line is improperly formatted` at the end of output when some lines are malformed. The current implementation emits per-line errors and sets the exit code but does not emit the summary count. Scripts that parse this summary will not see it.

**Fix:** Accumulate an `improperly_formatted_count: usize` counter inside `process_check_lines` and emit the GNU-style summary after the loop:

```rust
if improperly_formatted > 0 {
    eprintln!(
        "{}: WARNING: {} line{} improperly formatted",
        algo_name(algo),
        improperly_formatted,
        if improperly_formatted == 1 { " is" } else { "s are" }
    );
}
```

---

_Reviewed: 2026-04-29_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
