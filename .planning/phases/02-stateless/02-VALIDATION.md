---
phase: 02
slug: stateless
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-21
source: .planning/phases/02-stateless/02-RESEARCH.md ("## Validation Architecture" section)
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution. This file is a thin index that points at the authoritative content in `02-RESEARCH.md`'s `## Validation Architecture` section (line 1737+). The research document was written in a form that is already a complete validation strategy; duplicating it here would risk drift.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test runner + `assert_cmd` 2.2.1 + `predicates` 3 + `snapbox` 1.2 |
| **Config file** | none (cargo test built-in) |
| **Quick run command** | `cargo test -p gow-{name}` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~15 s per utility crate, ~60 s workspace cold |
| **CI platform** | GitHub Actions `windows-latest` (D-30c) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p gow-{name}` for the utility just touched.
- **After every plan wave:** Run `cargo test --workspace`.
- **Before `/gsd-verify-work`:** Full suite must be green; `cargo clippy --workspace -- -D warnings` must be clean.
- **Max feedback latency:** ~15 seconds (per-crate fast path).

---

## Validation Dimensions

Full rubric in `02-RESEARCH.md` — summarized here for discoverability:

| # | Dimension | Per-utility min tests | Source |
|---|-----------|----------------------|--------|
| 1 | GNU Compatibility (exit codes + flag surface) | 1 bad-flag → exit 1, 1 per major flag | RESEARCH.md §Validation.Dimension 1 |
| 2 | UTF-8 Correctness (wc, path args, filenames) | 1 UTF-8 fixture per I/O utility | RESEARCH.md §Validation.Dimension 2 |
| 3 | Windows-native primitives (PATHEXT, touch -h, tee -i, pwd -P UNC strip) | 1 Windows-specific integration test per primitive | RESEARCH.md §Validation.Dimension 3 |
| 4 | Error-path coverage (missing files, permission, broken pipe) | 1 error case per utility, GNU-format error message | RESEARCH.md §Validation.Dimension 4 |
| 5 | Throughput / performance (`yes`) | 1 smoke test ≥ 50 MB/s, not full benchmark | RESEARCH.md §Validation.Dimension 5 |

---

## Coverage Rule

Each locked decision from CONTEXT.md (D-16..D-30) that specifies user-observable behavior → at least one automated test. Implementation-detail decisions (e.g., D-24's PWD-vs-current_dir preference internal) do not need explicit coverage as long as the observable output passes.

The plan checker (`gsd-plan-checker`) verifies this coverage requirement when validating plans.

---

## Per-Task Verification Map

Per-task test mapping will be added to each PLAN.md task's `<acceptance_criteria>` field directly (the planner is instructed to reference specific RESEARCH.md Validation dimensions per task). This file does not replicate the per-task map — it serves as the contract that PLAN.md tasks must satisfy.

---

*Validation strategy drafted 2026-04-21. Full content in 02-RESEARCH.md §Validation Architecture.*
