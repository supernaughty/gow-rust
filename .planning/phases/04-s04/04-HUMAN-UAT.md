---
status: partial
phase: 04-s04
source: [04-VERIFICATION.md]
started: 2026-04-25T14:00:00Z
updated: 2026-04-25T14:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. grep --color=always emits ANSI escape codes

**Test:** Run `grep --color=always world <<< "hello world rust"` (or via cargo run) and inspect raw bytes of stdout for ANSI escape sequences.

**Expected:** Output contains `\x1b[1;31m` (or equivalent red+bold ANSI sequence) around "world". The bytes `0x1b 0x5b 0x31 0x3b 0x33 0x31 0x6d` (ESC[1;31m) should appear before "world" in the output.

**How to test (Windows PowerShell):**
```powershell
# Build first
cargo build -p gow-grep

# Run and inspect bytes
$out = & .\target\debug\grep.exe --color=always 'world' | Format-Hex
$out  # look for 0x1B bytes (ESC) in the output

# Or pipe to hexdump via WSL:
echo "hello world rust" | .\target\debug\grep.exe --color=always world | xxd | head -5
```

**Why human:** All 12 integration tests use `--color=never`. The code path at `crates/gow-grep/src/lib.rs:263-272` calls `stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))` when `supports_color()` returns true (which it always does for `ColorChoice::Always`). The code appears correct but no automated test captures the raw escape bytes.

**Result:** [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
