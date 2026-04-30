---
phase: 11-new-utilities-wave2
reviewed: 2026-04-30T00:00:00Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - Cargo.toml
  - build.bat
  - extras/bin/[.bat
  - crates/gow-expr/src/lib.rs
  - crates/gow-expr/tests/integration.rs
  - crates/gow-fmt/src/lib.rs
  - crates/gow-fmt/tests/integration.rs
  - crates/gow-join/src/lib.rs
  - crates/gow-join/tests/integration.rs
  - crates/gow-paste/src/lib.rs
  - crates/gow-paste/tests/integration.rs
  - crates/gow-printf/src/lib.rs
  - crates/gow-printf/tests/integration.rs
  - crates/gow-split/src/lib.rs
  - crates/gow-split/tests/integration.rs
  - crates/gow-test/src/lib.rs
  - crates/gow-test/tests/integration.rs
  - crates/gow-uname/src/lib.rs
  - crates/gow-uname/tests/integration.rs
  - crates/gow-unlink/src/lib.rs
  - crates/gow-unlink/tests/integration.rs
  - crates/gow-whoami/src/lib.rs
  - crates/gow-whoami/tests/integration.rs
findings:
  critical: 0
  warning: 5
  info: 6
  total: 11
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-04-30T00:00:00Z
**Depth:** standard
**Files Reviewed:** 23
**Status:** issues_found

## Summary

Phase 11 delivers ten new utilities: `whoami`, `uname`, `paste`, `join`, `split`, `printf`, `expr`, `test`/`[`, `fmt`, and `unlink`. The overall quality is high — every utility is well-structured, follows existing project conventions, uses proper Windows API interop with clear SAFETY comments, and has meaningful integration test coverage. No security vulnerabilities were found.

Five warnings were identified, all logic or correctness issues: a byte-vs-character width calculation bug in `printf` that produces wrong output for non-ASCII arguments; an unchecked arithmetic overflow in `split` byte-size parsing; a silent error swallow in `join`'s line reader; an inconsistency between `expr`'s `:` operator and `match` function when counting match lengths on multi-byte input; and a missing validation for `-a`/`-v` flag values in `join` that allows nonsensical values (e.g. `-a 3`) to silently do nothing.

Six informational items cover minor style issues: silently dropped multi-char separators in `join`, a minor `expr` test coverage gap for `|`/`&` short-circuit logic, a redundant double-break in `printf`'s loop, unconventional `consumed += 1` placement for `%%` (the `%%` branch correctly does NOT increment but the comment could be clearer), dead `.clone()` calls in `join`'s drain loops, and a missing test for `split -n` with zero chunks.

---

## Warnings

### WR-01: `printf` pad_string uses byte length, not character length — produces wrong output for multi-byte args

**File:** `crates/gow-printf/src/lib.rs:235,289,292`

**Issue:** `pad_string` uses `s.len()` (byte count) to compare against `width` and compute padding. For any format argument containing non-ASCII characters (e.g. `printf "%10s" "héllo"`), the padding count is wrong — `"héllo".len()` is 6 bytes but 5 chars, so the string gets one fewer space of right-padding than requested. GNU `printf` counts display columns (or at minimum Unicode scalar values), not bytes.

**Fix:**
```rust
// Replace s.len() with s.chars().count() throughout pad_string:
fn pad_string(s: &str, width: usize, left_align: bool, pad_char: char, is_negative_zero_pad: bool) -> String {
    let char_len = s.chars().count();
    if width == 0 || char_len >= width {
        return s.to_string();
    }
    let pad_count = width - char_len;
    // ... rest unchanged, using pad_count ...
}
```
Also update the `'s'` branch at line 235 to use `s.chars().count()` instead of `s.len()` in the `if width > s.len()` guard.

---

### WR-02: `split` parse_bytes silent integer overflow on 32-bit usize

**File:** `crates/gow-split/src/lib.rs:56`

**Issue:** `num_str.parse::<usize>().ok().map(|n| n * mult)` performs an unchecked multiplication. On a 64-bit Windows build `usize` is 8 bytes and the risk is low in practice, but if a user passes `split -b 9999999G` the multiplication `9999999 * 1024 * 1024 * 1024` wraps silently in debug mode (panics) or produces a truncated byte count in release builds (where overflow is defined to wrap per `panic = "abort"` profile). The function returns `Some(wrapped_value)` which then passes the `n > 0` guard and splits into absurdly-sized chunks with no error.

**Fix:**
```rust
fn parse_bytes(s: &str) -> Option<usize> {
    let s = s.trim();
    let (num_str, mult) = /* ... existing suffix detection ... */;
    let n = num_str.parse::<usize>().ok()?;
    n.checked_mul(mult)  // returns None on overflow → caller emits "invalid number of bytes"
}
```

---

### WR-03: `join` read_line silently swallows I/O errors — treated identically to EOF

**File:** `crates/gow-join/src/lib.rs:110`

**Issue:** When `reader.read_line(&mut line)` returns `Err(_)`, `read_line` returns `None`. The caller then treats this as end-of-file and moves the unmatched file's remaining lines to the "unpaired" output path (or silently discards them if `-a`/`-v` is not set). A disk error or encoding problem mid-file produces wrong output with exit code 0 — no diagnostic is emitted.

**Fix:**
```rust
fn read_line(reader: &mut Box<dyn BufRead>) -> Result<Option<String>, io::Error> {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => return Ok(None),
            Ok(_) => {
                if line.ends_with('\n') { line.pop(); if line.ends_with('\r') { line.pop(); } }
                return Ok(Some(line));
            }
            Err(e) => return Err(e),
        }
    }
}
// Callers propagate Err with eprintln! and return exit code 1.
```

---

### WR-04: `expr` colon operator and `match` function return byte length, not character length

**File:** `crates/gow-expr/src/lib.rs:229,294`

**Issue:** When the `:` operator or `match` function matches without a capturing group, it returns the match length using `.len()` (byte count). GNU `expr` returns the number of characters matched. For ASCII-only input this is equivalent, but for a multi-byte match (e.g. `expr "héllo" : "hél"`) the result is 5 (bytes) instead of 3 (characters), diverging from GNU behavior. This is inconsistent with the `length` keyword on line 261 which correctly uses `.chars().count()`.

**Fix:**
```rust
// Line 229 — colon operator no-capture branch:
return Ok(caps.get(0).map(|m| m.as_str().chars().count()).unwrap_or(0).to_string());

// Line 294 — match function:
let result = if let Some(m) = re.find(&s) { m.as_str().chars().count() } else { 0 };
```

---

### WR-05: `join` accepts invalid `-a`/`-v` values (e.g. 3, 0) without error

**File:** `crates/gow-join/src/lib.rs:181,201`

**Issue:** `-a` and `-v` accept any `u8` value via clap. Values other than 1 or 2 pass silently: the comparisons `cli.print_unpaired == Some(1)` and `cli.print_unpaired == Some(2)` both evaluate to false, so the flag has no effect and `join` behaves as if `-a`/`-v` were not specified — no error, no warning. GNU `join` rejects invalid file numbers with a usage error.

**Fix:** Add validation at the top of `run()`:
```rust
for &flag_val in [cli.print_unpaired, cli.only_unpaired].iter().flatten() {
    if flag_val != 1 && flag_val != 2 {
        eprintln!("join: invalid file number in -{}: {flag_val}", if cli.print_unpaired.is_some() { "a" } else { "v" });
        return 2;
    }
}
```

---

## Info

### IN-01: `join` separator silently drops all but the first character of a multi-char `-t` argument

**File:** `crates/gow-join/src/lib.rs:159-162`

**Issue:** `cli.separator.as_ref().and_then(|s| s.chars().next())` silently takes only the first character of a multi-char `-t` value (e.g. `-t "::"` becomes `':'`). GNU `join` rejects multi-char separators with an error. The silent truncation can confuse users who mistype the flag.

**Fix:** Emit an error if the separator string has more than one character:
```rust
let sep: Option<char> = match &cli.separator {
    None => None,
    Some(s) if s.chars().count() == 1 => s.chars().next(),
    Some(s) => {
        eprintln!("join: multi-character tab '{s}'");
        return 2;
    }
};
```

---

### IN-02: `join` drain loops use unnecessary `.clone()` on `line1`/`line2`

**File:** `crates/gow-join/src/lib.rs:188,207`

**Issue:** `while let Some(ref l1) = line1.clone()` clones the entire `Option<String>` on every iteration just to avoid a borrow conflict. This is avoidable with a simple `while` + `take`/reassign pattern. Not a correctness problem but wastes allocations in large files.

**Fix:**
```rust
// Instead of cloning in the loop condition:
while let Some(l1) = line1.take() {
    let key1 = get_field(&l1, cli.field1, sep).to_string();
    if should_print {
        let f1_other = other_fields(&l1, cli.field1, sep);
        print_joined(&mut out, &key1, &f1_other, "", sep);
    }
    line1 = read_line(&mut reader1);
}
```

---

### IN-03: `printf` loop has redundant double-break condition

**File:** `crates/gow-printf/src/lib.rs:35-40`

**Issue:** The repeat loop checks `arg_idx >= total_args` twice: once in the combined condition on line 35 and again immediately after incrementing on line 39. The second check on line 39 is never reached when the condition on line 35 already handles it (since `args_consumed == 0` covers the no-specifier case). The logic is correct but the duplicate check is dead code that makes the loop harder to reason about.

**Fix:** Simplify:
```rust
loop {
    let (output, args_consumed) = format_one_pass(format_str, &all_args[arg_idx..]);
    if let Err(e) = out.write_all(output.as_bytes()) {
        eprintln!("printf: write error: {e}");
        return 1;
    }
    if args_consumed == 0 || arg_idx + args_consumed >= total_args {
        break;
    }
    arg_idx += args_consumed;
}
```

---

### IN-04: `expr` integration tests do not cover `|` and `&` short-circuit semantics

**File:** `crates/gow-expr/tests/integration.rs`

**Issue:** The `|` (or) and `&` (and) operators are implemented but not tested in the integration suite. Both have subtle GNU semantics: `|` returns the lhs value (not "1") if lhs is non-null, and `&` returns the lhs value if both operands are non-null. These are easy to regress silently.

**Fix:** Add tests covering:
- `expr 5 \| 0` → output `5`, exit 0
- `expr 0 \| 3` → output `3`, exit 0
- `expr 0 \| 0` → output `0`, exit 1
- `expr 5 \& 3` → output `5`, exit 0
- `expr 0 \& 3` → output `0`, exit 1

---

### IN-05: `split -n 0` is not tested and the path differs from `-l 0`

**File:** `crates/gow-split/src/lib.rs:156-159`

**Issue:** `-n 0` (zero chunks) correctly returns exit code 1 with an error message, but this is not covered by any integration test. The `-l 0` case is tested (`split_zero_lines_exits_1`). Adding a matching test for `-n 0` would complete the validation boundary coverage.

**Fix:** Add a test analogous to `split_zero_lines_exits_1`:
```rust
#[test]
fn split_zero_chunks_exits_1() {
    let f = write_temp("a\n");
    let dir = TempDir::new().unwrap();
    let prefix = dir.path().join("x").to_str().unwrap().to_string();
    Command::cargo_bin("split").unwrap()
        .args(["-n", "0", f.path().to_str().unwrap(), &prefix])
        .assert().failure().code(1).stderr(contains("split:"));
}
```

---

### IN-06: `test` binary-expression parser accepts ambiguous single-token strings that look like operators

**File:** `crates/gow-test/src/lib.rs:187-189`

**Issue:** The fallthrough at the bottom of `parse_primary` (lines 187-189) treats any unrecognized token as a string truthiness test. If a user writes `test -q` (unknown flag), the `-q` token does not match any known predicate, falls through to the single-string path, and returns `true` (non-empty string) with exit 0 — silently ignoring the unknown flag rather than reporting a usage error. GNU `test` also behaves this way for POSIX compliance, so this is a known trade-off, but it can mask typos like `test -ef` (no `-ef` support).

**Fix (optional):** Consider logging a warning to stderr for tokens that look like flags (start with `-` followed by letters) but are not recognized. This is non-standard but would aid debugging.

---

_Reviewed: 2026-04-30T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
