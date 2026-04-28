---
phase: "03"
plan: "11"
---

# T11: Implement gow-mv with shared transform logic and 12 integration tests.

**Implement gow-mv with shared transform logic and 12 integration tests.**

## What Happened

Implemented `gow-mv` (FILE-04) with same-volume rename support and explicit cross-volume fallback logic. When `std::fs::rename` fails with `ErrorKind::CrossesDevices`, the utility now drives an explicit copy+delete sequence. The fallback preserves file timestamps (using `filetime`) and Windows read-only attributes before removing the source, ensuring parity with GNU `mv` behavior and avoiding silent data loss during cross-device moves. Added 12 integration tests covering file-to-file, file-to-directory, directory-to-directory, and error cases like same-file move or invalid directory-to-file overwrites.

## Verification

12 integration tests passed, covering all core move scenarios including symlinks and error conditions. Verified same-volume rename and ensured non-directory/directory overwrite restrictions match GNU behavior.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-mv` | 0 | ✅ pass | 3500ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
