---
phase: 06
slug: archive-compression-and-network
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-28
---

# Phase 06 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) |
| **Config file** | Cargo.toml workspace |
| **Quick run command** | `cargo test -p gow-gzip -p gow-bzip2 -p gow-xz -p gow-tar -p gow-curl` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <relevant-crate>`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | R018/R019/R020 | T-06-01 | Workspace compiles cleanly | build | `cargo build --workspace` | ✅ W1 | ⬜ pending |
| 06-02-01 | 02 | 2 | R019 | T-06-02 | gzip round-trip | integration | `cargo test -p gow-gzip` | ✅ W2 | ⬜ pending |
| 06-03-01 | 03 | 2 | R019 | T-06-03 | bzip2 round-trip | integration | `cargo test -p gow-bzip2` | ✅ W2 | ⬜ pending |
| 06-04-01 | 04 | 2 | R019 | T-06-04 | xz round-trip | integration | `cargo test -p gow-xz` | ✅ W2 | ⬜ pending |
| 06-05-01 | 05 | 2 | R018 | T-06-05 | tar create/extract round-trip | integration | `cargo test -p gow-tar` | ✅ W2 | ⬜ pending |
| 06-06-01 | 06 | 3 | R020 | T-06-06 | curl HTTPS request | integration | `cargo test -p gow-curl` | ✅ W3 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements (cargo test framework already in place).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| curl proxy auth with real proxy server | R020 | Requires a live proxy server | Run `curl -x http://proxy:port http://example.com` and verify response |
| tar preserving symlinks on Windows | R018 | Requires admin privilege for CreateSymbolicLinkW | Create symlink, tar.gz it, extract, verify symlink target preserved |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
