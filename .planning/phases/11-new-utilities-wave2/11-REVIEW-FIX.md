---
phase: 11-new-utilities-wave2
fix_scope: critical_warning
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
iteration: 1
fixed_at: 2026-04-30T00:00:00Z
review_path: .planning/phases/11-new-utilities-wave2/11-REVIEW.md
---

# Phase 11: Code Review Fix Report

**Fixed at:** 2026-04-30T00:00:00Z
**Source review:** .planning/phases/11-new-utilities-wave2/11-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (WR-01 through WR-05; Info findings excluded per fix_scope = critical_warning)
- Fixed: 5
- Skipped: 0

## Fixed Issues

### WR-01: `printf` pad_string uses byte length, not character length

**Files modified:** `crates/gow-printf/src/lib.rs`
**Commit:** 1e2233e
**Applied fix:** Introduced `char_len = s.chars().count()` in `pad_string` and replaced the `s.len() >= width` guard and `width - s.len()` computation with `char_len`-based equivalents. Also updated the `'s'` format specifier branch guard at line 235 from `width > s.len()` to `width > s.chars().count()`. Multi-byte UTF-8 arguments now receive correct padding widths.

---

### WR-02: `split` parse_bytes silent integer overflow on large suffixed values

**Files modified:** `crates/gow-split/src/lib.rs`
**Commit:** b6f6741
**Applied fix:** Replaced `num_str.parse::<usize>().ok().map(|n| n * mult)` with an explicit `?`-based parse followed by `n.checked_mul(mult)`. Overflow on inputs like `9999999G` now returns `None`, which the existing `parse_bytes` call site converts into the `"invalid number of bytes"` error message and exit code 1.

---

### WR-03: `join` read_line silently swallows I/O errors — treated as EOF

**Files modified:** `crates/gow-join/src/lib.rs`
**Commit:** eea3057
**Applied fix:** Changed `read_line` return type from `Option<String>` to `Result<Option<String>, io::Error>`. Added a `next_line!` macro inside `run()` that matches the result: `Ok(v)` continues as before; `Err(e)` emits `"join: read error on file N: {e}"` to stderr and returns exit code 1. All ten call sites in the loop body were updated to use `next_line!`.

---

### WR-04: `expr` colon operator and `match` function return byte length, not character length

**Files modified:** `crates/gow-expr/src/lib.rs`
**Commit:** eb42192
**Applied fix:** In `parse_colon` (no-capturing-group branch): replaced `.map(|m| m.as_str().len())` with `.map(|m| m.as_str().chars().count())`. In `parse_atom` `"match"` branch: replaced `m.len()` with `m.as_str().chars().count()`. Both sites now count Unicode scalar values, consistent with the `length` keyword and with GNU `expr` behaviour.

---

### WR-05: `join` accepts invalid `-a`/`-v` values without error

**Files modified:** `crates/gow-join/src/lib.rs`
**Commit:** ee9a459
**Applied fix:** Added a validation block at the top of `run()`, after the 2-file check. It iterates over the optional `-a` and `-v` values and rejects any value other than 1 or 2 with a `"join: invalid file number in -{flag}: {val}"` diagnostic and exit code 2, matching GNU `join` behaviour.

---

## Skipped Issues

None — all five in-scope findings were successfully fixed.

---

_Fixed: 2026-04-30T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
