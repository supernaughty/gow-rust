# Phase 08: Code Review Fixes — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-29
**Phase:** 08-code-review-fixes
**Areas discussed:** Plan grouping, Test fixtures, Info items scope

---

## Plan Grouping

| Option | Description | Selected |
|--------|-------------|----------|
| Per-crate — 4 plans | tar (WR-01/02/03) + xz (WR-04) + gzip (WR-05) + curl (WR-06/07). One plan per crate for clean rollback and review | ✓ |
| One combined plan | All 7 fixes in a single plan. Faster but harder to diagnose failures | |
| Two plans (compression + network) | WR-01/02/03/04/05 in one plan, WR-06/07 in another | |

**User's choice:** Per-crate — 4 plans
**Notes:** Clean crate-level isolation; aligns with prior phase structure.

---

## Test Fixtures

| Option | Description | Selected |
|--------|-------------|----------|
| Generate inline in Rust | Use bzip2/liblzma crates to build multi-stream fixtures in test code. No binary blobs, no external tools. | ✓ |
| Commit binary fixture files | Add pre-built .tar.bz2 / .xz files under tests/fixtures/. Simple but adds binary blobs to git. | |
| Generate via shell commands | Call system bzip2/xz in test setup. Fragile on Windows CI. | |

**User's choice:** Generate inline in Rust
**Notes:** Consistent with how all prior phases create test data via tempfile + crate encoders.

---

## Info Items Scope

| Option | Description | Selected |
|--------|-------------|----------|
| IN-01: gzip dead code | Simplify unreachable stdin error branch in gow-gzip. ~5 lines, same behavior. | ✓ |
| IN-02: tar .tar.xz support | Add -J/--xz flag to gow-tar. New feature — deferred. | |
| IN-03: bzip2 compression levels | Add -1 through -9 flags to gow-bzip2. New feature — deferred. | |

**User's choice:** IN-01 included; IN-02 and IN-03 deferred
**Notes:** IN-01 is a trivial cleanup that should accompany the WR-05 gzip plan. IN-02/03 are new capabilities.

---

## Claude's Discretion

- Exact test count and fixture construction details
- Exit code mapping (fully specified by GNU semantics)

## Deferred Ideas

- IN-02: tar .tar.xz support (-J flag) — future phase
- IN-03: bzip2 -1..-9 compression levels — future phase
