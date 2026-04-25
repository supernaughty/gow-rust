# Phase 05: Search and Navigation - Context

**Gathered:** 2026-04-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement three GNU navigation utilities: `find` (file search with predicates and -exec), `xargs` (stdin-to-command-line builder), and `less` (interactive terminal pager). Each ships as an independent binary following the established `lib.rs` + `main.rs` crate pattern.

Scope covers: R015 (find), R016 (xargs), R017 (less). Archive, compression, curl, and installer work belong in Phase 06.

</domain>

<decisions>
## Implementation Decisions

### find: Name Matching

- **D-01:** `-name` is case-sensitive (strict GNU behavior). Add `-iname` for case-insensitive matching — do not make `-name` Windows-aware.
- **D-02:** Wildcard support uses standard POSIX glob: `*`, `?`, `[...]`. Use `globset` crate (add to workspace deps). No `**` recursive glob in `-name` — that is not GNU find behavior.

### find: Predicate Set

- **D-03:** Implement all four predicate groups: `-type f/d/l` (file/dir/symlink), `-size +N/-N` (with bytes/k/M/G units), `-mtime/-atime/-ctime` (days), `-maxdepth/-mindepth` (depth control).
- **D-04:** `-atime` note — Windows does not track access time by default; implement the flag but document that atime may equal mtime on NTFS without the `NtfsDisableLastAccessUpdate` registry change.

### find: -exec Behavior

- **D-05:** Support `-exec cmd {} \;` form only. The arg-accumulating `{} +` form and `-execdir` are deferred to gap-closure plans.
- **D-06:** Execute commands via `std::process::Command` (CreateProcess) — no shell intermediary. This handles paths with spaces natively (fixes GOW #209) and avoids cmd.exe quoting issues.

### less: Feature Depth

- **D-07:** Core pager feature set: arrow keys + PgUp/PgDn scroll, `q` quit, `/` forward search with `n`/`N` navigation, `G`/`g` jump to end/start. File arguments and stdin piping both supported.
- **D-08:** ANSI color passthrough enabled by default (like `less -R`). Detect ANSI escape sequences in input and render them — `grep --color | less` shows highlighted output.
- **D-09:** Streaming/buffered I/O — never load the full file into memory. Read forward lazily, buffer a sliding window. `G` (jump to end) requires a seek but must not OOM on large files.
- **D-10:** Add `crossterm` to workspace deps. Use it for raw terminal mode, cursor movement, and screen clearing. `termcolor` is not sufficient for a full interactive pager.

### xargs: Scope

- **D-11:** Serial-only execution — no `-P N` parallel mode in this phase. Defer parallel xargs to a gap-closure plan.
- **D-12:** Core flags to implement: `-0` (null-separated input, pairs with `find -print0`), `-I {}` (fixed `{}` replacement string only — no configurable `-I STR`), `-n maxargs`, `-L maxlines`.

### Claude's Discretion

- Internal buffering strategy for less (ring buffer vs. line index)
- globset vs. manual glob matching for find (globset is the right choice given it's already in the tech stack)
- Integration test structure (follow established patterns from phases 3-4)
- Whether to add `-print0` to find (needed for xargs -0 interop — yes, include it)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — R015 (find), R016 (xargs), R017 (less) — full capability specs and GOW issue references
- `.planning/PROJECT.md` — Core constraints: Rust stable, MSVC toolchain, independent binaries, UTF-8 default, GNU compatibility goal

### Established Patterns (read at least one crate for implementation pattern)
- `crates/gow-grep/` — Most recent complex crate; use as template for find (regex, walkdir, color)
- `crates/gow-awk/` — Most recent crate overall; use as template for Cargo.toml structure and lib layout
- `crates/gow-core/` — Shared init (UTF-8 console, path conversion, color detection)

### Tech Stack Reference
- `CLAUDE.md` — Workspace library versions (crossterm 0.29, walkdir 2.5, globset 0.4); add crossterm and globset to workspace Cargo.toml

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `walkdir` (workspace dep): Already used by ls, cp, rm, grep, chmod — ready for find's directory traversal
- `gow-core`: UTF-8 init, path conversion (Unix↔Windows), color detection — every new crate calls `gow_core::init()`
- `embed-manifest` build dep: Required for all new crates (Windows manifest embedding for proper UAC behavior)
- `regex` (workspace dep): Used by grep/sed/awk — available for less `/` search highlighting
- `terminal_size` (workspace dep): Used by ls — available for less viewport sizing

### Established Patterns
- Each crate: `src/main.rs` (one-liner calling `uu_<name>::uumain(std::env::args_os())`), `src/lib.rs` (full impl), `build.rs` (embed-manifest)
- Integration tests live in `tests/` within the crate, use `assert_cmd` + `predicates` + `tempfile`
- Workspace members declared in root `Cargo.toml` with a phase comment (e.g., `# Phase 5 — search and navigation (S05)`)

### Integration Points
- New crates: `gow-find`, `gow-xargs`, `gow-less` — register all three in root `Cargo.toml` `[workspace] members`
- Add to workspace deps: `crossterm = "0.29"`, `globset = "0.4"`
- `find -print0` → `xargs -0` pipeline must work end-to-end on Windows (null bytes in stdout piped to stdin)

</code_context>

<specifics>
## Specific Ideas

- `find -exec` must fix GOW issue #208 (exec support) and #209 (paths with spaces) — these are explicit requirements
- `find | xargs` integration: ensure `find -print0 | xargs -0 cmd` works as a Windows-native pipeline
- `less` should feel familiar to Linux developers — don't invent new key bindings; match standard less

</specifics>

<deferred>
## Deferred Ideas

- `find -exec cmd {} +` (arg-accumulating form) — gap-closure plan after Phase 05
- `find -execdir` — gap-closure plan after Phase 05
- `xargs -P N` (parallel execution) — gap-closure plan after Phase 05
- `xargs -I STR` (configurable replacement string) — gap-closure plan after Phase 05
- `less` line numbers (`:N` toggle), marks (`m`/`M`), `LESS` env var, `-e` auto-exit — Phase 05 gap-closure or Phase 07+

</deferred>

---

*Phase: 05-search-and-navigation*
*Context gathered: 2026-04-26*
