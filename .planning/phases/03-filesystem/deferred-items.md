# Phase 03 Deferred Items

Out-of-scope issues observed during plan execution. These are NOT fixed by the plan that observed them — they belong to a different owner (e.g., gow-core / Wave 0) or to a follow-up hygiene pass.

## From Plan 03-03 (head)

### 1. Unused import `clap::error::ErrorKind` in `crates/gow-core/src/args.rs`

- **Observed:** Working-tree showed `+ use clap::error::ErrorKind;` at line 14 that was NOT committed to any Phase 3 plan. Triggers `-D warnings` failure on any workspace-wide clippy run.
- **Ownership:** `gow-core` is Wave 0 (plan 03-01). This edit is not part of 03-03's files.
- **Recommendation:** Either commit with a consumer (add ErrorKind match arm to parse_gnu) or revert the import. Not a gow-head concern.
- **Workaround used for 03-03 verification:** Stashed the change, ran `cargo clippy -p gow-head --all-targets -- -D warnings` clean, restored the stash.
