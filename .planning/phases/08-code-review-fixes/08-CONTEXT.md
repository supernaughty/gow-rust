# Phase 08: Code Review Fixes — Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Apply all 7 code review findings (WR-01 through WR-07) from the Phase 06 review to
gow-tar, gow-xz, gow-gzip, and gow-curl. Also include IN-01 (gzip stdin dead code
simplification). IN-02 and IN-03 are deferred to a future phase.

Every fix has a precise code-level description in `06-REVIEW.md`. This phase does NOT
add new features or flags — it corrects existing behavior to match GNU semantics.

</domain>

<decisions>
## Implementation Decisions

### Plan Structure
- **D-01:** 4 plans, one per crate:
  - `08-01-PLAN.md` — gow-tar: WR-01 (MultiBzDecoder), WR-02 (.unwrap() → graceful error), WR-03 (non-zero exit on per-entry errors)
  - `08-02-PLAN.md` — gow-xz: WR-04 (XzDecoder::new_multi_decoder)
  - `08-03-PLAN.md` — gow-gzip: WR-05 (reject files without .gz suffix) + IN-01 (stdin dead code simplification)
  - `08-04-PLAN.md` — gow-curl: WR-06 (suppress headers in silent mode), WR-07 (remove partial file on I/O error)

### Test Fixture Strategy
- **D-02:** Multi-stream test data for WR-01 (bzip2) and WR-04 (xz) must be generated inline
  in Rust test code using the `bzip2` and `liblzma` crates. No binary fixture files committed
  to git, no external tool dependencies. This matches the pattern of all prior phases where
  temp files are constructed programmatically via `tempfile` + crate encoders.

### Info Items
- **D-03:** IN-01 included — simplify the unreachable stdin error branch in gow-gzip alongside WR-05 in plan 08-03.
- **D-04:** IN-02 (tar .tar.xz / -J flag) deferred — it is a new feature, not a bug fix.
- **D-05:** IN-03 (bzip2 -1..-9 compression levels) deferred — new feature, belongs in a later wave.

### Existing Error Handling Pattern
- **D-06:** Symlink extraction errors in tar (WR-03) remain warnings, not fatal errors.
  The fix tracks `had_error` only for non-symlink failures. Symlink failures on Windows
  require elevated privileges and should not abort extraction of the rest of the archive.

### Claude's Discretion
- Integration test count and exact fixture construction — match density of prior phases
  (typically 8–15 integration tests per plan covering happy path + error cases).
- Exit code mapping for each fix is fully specified by GNU semantics: exit 1 for runtime
  errors, exit 2 for argument misuse.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Fix Specifications (primary source of truth)
- `.planning/phases/06-archive-compression-and-network/06-REVIEW.md` — Exact code-level fix instructions for WR-01 through WR-07 and IN-01 through IN-03. Every plan must implement the fix exactly as described here.

### Source Files to Modify
- `crates/gow-tar/src/lib.rs` — WR-01 (BzDecoder → MultiBzDecoder), WR-02 (.unwrap() panic), WR-03 (unpack_archive had_error)
- `crates/gow-xz/src/lib.rs` — WR-04 (XzDecoder::new → new_multi_decoder)
- `crates/gow-gzip/src/lib.rs` — WR-05 (reject no-.gz suffix), IN-01 (stdin dead code)
- `crates/gow-curl/src/lib.rs` — WR-06 (header silent guard), WR-07 (partial file cleanup)

### Test Files (extend, don't replace)
- `crates/gow-tar/tests/tar_tests.rs`
- `crates/gow-xz/tests/xz_tests.rs`
- `crates/gow-gzip/tests/gzip_tests.rs`
- `crates/gow-curl/tests/curl_tests.rs`

### Project Context
- `.planning/REQUIREMENTS.md` §FIX-01 through §FIX-07 — Requirement definitions
- `.planning/PROJECT.md` — Core constraints (GNU compatibility, MSVC toolchain, UTF-8)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `gow_core::init()` — Called at the top of every `uumain`; already present in all 4 crates
- `gow_core::path::try_convert_msys_path()` — Path conversion; already applied in all 4 crates
- `tempfile` crate — Already in workspace dependencies; use for test temp dirs
- `bzip2::read::MultiBzDecoder` — Already available via the `bzip2` workspace dep (used in gow-bzip2 correctly); just needs to be imported in gow-tar

### Established Patterns
- **Graceful CLI error:** `match Cli::from_arg_matches(&matches) { Ok(c) => c, Err(e) => { eprintln!("{}:  {e}", name); return 2; } }` — used by every other utility
- **Partial file cleanup:** Create file → stream → on error: `let _ = std::fs::remove_file(&path); return Err(...)` — used by gow-bzip2, gow-gzip, gow-xz already
- **Unknown suffix rejection:** `eprintln!("gzip: {name}: unknown suffix -- ignored"); exit_code = 1; continue;` — pattern already in gow-bzip2 and gow-xz, now needed in gow-gzip

### Integration Points
- All 4 crates are independent; fixes are self-contained within each crate's lib.rs
- No cross-crate changes needed
- CI runs `cargo test --workspace` — all new tests are automatically picked up

</code_context>

<specifics>
## Specific Ideas

- For WR-01 multi-stream bzip2 test: write two separate bzip2 streams into a single buffer
  using `BzEncoder::finish()` twice, wrap in a `tar::Builder`, then verify `tar xjf` extracts
  both entries. This is the canonical way to prove MultiBzDecoder is needed.
- For WR-04 concatenated xz test: write two XzEncoder streams back-to-back, decompress with
  the fixed `new_multi_decoder`, verify both streams' data is present in the output.
- WR-03 symlink caveat: the `had_error` flag should only be set for non-symlink/non-privilege
  errors, so regular users on Windows don't see false exit code failures from symlink skips.

</specifics>

<deferred>
## Deferred Ideas

- **IN-02: tar .tar.xz support** — Add `-J`/`--xz` flag + liblzma dep to gow-tar. New feature; belongs in a new utilities wave or dedicated phase.
- **IN-03: bzip2 compression levels** — Add `-1` through `-9` flags to gow-bzip2. New feature; belongs in a later phase.

</deferred>

---

*Phase: 08-code-review-fixes*
*Context gathered: 2026-04-29*
