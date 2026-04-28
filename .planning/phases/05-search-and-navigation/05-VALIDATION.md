---
phase: 05
slug: search-and-navigation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-28
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + `assert_cmd` + `predicates` + `tempfile` |
| **Config file** | `Cargo.toml` (workspace member per crate) |
| **Quick run command** | `cargo test -p gow-find -p gow-xargs -p gow-less` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30–60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p gow-find -p gow-xargs -p gow-less`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | R015, R016, R017 | — | N/A | unit | `cargo test --workspace` | ❌ W0 | ⬜ pending |
| 05-02-xx | 02 | 2 | R015 | T-05-find | Path traversal safety | integration | `cargo test -p gow-find` | ❌ W0 | ⬜ pending |
| 05-03-xx | 03 | 2 | R016 | T-05-xargs | Binary mode stdin | integration | `cargo test -p gow-xargs` | ❌ W0 | ⬜ pending |
| 05-04-xx | 04 | 3 | R017 | T-05-less | Raw mode restore on panic | integration | `cargo test -p gow-less` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/gow-find/tests/find_tests.rs` — stubs for R015 (name, type, size, mtime predicates, exec)
- [ ] `crates/gow-xargs/tests/xargs_tests.rs` — stubs for R016 (-0, -I{}, -n, -L flags)
- [ ] `crates/gow-less/tests/less_tests.rs` — stubs for R017 (non-TTY passthrough mode)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `less` interactive scroll/search | R017 | Requires real terminal (crossterm raw mode) | Launch `less /path/to/large/file`, verify arrow keys, `/search`, `n`/`N`, `G`/`g`, `q` |
| ANSI color passthrough | R017 | Visual output validation | Run `grep --color foo file \| less`, verify colors display |
| `find -print0 \| xargs -0` pipeline | R015, R016 | Requires Windows binary mode pipe | Run in PowerShell: `find . -name "*.txt" -print0 \| xargs -0 echo` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
