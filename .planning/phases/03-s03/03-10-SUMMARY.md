---
phase: "03"
plan: "10"
---

# T10: Implement gow-ln with shared create_link logic, junction fallback, and 12 integration tests.

**Implement gow-ln with shared create_link logic, junction fallback, and 12 integration tests.**

## What Happened

Implemented gow-ln with full GNU argument compatibility and Windows junction fallback. The implementation uses gow_core::fs::create_link for the core linking logic, ensuring that directory symlinks automatically fall back to junctions if SeCreateSymbolicLinkPrivilege is missing. Added 12 integration tests covering various modes (1-arg, 2-arg, multiple targets, -t, -T, -n, -f, -v) and verified they pass on Windows. Fixed a specific 'Access Denied' issue when replacing junctions by using fs::remove_dir instead of fs::remove_file for reparse points.

## Verification

Ran cargo test -p gow-ln --all-targets which executed 12 integration tests covering all major functional requirements including junction fallback and no-dereference modes.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test -p gow-ln --all-targets` | 0 | ✅ pass | 920ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
