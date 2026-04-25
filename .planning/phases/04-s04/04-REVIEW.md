---
phase: 04-s04
reviewed: 2026-04-25T00:00:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - crates/gow-sort/src/lib.rs
  - crates/gow-sort/tests/integration.rs
  - crates/gow-sed/src/lib.rs
  - crates/gow-sed/tests/sed_test.rs
findings:
  critical: 3
  warning: 4
  info: 3
  total: 10
status: issues_found
---

# Phase 04-s04: Code Review Report

**Reviewed:** 2026-04-25T00:00:00Z
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

Reviewed `gow-sort` (key-field `-k` support and field extraction) and `gow-sed` (address ranges and the `d`/`p`/`q` commands). Both crates have functional cores for basic usage, but each contains multiple correctness bugs that trigger on common real-world inputs.

The most severe issues are: (1) `gow-sed` range addresses using a regex end-pattern incorrectly close the range on the activating line itself whenever that line is not line 1 — a frequent pattern like `/start/,/end/d` produces wrong output on any input where the start is not on line 1; (2) expressions are split on `;` before any parsing, so `s` commands whose pattern or replacement contain a semicolon are silently fragmented; (3) `gow-sort`'s `parse_single_key` finds the first alphabetic character in the whole keydef string to locate modifier start, which breaks per-field modifiers like `-k 1n,2r`.

---

## Critical Issues

### CR-01: Regex end-condition of a range fires on the activating line for lines after line 1

**File:** `crates/gow-sed/src/lib.rs:442`

**Issue:** For a range address where the end is a regex pattern (e.g., `1,3d` is fine, but `/start/,/end/d` is affected), the guard preventing the start-trigger line from immediately closing the range is `line_num > 1`. This tests the global line number, not whether the current line is the one that just opened the range.

Consequence: whenever a regex-end range is activated on any line with `line_num > 1`, the end check runs immediately on the same iteration. If the end regex also matches that line, the range is closed before any subsequent lines are processed.

Concrete example: `sed '/foo/,/bar/d'` on input `["a", "foo", "bar", "baz"]`. Line 2 "foo" activates the range; the exit check evaluates `line_num > 1` (true) and `re.is_match("foo")` where `re` is `/bar/` (false here, so this particular example works). But with `sed '/foo/,/foo/d'` on `["a", "foo", "b", "foo", "c"]`: line 2 activates the range; exit check is `line_num > 1 && /foo/.is_match("foo")` = true; range is immediately deactivated. Lines 3 and 4 are not deleted. GNU sed would delete lines 2–4.

**Fix:** Track whether the current line is the activating line for each command, independently of its absolute line number:

```rust
// Add alongside range_active:
let mut range_just_entered = vec![false; commands.len()];

// In the Range match arm:
if !range_active[cmd_idx] {
    let enters = match start { /* ... */ };
    if enters {
        range_active[cmd_idx] = true;
        range_just_entered[cmd_idx] = true;  // mark activation line
    }
} else {
    range_just_entered[cmd_idx] = false;     // clear after activation line
}

if range_active[cmd_idx] {
    let exits = match end {
        Address::Line(n) => line_num >= *n,
        Address::Last => line_num == total_lines,
        // Only check end regex when NOT on the activating line
        Address::Pattern(re) => !range_just_entered[cmd_idx] && re.is_match(&line_str),
    };
    if exits { range_active[cmd_idx] = false; }
    true
} else {
    false
}
```

---

### CR-02: Semicolon splitting of expressions breaks `s` commands containing `;`

**File:** `crates/gow-sed/src/lib.rs:102`

**Issue:** All expression strings are split on `;` (and `\n`) before any parsing:

```rust
for part in expr.split(|c| c == ';' || c == '\n') {
```

This means `sed 's/a;b/c/'` is split into `"s/a"` and `"b/c/"`. The first fragment (`"s/a"`) lacks the required closing delimiters and produces an error `"invalid substitution command"`. The user's intended substitution is never executed.

Any `s` command whose regex or replacement naturally contains a semicolon is affected. This is a common real-world pattern (e.g., removing semicolons: `sed 's/;//g'`).

**Fix:** Move the semicolon split inside the command parser, after the command letter has been identified and its delimiters consumed. The simplest approach is to make `parse_command` return both the parsed command and the unconsumed tail, then chain:

```rust
// Conceptual fix — parse_command_consuming returns (SedCommand, remaining &str)
let mut remaining = expr.as_str();
while !remaining.is_empty() {
    remaining = remaining.trim_start_matches(|c| c == ';' || c == '\n');
    if remaining.is_empty() { break; }
    let (cmd, rest) = parse_command_consuming(remaining)?;
    commands.push(cmd);
    remaining = rest;
}
```

For `s` commands specifically, `parse_command_consuming` must consume the full `s/pat/repl/flags` without treating the content's semicolons as separators.

---

### CR-03: `parse_single_key` modifier detection breaks per-field modifiers

**File:** `crates/gow-sort/src/lib.rs:103`

**Issue:** The modifier position is found by:

```rust
let modifier_start = keydef
    .find(|c: char| c.is_alphabetic())
    .unwrap_or(keydef.len());
```

This returns the index of the **first** alphabetic character in the entire keydef string. For a keydef like `"1n,2r"` (field 1 numeric, field 2 reverse — a valid GNU sort KEYDEF with per-field modifiers), `modifier_start` is 1 (`n`), so `numeric_part = "1"` and `modifiers = "n,2r"`. The comma and `"2r"` are consumed into `modifiers` instead of being parsed as a range end, so `end_field` is never set and the range `1,2` is silently dropped.

**Fix:** Modifiers are a trailing alphabetic suffix; scan from the right, stopping at the last non-alphabetic character:

```rust
fn parse_single_key(keydef: &str) -> KeySpec {
    // Modifiers are the trailing alphabetic suffix after all digits and commas
    let modifier_start = keydef
        .rfind(|c: char| !c.is_alphabetic())
        .map(|i| i + 1)   // one past the last non-alpha
        .unwrap_or(0);    // entire string is alpha (no numeric part)
    let modifiers = &keydef[modifier_start..];
    let numeric_part = &keydef[..modifier_start];
    // ... rest unchanged
}
```

---

## Warnings

### WR-01: `\n` in sed replacement does not produce a newline

**File:** `crates/gow-sed/src/lib.rs:304`

**Issue:** In GNU sed, `s/x/\n/` replaces `x` with a literal newline character. In `parse_s_command_inner`, when the replacement string contains `\n`, the `\\` is pushed to `replacement` as a literal backslash (at the point where `escaped` is false and `c == '\\'`), then on the next iteration `n` is pushed as an escaped character. The resulting replacement passed to `regex::replace` is the two-character sequence `\` + `n`. The `regex` crate does not interpret `\n` as a newline in replacements, so the output contains a literal backslash-n instead of a newline.

**Fix:** Expand `\n` (and `\t`) in the replacement conversion loop:

```rust
'\\' => {
    if let Some(&next) = r_chars.peek() {
        if next.is_ascii_digit() {
            regex_replacement.push('$');
            regex_replacement.push(r_chars.next().unwrap());
        } else if next == 'n' {
            r_chars.next();
            regex_replacement.push('\n');  // actual newline
        } else if next == 't' {
            r_chars.next();
            regex_replacement.push('\t');  // actual tab
        } else if next == '\\' || next == delimiter || next == '&' {
            regex_replacement.push(r_chars.next().unwrap());
        } else {
            regex_replacement.push('\\');
            regex_replacement.push(r_chars.next().unwrap());
        }
    } else {
        regex_replacement.push('\\');
    }
}
```

---

### WR-02: Malformed line-number address silently becomes a no-op instead of an error

**File:** `crates/gow-sed/src/lib.rs:210`

**Issue:** `s[..end].parse::<usize>().unwrap_or(0)` defaults to line number 0 on parse failure. Since lines are 1-based, `Address::Line(0)` never matches any line. A user who types `sed '0d'` or mistakenly writes a non-numeric address that starts with digits receives no error and sees all input pass through unchanged.

The same pattern at line 231 in `parse_single_address` uses `unwrap_or(1)`, defaulting to line 1 instead.

**Fix:** Propagate a parse error:

```rust
// parse_address, line ~210:
let line_num: usize = s[..end].parse()
    .map_err(|_| anyhow::anyhow!("invalid address '{}': not a line number", &s[..end]))?;
if line_num == 0 {
    return Err(anyhow::anyhow!("invalid usage of line address 0"));
}
```

---

### WR-03: `parse_single_key` silently defaults on non-numeric field argument

**File:** `crates/gow-sort/src/lib.rs:115`

**Issue:** `start_str.parse::<usize>().unwrap_or(1).max(1)` defaults to field 1 when the field string is non-numeric (e.g., `sort -k abc file` silently becomes `sort -k 1`). GNU sort emits `sort: invalid field specification 'abc'` and exits with status 2. The silent default masks user errors and produces wrong sort order.

**Fix:** Return an error from `parse_single_key`:

```rust
fn parse_single_key(keydef: &str) -> Result<KeySpec, String> {
    // ...
    let start_field = start_str.parse::<usize>()
        .map_err(|_| format!("invalid field specification '{}'", keydef))?;
    if start_field == 0 {
        return Err(format!("invalid field specification '{}': fields are 1-based", keydef));
    }
    // ...
}
```

`parse_key_specs` and `uumain` must be updated to propagate the error.

---

### WR-04: Inverted key range (`-k 3,1`) silently produces empty comparison key

**File:** `crates/gow-sort/src/lib.rs:151`

**Issue:** When `key.end_field` is `Some(m)` with `m < key.start_field` (e.g., `-k 3,1`), the code computes `end_idx < start_idx` and returns `Vec::new()` at line 153. All such lines compare equal on that key, producing a silently wrong sort order. GNU sort rejects this with `sort: disorder in -k key: 3,1`.

**Fix:** Validate the range in `parse_single_key`:

```rust
if let Some(end) = end_field {
    if end < start_field {
        // For now, clamp silently or — better — propagate an error:
        // return Err(format!("disorder in -k key: {}", keydef));
        end_field = Some(start_field); // minimum: treat as single-field
    }
}
```

The preferred fix is to return an error to match GNU sort behavior.

---

## Info

### IN-01: `test_unique_sort` unit test is an empty placeholder

**File:** `crates/gow-sort/src/lib.rs:499`

**Issue:** The test body contains only a comment and no assertions. The `write_sorted` deduplication path (the `last_line` clone and `compare_lines` equality check) has no unit-level coverage for edge cases such as unique-with-key-sort or unique-with-numeric-sort.

**Fix:** Add at minimum one assertion, or remove the empty test:

```rust
#[test]
fn test_unique_sort() {
    let config = SortConfig {
        unique: true, numeric: false, reverse: false,
        ignore_case: false, buffer_size: 0,
        keys: vec![], field_separator: None,
    };
    let lines = vec![b"b".to_vec(), b"a".to_vec(), b"a".to_vec()];
    let mut out = Vec::new();
    write_sorted(lines, &config, Box::new(&mut out)).unwrap();
    assert_eq!(out, b"a\nb\n");
}
```

---

### IN-02: Non-UTF-8 bytes silently replaced with U+FFFD during sed line processing

**File:** `crates/gow-sed/src/lib.rs:411`

**Issue:** `String::from_utf8_lossy(line).into_owned()` replaces invalid UTF-8 sequences with the Unicode replacement character (U+FFFD). GNU sed processes arbitrary byte streams. Files in Windows codepages (e.g., CP1252) with bytes above 0x7F will have those bytes silently corrupted in the output.

The `bstr` crate is already a dependency and provides byte-string regex operations. Lines could remain as `&[u8]` / `BString` throughout, converting to `str` only for the `regex` crate's `is_match`/`replace` calls via `std::str::from_utf8` with a fallback.

**Fix:** This is an architectural change; track as a known limitation and add a comment noting the UTF-8 requirement, or migrate line processing to `BString`.

---

### IN-03: `test_sort_ignore_case` uses loose presence checks instead of verifying sort order

**File:** `crates/gow-sort/tests/integration.rs:69`

**Issue:** The test only asserts that each of the four input characters appears somewhere in the output. It does not assert that the output is in sorted order. A bug that emitted lines in arbitrary order would still pass.

**Fix:** Assert the full output string. Since `sort_by` is stable, equal elements (same case-folded value) preserve input order; with input `"b\nA\nB\na\n"` the expected case-fold-sorted output is `"A\na\nB\nb\n"` or `"a\nA\nb\nB\n"` depending on input order preservation for ties:

```rust
// Verify that a's come before b's, regardless of A/a ordering:
let out = std::str::from_utf8(&cmd.output().unwrap().stdout).unwrap().to_string();
let first_b_pos = out.find(|c| c == 'b' || c == 'B').unwrap_or(usize::MAX);
let last_a_pos = out.rfind(|c| c == 'a' || c == 'A').unwrap_or(0);
assert!(last_a_pos < first_b_pos, "a's should all precede b's in case-insensitive sort");
```

---

_Reviewed: 2026-04-25T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
