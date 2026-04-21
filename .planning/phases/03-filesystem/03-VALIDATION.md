---
phase: 3
slug: filesystem
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-21
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Expands on RESEARCH.md §Validation Architecture (lines 1568+).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + `assert_cmd 2` + `snapbox 1.2` + `predicates 3` + `tempfile 3` |
| **Config file** | workspace `Cargo.toml` (no separate test config) |
| **Quick run command** | `cargo test -p gow-{crate-name}` (per-crate — fast iteration) |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` |
| **Estimated runtime** | ~30-60 seconds (Phase 2 baseline: 265 tests in ~45s; Phase 3 adds ~300+ tests → ~90s estimate) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p gow-{crate-name}` (per-crate, ~2-5s)
- **After every plan wave:** Run `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite green + all 5 ROADMAP success criteria demonstrated via integration test (see RESEARCH.md §Validation Architecture)
- **Max feedback latency:** 5s per-crate, 90s full suite

---

## Per-Task Verification Map

Populated by the planner — each task in each PLAN.md gets a row. Reference columns reserved for:

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-00-01 | 00 prep | 0 | — | T-03-01 (privilege-aware tempfiles) | Tempfile created in same dir as target, no world-writable | unit | `cargo test -p gow-core fs::atomic_rewrite` | ❌ W0 | ⬜ pending |
| 03-00-02 | 00 prep | 0 | — | T-03-02 (link create privilege) | Junction fallback does not silently succeed without notice | unit | `cargo test -p gow-core fs::create_link` | ❌ W0 | ⬜ pending |
| (planner fills the rest) | | | | | | | | | |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/gow-core/src/fs.rs` — new helpers: `atomic_rewrite<F>`, `create_link(src, dst, kind) -> LinkOutcome`, `is_hidden`, `is_readonly`, `has_executable_extension`, `clear_readonly`, `is_drive_root` (RESEARCH.md §Pattern 1-4)
- [ ] `Cargo.toml` [workspace.dependencies]: add `walkdir = "2.5"`, `notify = "8.2"`, `terminal_size = "0.4"`, `junction = "1.4"` (D-50 gap per research)
- [ ] `Cargo.toml` [workspace.members]: add 11 new crate paths
- [ ] 11 new crate scaffolds (lib+bin pattern, D-16) with stub `uumain` that prints `{name}: not yet implemented` and returns 1
- [ ] `crates/gow-core/src/fs.rs` unit tests for all new helpers + privilege-skip pattern for symlink tests (reuse fs.rs:118 template)

**If none:** Existing infrastructure does NOT cover Phase 3 — Wave 0 is mandatory.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `tail -f` 200ms latency under real log rotation | TEXT-02 / ROADMAP Criterion 3 | Timing-sensitive; notify backend + OS scheduler variance makes pure CI assertions flaky. Automated test verifies event delivery + sub-500ms latency in tempdir; manual test confirms real-world log append from PowerShell parallel session. | 1. Terminal A: `tail -f test.log`. 2. Terminal B: `while ($true) { Add-Content test.log "line $([DateTime]::Now.Ticks)"; Start-Sleep -Milliseconds 100 }`. 3. Observe lines appear in Terminal A within 200ms perceptual latency. 4. Kill B, verify A still watching. 5. Rotate: Terminal B: `Move-Item test.log test.log.1; "new content" > test.log`. 6. With `-f` (descriptor), A stays on old file (logs warning). With `-F`, A picks up new file. |
| `ls --color` rendering in both ConHost and Windows Terminal | FILE-02 / ROADMAP Criterion 1 | Terminal-specific ANSI + VT100 behavior cannot be asserted via stdout capture (which strips color codes unless forced). | 1. Open legacy ConHost (cmd.exe). 2. Run `ls -la C:\Windows\System32`. 3. Verify directories are blue, symlinks cyan, .exe green. 4. Repeat in Windows Terminal — same output. 5. Pipe to file: `ls --color=always > colored.txt` and grep for `\e[` escape codes. |
| `cp -r` across drives (C:\ → D:\) with timestamp preservation | FILE-03 / ROADMAP Criterion 2 | Requires 2 physical or logical drives; CI runners typically single-drive. | 1. On dev machine with D: drive: `cp -rp C:\src-tree D:\dest-tree`. 2. `Get-Item D:\dest-tree\**\* \| select Name, LastWriteTime` matches source timestamps. 3. Read-only attributes match on files that had them. |
| Windows Developer Mode symlink creation | FILE-09 | Symlink privilege is environmental, not testable in every CI run (even if tests skip). | Manual: Enable Dev Mode in Settings → Update & Security → For Developers. Then `ln -s tgt link` should create symlink (not fallback to junction). Confirm stderr is silent (no 'junction instead' message). |

*Additional manual tests documented in RESEARCH.md §Validation Architecture.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (gow-core helpers, crate stubs, workspace deps)
- [ ] No watch-mode flags in CI commands
- [ ] Feedback latency < 5s per-crate
- [ ] `nyquist_compliant: true` set in frontmatter after planner fills per-task map
- [ ] All 5 ROADMAP success criteria have at least one automated verify OR manual test with reproducible steps

**Approval:** pending
