# Phase 02 — Deferred Items

Out-of-scope issues discovered during execution. Tracked here so they are not lost but are also not silently fixed in a plan that does not own them.

## Clippy — `explicit_into_iter_loop` in `gow-core::args` tests

- **Discovered by:** Plan 02-03 executor (gow-echo)
- **Command:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Errors (2):**
  - `crates/gow-core/src/args.rs`: two test cases use `["..."].map(OsString::from).into_iter()` which clippy (>= 1.95) rejects because `parse_gnu` accepts `IntoIterator` directly — the `.into_iter()` call is redundant.
- **Why deferred:** Plan 02-03's scope is `crates/gow-echo/*` only. Touching `gow-core` falls outside the worktree's lane and could conflict with any parallel Wave 2 agent.
- **Suggested fix:** Drop the `.into_iter()` calls in `crates/gow-core/src/args.rs` test-module so the expressions type as `IntoIterator<Item = OsString>` directly. Fix is ~2 line edit; takes < 1 minute.
- **Owner:** Phase 1 follow-up or any Wave 3 plan that already needs to touch gow-core.

*Pre-existing — unrelated to gow-echo implementation.*
