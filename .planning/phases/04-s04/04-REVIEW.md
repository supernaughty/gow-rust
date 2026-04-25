---
phase: 04-s04
reviewed: 2026-04-25T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - Cargo.toml
  - crates/gow-awk/Cargo.toml
  - crates/gow-awk/src/lib.rs
  - crates/gow-awk/tests/integration.rs
  - crates/gow-diff/Cargo.toml
  - crates/gow-diff/src/lib.rs
  - crates/gow-diff/tests/integration.rs
  - crates/gow-patch/Cargo.toml
  - crates/gow-patch/src/lib.rs
  - crates/gow-patch/tests/integration.rs
findings:
  critical: 1
  warning: 4
  info: 3
  total: 8
status: issues_found
---

# Phase 04: Code Review Report

**Reviewed:** 2026-04-25T00:00:00Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Three crates were reviewed: `gow-awk` (a full AWK interpreter written from scratch), `gow-diff` (unified diff using the `similar` crate), and `gow-patch` (patch application using the `diffy` crate). The workspace `Cargo.toml` is clean.

`gow-diff` and `gow-patch` are well-structured and largely correct. `gow-awk` is the most complex file and contains the majority of findings. The most significant issue is a byte-index slice into a UTF-8 `String` inside `format_printf` (`%s` precision truncation), which will **panic** on any multi-byte character. A logic bug in `parse_ternary` also means AWK's `expr ? t : f` syntax silently produces wrong results. The exit-code signalling via a mangled string is fragile but functional.

---

## Critical Issues

### CR-01: Panic on multi-byte UTF-8 string in `%s` precision truncation

**File:** `crates/gow-awk/src/lib.rs:1799-1803`

**Issue:** The `%s` printf format specifier truncates the string using a raw byte-index slice: `s[..prec]`. If the string contains any multi-byte UTF-8 character (e.g., Japanese, emoji, accented Latin) and `prec` falls in the middle of a code point, Rust will panic with `byte index N is not a char boundary`.

```rust
// current — panics if prec splits a multibyte char
's' => {
    let s = arg.to_str();
    let s = if let Some(prec) = precision {
        if prec < s.len() {
            s[..prec].to_string()  // BUG: byte index, not char count
```

**Fix:**
```rust
's' => {
    let s = arg.to_str();
    let s = if let Some(prec) = precision {
        // Truncate by character count, not byte count
        let chars: Vec<char> = s.chars().collect();
        if prec < chars.len() {
            chars[..prec].iter().collect()
        } else {
            s
        }
    } else {
        s
    };
    pad_string(&s, width, left_align, ' ')
}
```

---

## Warnings

### WR-01: AWK ternary operator `?:` is never parsed — silently ignored

**File:** `crates/gow-awk/src/lib.rs:1116-1124`

**Issue:** The `Expr::Ternary` AST variant exists and is correctly evaluated at lines 1955-1963, but `parse_ternary()` never actually parses the `?` token — there is no `?` in the `Token` enum and the function body is a no-op that just calls `parse_or()`. Any AWK program using `condition ? value1 : value2` will silently mis-parse: the `?` character triggers `AwkError::Parse("unexpected character: '?'")` from the lexer's `_` arm at line 489. So the operator fails loudly rather than silently, but it is wholly unimplemented despite the evaluator code existing for it.

```rust
// current — stub, never produces Expr::Ternary
fn parse_ternary(&mut self) -> Result<Expr> {
    let cond = self.parse_or()?;
    if matches!(self.peek(), Token::Gt) {
        // Could be ternary ? but we don't have ? token...
        // AWK doesn't actually have ?:, skip  <-- incorrect: AWK *does* have ?:
        Ok(cond)
    } else {
        Ok(cond)
    }
}
```

**Fix:** Add `Token::Question` and `Token::Colon` to the `Token` enum, emit them in the lexer for `?` and `:`, and implement the ternary parse:

```rust
// In Token enum
Question,
Colon,

// In parse_ternary
fn parse_ternary(&mut self) -> Result<Expr> {
    let cond = self.parse_or()?;
    if matches!(self.peek(), Token::Question) {
        self.advance(); // consume '?'
        let then_expr = self.parse_expr()?;
        self.expect(&Token::Colon)?;
        let else_expr = self.parse_expr()?;
        Ok(Expr::Ternary(Box::new(cond), Box::new(then_expr), Box::new(else_expr)))
    } else {
        Ok(cond)
    }
}
```

---

### WR-02: `pad_string` uses byte length for width comparison, garbling multi-byte output

**File:** `crates/gow-awk/src/lib.rs:1832-1844`

**Issue:** `pad_string` compares `s.len()` (byte count) against `width` (a character-count field-width specifier). For strings containing multi-byte characters, `s.len()` is larger than the visible width, so the padding calculation undercounts and wrong padding (or no padding) is emitted. The slices `&s[..1]` at line 1841 also assume the sign character is a single byte, which it always is, so that specific line is safe — but the length check is wrong.

```rust
// current
fn pad_string(s: &str, width: usize, left_align: bool, pad_char: char) -> String {
    if s.len() >= width {   // BUG: byte length vs display width
        return s.to_string();
    }
```

**Fix:**
```rust
fn pad_string(s: &str, width: usize, left_align: bool, pad_char: char) -> String {
    let char_len = s.chars().count();
    if char_len >= width {
        return s.to_string();
    }
    let padding: String = std::iter::repeat(pad_char).take(width - char_len).collect();
    // ... rest unchanged
}
```

---

### WR-03: Exit-code propagation via magic error string prefix is fragile

**File:** `crates/gow-awk/src/lib.rs:2314` (production) and `2787-2792` (consumer)

**Issue:** When `exit N` is executed inside a user-defined function call, the code signals the exit code by returning an `anyhow::Error` whose `to_string()` is `"__exit__N"`. The top-level handler then strips this prefix to recover `N`. If any underlying I/O or regex error message ever starts with `"__exit__"` by coincidence the wrong exit code would be extracted. This is fragile design; it also prevents error context from being chained on the `anyhow` error without corrupting the magic prefix.

```rust
// production — at line 2314
return Err(anyhow!("__exit__{}", code));

// consumer — at line 2788
if let Some(code_str) = msg.strip_prefix("__exit__") {
    code_str.parse::<i32>().unwrap_or(1)
```

**Fix:** Use a dedicated error variant or a `Result`-like enum to propagate exit codes rather than encoding them in an error message:

```rust
// In ControlFlow (already exists) — bubble Exit upward through execute()
// rather than converting to anyhow::Error.
// In eval_fn_call, return Ok(ControlFlow::Exit(code)) and let execute()
// handle it, the same way run_rules() already does.
```

The `execute()` and `process_input()` functions already propagate `ControlFlow::Exit(c)` cleanly; the function-call path should do the same instead of converting to an error.

---

### WR-04: `gow-diff` non-recursive directory comparison silently omits `file2` from the error message

**File:** `crates/gow-diff/src/lib.rs:85-89`

**Issue:** When two directories are provided without `-r`, the error message only mentions `file1`, discarding `file2`. GNU diff emits `"diff: file2: Is a directory"` in this case (it reports the second argument). The current code reports the first argument only, making the message misleading when the first argument is a file and only the second is a directory (a case that also reaches this error through the `else` branch at line 90 indirectly, but the branch at 81 only fires if both are directories).

```rust
// current
anyhow::bail!(
    "diff: {}: Is a directory",
    args.file1.display()
);
```

**Fix:**
```rust
anyhow::bail!(
    "diff: {}: Is a directory\ndiff: {}: Is a directory",
    args.file1.display(),
    args.file2.display()
);
```

---

## Info

### IN-01: `local_arrays` declared `mut` but never mutated before use

**File:** `crates/gow-awk/src/lib.rs:2289`

**Issue:** `let mut local_arrays: HashMap<...> = HashMap::new();` is declared with `mut` but the variable is only pushed onto the stack immediately after — never mutated in between. The compiler will emit an `unused_mut` warning.

**Fix:**
```rust
let local_arrays: HashMap<String, HashMap<String, Value>> = HashMap::new();
```

---

### IN-02: `gow-patch` `strip_path` returns original path when strip count exceeds components

**File:** `crates/gow-patch/src/lib.rs:77-83`

**Issue:** The comment says "will likely fail to find the file, which is the correct behavior," which is a reasonable trade-off, but GNU `patch` exits with an error in this case rather than silently trying an unmodified path. Currently a user who passes `-p5` on a two-component path gets a confusing "No such file" instead of "can't determine target filename."

This is a behavior gap, not a crash, but worth tracking for future GNU compatibility work.

**Fix:** Add an explicit diagnostic:
```rust
if sep_count < strip {
    // Not enough path components — this will yield a confusing error downstream.
    // Consider: return Err(anyhow!("patch: can only strip {} components from '{}'", sep_count, path));
    path
}
```

---

### IN-03: `diffy` appears in both `[dependencies]` and `[dev-dependencies]` of `gow-patch`

**File:** `crates/gow-patch/Cargo.toml:24` and `33`

**Issue:** `diffy` is listed under both `[dependencies]` (line 24) and `[dev-dependencies]` (line 33). The `[dev-dependencies]` entry is redundant because dev code automatically has access to production dependencies. The duplicate entry causes a harmless warning from Cargo but is dead configuration.

**Fix:** Remove the `diffy` entry from `[dev-dependencies]`.

```toml
# Remove this line from [dev-dependencies]:
diffy = { workspace = true }
```

---

_Reviewed: 2026-04-25T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
