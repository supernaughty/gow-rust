---
phase: 11
slug: new-utilities-wave2
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-29
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | assert_cmd 2.2.1 + predicates 3.1.4 + tempfile 3.27.0 |
| **Config file** | none — standard `cargo test` discovers tests/integration.rs per crate |
| **Quick run command** | `cargo test -p gow-test -p gow-expr` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <crate-being-implemented>`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | U2-01..U2-10 | — | N/A | integration | `cargo build --workspace` | ❌ W0 | ⬜ pending |
| 11-02-01 | 02 | 2 | U2-10 | — | unlink refuses dirs | integration | `cargo test -p gow-unlink` | ❌ W0 | ⬜ pending |
| 11-02-02 | 02 | 2 | U2-09 | — | N/A | integration | `cargo test -p gow-fmt` | ❌ W0 | ⬜ pending |
| 11-02-03 | 02 | 2 | U2-03 | — | N/A | integration | `cargo test -p gow-paste` | ❌ W0 | ⬜ pending |
| 11-03-01 | 03 | 2 | U2-04 | — | N/A | integration | `cargo test -p gow-join` | ❌ W0 | ⬜ pending |
| 11-03-02 | 03 | 2 | U2-05 | — | split validates -b/-l/-n > 0 | integration | `cargo test -p gow-split` | ❌ W0 | ⬜ pending |
| 11-04-01 | 04 | 2 | U2-06 | — | N/A | integration | `cargo test -p gow-printf` | ❌ W0 | ⬜ pending |
| 11-04-02 | 04 | 2 | U2-07 | T-11-04-01 | expr limits recursion depth | integration | `cargo test -p gow-expr` | ❌ W0 | ⬜ pending |
| 11-05-01 | 05 | 2 | U2-08 | — | test -f on device paths returns graceful result | integration | `cargo test -p gow-test` | ❌ W0 | ⬜ pending |
| 11-06-01 | 06 | 3 | U2-01 | — | N/A | integration | `cargo test -p gow-whoami` | ❌ W0 | ⬜ pending |
| 11-06-02 | 06 | 3 | U2-02 | — | uname uses RtlGetVersion not GetVersionExW | integration | `cargo test -p gow-uname` | ❌ W0 | ⬜ pending |
| 11-06-03 | 06 | 3 | U2-01..U2-10 | — | N/A | integration | `cargo test --workspace` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/gow-whoami/tests/integration.rs` — scaffold_compiles placeholder for U2-01
- [ ] `crates/gow-uname/tests/integration.rs` — scaffold_compiles placeholder for U2-02
- [ ] `crates/gow-paste/tests/integration.rs` — scaffold_compiles placeholder for U2-03
- [ ] `crates/gow-join/tests/integration.rs` — scaffold_compiles placeholder for U2-04
- [ ] `crates/gow-split/tests/integration.rs` — scaffold_compiles placeholder for U2-05
- [ ] `crates/gow-printf/tests/integration.rs` — scaffold_compiles placeholder for U2-06
- [ ] `crates/gow-expr/tests/integration.rs` — scaffold_compiles placeholder for U2-07
- [ ] `crates/gow-test/tests/integration.rs` — scaffold_compiles placeholder for U2-08
- [ ] `crates/gow-fmt/tests/integration.rs` — scaffold_compiles placeholder for U2-09
- [ ] `crates/gow-unlink/tests/integration.rs` — scaffold_compiles placeholder for U2-10

All 10 created in scaffold plan 11-01.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `[` bracket shim works in cmd.exe | U2-08 | Requires interactive shell; batch dispatch only testable at runtime | After MSI install: `[ -f C:\Windows\notepad.exe ] && echo yes` in cmd.exe |
| `uname -a` output format on live Windows 10/11 | U2-02 | RtlGetVersion returns real build number at runtime | Run `uname -a`; verify format matches `Windows_NT HOSTNAME MAJOR.MINOR.BUILD 0 ARCH ARCH` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
