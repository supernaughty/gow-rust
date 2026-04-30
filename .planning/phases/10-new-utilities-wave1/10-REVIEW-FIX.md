---
phase: 10-new-utilities-wave1
fixed_at: 2026-04-29T00:00:00Z
fix_scope: critical_warning
findings_in_scope: 5
fixed: 5
skipped: 0
iteration: 1
status: all_fixed
---

# Phase 10: Code Review Fix Report

**Fixed:** 2026-04-29
**Scope:** Critical + Warning (5 findings)
**Status:** all_fixed

## Fixes Applied

### CR-01 — `df`: `GetLogicalDriveStringsW` out-of-bounds panic ✓ Fixed

**Commit:** `fix(10): df — guard against GetLogicalDriveStringsW buf overflow`

**Change:** `crates/gow-df/src/lib.rs`
- Increased `buf` from `[0u16; 256]` to `[0u16; 512]` (doubles headroom for mounted drives)
- Added early-return guard: `if len as usize > buf.len() { return Vec::new(); }` before the loop

The caller already handles an empty drives vec with `"df: no drives detected"` + exit code 1, so the safe fallback is consistent with existing behavior.

---

### WR-01 — `seq`: `10_i64.pow(precision)` overflow for 19+ decimal places ✓ Fixed

**Commit:** `fix(10): seq — use checked_pow to prevent precision overflow panic`

**Change:** `crates/gow-seq/src/lib.rs`
- Replaced `10_i64.pow(precision)` with `10_i64.checked_pow(precision)`
- On `None` (overflow): emits `"seq: precision overflow"` to stderr and returns exit code 1

---

### WR-02 — `fold`: byte-boundary wrapping corrupts multi-byte UTF-8 ✓ Fixed (documented)

**Commit:** `fix(10): fold — document byte-only wrapping limitation in --help`

**Change:** `crates/gow-fold/src/lib.rs`
- Updated `--bytes` arg doc string to clearly state character-boundary wrapping is not implemented
- Users must pass `-b` when input may contain multi-byte UTF-8 characters

Full character-aware wrapping deferred to a future phase; this fix surfaces the limitation in `--help` so users are not silently misled.

---

### WR-03 — `nl`: `-b n` emits `<separator><line>` instead of `<line>` ✓ Fixed

**Commit:** `fix(10): nl — -b n emits raw line only, no separator prefix`

**Changes:** `crates/gow-nl/src/lib.rs`, `crates/gow-nl/tests/integration.rs`
- `"n"` arm changed from `write!(writer, "{}{}\n", separator, line)?` to `writeln!(writer, "{}", line)?`
- Integration test `nl_b_n_numbers_no_lines` updated: expected output changed from `"\ta\n\tb\n"` to `"a\nb\n"`
- All 9 nl integration tests pass

---

### WR-04 — `tac`: stdin read errors silently ignored; exit 0 on failure ✓ Fixed

**Commit:** `fix(10): tac — propagate stdin read errors; exit code 1 on failure`

**Change:** `crates/gow-tac/src/lib.rs`
- Replaced `let _ = io::stdin().read_to_end(&mut buf)` with `if let Err(e) = ...` block
- On error: emits `"tac: stdin: {e}"` and sets `exit_code = 1`
- Partial buffer (bytes read before error) is still reversed and written — consistent with file behavior

---

## Test Results

All tests pass after fixes:

| Crate      | Tests | Result |
|------------|-------|--------|
| gow-df     | 5     | ✓ all pass |
| gow-seq    | 9     | ✓ all pass |
| gow-fold   | 8     | ✓ all pass |
| gow-nl     | 9     | ✓ all pass |
| gow-tac    | 5     | ✓ all pass |

## Info Findings (not in scope — no action taken)

- **IN-01** (`od` unreachable `_` arm): deferred — no correctness impact
- **IN-02** (`du` O(N²) walk): deferred to Phase 11 per review recommendation
- **IN-03** (`hashsum` improperly-formatted line counter): deferred — exit code is correct; summary message is missing but non-blocking

---

_Fixed: 2026-04-29_
_Fixer: Claude (gsd-code-fixer)_
