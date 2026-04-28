---
phase: "03"
plan: "08"
---

# T08: Implement gow-rm with recursive removal, drive-root safety, and read-only file handling.

**Implement gow-rm with recursive removal, drive-root safety, and read-only file handling.**

## What Happened

Implemented `gow-rm` with support for `-r`, `-f`, `-i`, `-v`, and `--preserve-root`.
The implementation uses `walkdir` with `contents_first(true)` to ensure that directory contents are removed before the directory itself, avoiding common pitfalls with recursive removal.
Safety is enforced by `gow_core::fs::is_drive_root`, which prevents accidental removal of Windows drive roots when `--preserve-root` is enabled (default).
Read-only files and directories are handled correctly by prompting the user (unless `-f` is used) and clearing the read-only attribute before deletion on Windows.
The CLI arguments are handled via `clap`, with a custom `preserve_root()` method to manage the interaction between `--preserve-root` and `--no-preserve-root` flags.
18 integration tests cover various scenarios including nested directories, write-protected files, interactive prompts, and drive-root rejection.

## Verification

Ran 18 integration tests covering all required features. All tests passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-rm` | 0 | ✅ pass | 1000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
