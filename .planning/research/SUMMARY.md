# Project Research Summary

**Project:** gow-rust (GNU On Windows -- Rust rewrite)
**Domain:** Native Windows GNU utility suite (Rust-based coreutils reimplementation)
**Researched:** 2026-04-20
**Confidence:** HIGH

## Executive Summary

gow-rust is a Windows-native reimplementation of the GNU core utilities suite using Rust, targeting the exact user pain points documented in GOW 0.8.0 issues: broken tail -f, corrupted path translation, non-functional grep --color, and decade-old binary versions. The expert approach -- validated by the uutils/coreutils project (100+ utilities, 94.74% GNU test suite pass rate) -- is a Cargo workspace with one shared platform library (gow-core) and one crate per utility, each compiled as an individual .exe. The MSVC toolchain is mandatory; MinGW introduces a separate C runtime and is explicitly not recommended by uutils for Windows distribution.

The recommended approach is to build incrementally: first establish the shared gow-core crate (UTF-8 console setup, Windows path normalization, ANSI color init, GNU argument parsing abstraction), then layer utilities in dependency order -- simple stateless tools first, filesystem tools next, text processing after, then complex tools like find/sed/curl last. The MSI installer using cargo-wix plus WiX v3 handles PATH registration. Three differentiators justify the rewrite over installing uutils directly: a curated Windows-native selection, a single installable MSI package, and explicit fixes for the GOW issues that made the original unusable.

The primary risks are all Windows-specific: clap argument parsing semantics differ from GNU getopt in exit-code-breaking ways; Windows path separator handling has a documented history of corrupting CLI flags; the console codepage defaults to legacy encodings that produce mojibake; and Windows file locking semantics break in-place editing tools. All of these are preventable by centralizing the shared platform concerns in gow-core before building any individual utility -- if this foundation is skipped or done poorly, the bugs replicate across every tool.

## Key Findings

### Recommended Stack

The stack is anchored around stable Rust 1.85+ with the x86_64-pc-windows-msvc target. clap 4 with the derive API handles argument parsing, but requires a GNU compatibility shim to fix exit code mismatches and option permutation behavior. thiserror in library crates and anyhow at binary entry points covers error handling. windows-sys 0.61+ (not the heavier windows crate) provides Win32 API bindings with faster compile times. Key domain crates are notify for tail -f, termcolor for color output (not crossterm, reserved for the less pager), bstr for non-UTF-8 byte streams, and regex for all pattern matching. The test stack is assert_cmd + predicates + snapbox for GNU compatibility snapshot tests.

**Core technologies:**
- clap 4.6.1 (derive API): Argument parsing -- industry standard, needs a GNU compat wrapper for exit codes and option permutation
- thiserror 2.0.18 / anyhow 1.0.102: Error handling -- thiserror in libs for typed errors; anyhow only at main() entry
- windows-sys 0.61.2: Win32 API bindings -- raw-dylib, zero overhead, faster compile than windows crate
- notify 8.2.0: Filesystem watching -- ReadDirectoryChangesW backend, fixes GOW tail -f issues #75/#169/#89
- termcolor 1.4.1: Color output -- handles ANSI escapes and Windows Console API, fixes GOW issue #85
- bstr 1.9.1: Byte-string ops -- processes arbitrary byte streams without panicking on non-UTF-8
- regex 1.12.3: Pattern matching -- linear-time guarantees, SIMD-accelerated, no catastrophic backtracking
- encoding_rs 0.8.35: Codepage conversion -- Windows CP932/CP1252 to UTF-8 at I/O boundaries
- assert_cmd 2.2.1 + snapbox 1.2.1: Testing -- binary integration tests plus GNU compatibility snapshots
- cargo-wix 0.3.9 + WiX v3: MSI installer -- PATH registration, multi-binary, GitHub Actions windows-latest compatible

### Expected Features

The feature landscape is well-understood from the GOW open issue tracker. The MVP is the classic coreutils set with UTF-8 and MSI install; the Rust rewrite justification tier is the Windows bug fixes (tail -f, grep --color, find with spaces); power-user completeness is sed/diff/curl/tar/less.

**Must have (table stakes):**
- ls, cat, cp, mv, rm, mkdir, pwd, echo, head, wc, sort, uniq, tr, tee, cut, basename, dirname -- daily muscle memory; missing means users revert to Cygwin
- UTF-8 by default -- the #1 GOW complaint; must force UTF-8 at process start
- PATH auto-registration via MSI -- without this, adoption dies at the installer

**Should have (competitive differentiators vs. GOW 0.8.0):**
- tail -f via notify/ReadDirectoryChangesW -- fixes GOW #75/#169/#89 (entirely broken in original)
- grep --color working in Windows Terminal and ConHost -- fixes GOW #85
- find with spaces in paths, -exec, -print0 -- fixes GOW #208/#209
- xargs -0 -- essential companion to find for space-in-path safety
- dos2unix/unix2dos -- first tool users need on a new Windows machine
- Modern binary versions -- the top GOW complaint is decade-old binaries

**Defer (v2+):**
- curl -- reqwest + tokio async runtime; isolated to its own late phase
- Chocolatey/Scoop/winget publishing -- follows after binaries stabilize
- Path translation heuristics (Unix to Windows mount paths) -- implement incrementally
- gawk -- complex grammar; consider wrapping existing Windows build
- PowerShell module wrapper -- nice-to-have after core utilities proven stable

### Architecture Approach

The architecture is a Cargo workspace with a gow-core shared library crate and one uu_name library crate per utility, each with a thin main.rs binary entry point. This mirrors uutils/coreutils exactly and is the only proven pattern for this scale. gow-core centralizes all Windows platform concerns: UTF-8 console setup, path normalization, ANSI/VT100 mode init, shared error types, and GNU argument parsing helpers. No utility crate knows about Win32. No utility crate depends on another utility crate.

**Major components:**
1. gow-core (shared lib) -- UTF-8 init, path conversion, ANSI color, shared error types, GNU arg parse helpers; the foundation everything else depends on
2. uu_name (per-utility lib crates) -- single-utility logic, pub fn uu_app() and pub fn uumain(); testable without spawning a process
3. name.exe (thin binary wrappers) -- main.rs calls uu_name::uumain(args_os()); process::exit(code)
4. tests/common + tests/by-util/ -- TestScenario helper spawning real binaries; GNU compatibility snapshots via snapbox
5. packaging/wix/main.wxs -- custom WiX manifest with all binary Component entries plus single PATH Environment element

### Critical Pitfalls

1. **Clap argument parsing semantics diverge from GNU getopt** -- clap exits code 2 on bad args (GNU uses 1); cut -d= is mis-parsed; -5 shorthand (head -5) is unmodelable; option permutation differs. Prevention: build a gnu_arg abstraction over lexopt before writing any individual utility. Phase 1 work; cannot be retrofitted.

2. **Windows path conversion corrupts CLI flags** -- naive /c/Users to C:\\Users regex also matches /c flag arguments, reproducing GOW #244. Prevention: path conversion must be context-aware, not regex-based. Implement in gow-core::path with --no-path-conversion escape hatch.

3. **UTF-8/codepage mismatch at console I/O boundaries** -- Windows defaults to legacy codepages; Path::to_str().unwrap() panics on non-UTF-8 filenames. Prevention: call SetConsoleOutputCP(65001) via gow_core::init(), add Application Manifest activeCodePage=UTF-8, never unwrap paths.

4. **tail -f requires watching the parent directory, not the file** -- ReadDirectoryChangesW only watches directories. notify-rs must watch the parent dir and filter by filename, with 50-100ms debouncing. GOW #75/#169 are the direct consequence of not handling this.

5. **Windows file locking breaks in-place editing** -- sed -i and cp --force fail when target files are open. Prevention: use MoveFileExW(MOVEFILE_REPLACE_EXISTING), add retry logic (3x/100ms), implement Drop-based temp file cleanup.

## Implications for Roadmap

Based on the architecture dependency graph and pitfall phase warnings, the natural build order is 8 phases.

### Phase 1: Foundation -- gow-core and Build Infrastructure
**Rationale:** Every utility depends on this. Three of the six critical pitfalls are Phase 1 architectural decisions that cannot be retrofitted across 20+ utilities.
**Delivers:** gow-core crate (UTF-8 init, path normalization, ANSI color init, shared error types, GNU arg parsing abstraction); workspace Cargo.toml with pinned deps; .cargo/config.toml with MSVC target and static CRT flags; CI skeleton.
**Avoids:** Pitfall 1 (clap exit codes), Pitfall 3 (UTF-8 codepage), Pitfall 14 (POSIXLY_CORRECT), Pitfall 15 (-- end-of-options), Pitfall 16 (CRT dependency)

### Phase 2: Simple Stateless Utilities
**Rationale:** Validates workspace pattern and gow-core integration with no filesystem complexity. Proves build/test pipeline before tackling hard problems.
**Delivers:** cat, echo, pwd, true, false, yes, basename, dirname, env, tee, wc, which, dos2unix/unix2dos
**Uses:** gow-core, clap derive, bstr, thiserror, assert_cmd

### Phase 3: Filesystem Utilities
**Rationale:** Platform init proven; now tackle the filesystem layer. Symlink/junction handling and case sensitivity strategy must be established here -- they affect ls, find, cp -r, and rm -r.
**Delivers:** ls, cp, mv, rm, mkdir, head, tail (including -f), touch
**Uses:** walkdir, filetime, notify, terminal_size, termcolor
**Avoids:** Pitfall 4 (symlinks/junctions), Pitfall 5 (tail -f watcher), Pitfall 6 (file locking), Pitfall 7 (MAX_PATH), Pitfall 12 (case sensitivity)

### Phase 4: Text Processing Utilities
**Rationale:** Encoding patterns from Phase 3 proven. Text tools build on bstr/regex and the stream processing model.
**Delivers:** sort, uniq, tr, cut, grep (with --color), sed (with -i), diff
**Uses:** regex, bstr, termcolor, tempfile
**Avoids:** Pitfall 13 (sed temp file cleanup), Pitfall 10 (ANSI color TTY detection)

### Phase 5: Search and Subprocess Utilities
**Rationale:** find and xargs are the highest-signal GOW bug fixes. They require all prior patterns (path normalization, encoding, case sensitivity, subprocess quoting).
**Delivers:** find (with -exec, -print0, space-safe paths), xargs (with -0), grep -r
**Uses:** walkdir, globset, std::process::Command argv array API only
**Avoids:** Pitfall 2 (path conversion corrupting flags), Pitfall 11 (find -exec subprocess quoting)

### Phase 6: Compression and Network Utilities
**Rationale:** Most complex utilities. curl requires tokio and must be isolated from simpler coreutils; tar/gzip require C dependencies to be verified for MSVC.
**Delivers:** gzip/gunzip/zcat, tar, bzip2, less (pager), curl
**Uses:** flate2 (miniz backend), tar crate, crossterm (less only), reqwest + native-tls + tokio (curl only)

### Phase 7: Test Harness Hardening
**Rationale:** With all utilities built, run GNU compatibility test suite systematically. Distinct from per-utility integration tests written alongside each tool.
**Delivers:** GNU test suite integration via util/run-gnu-test.sh, compatibility percentage tracking in CI, snapbox baselines for all utilities, CI matrix across Windows versions

### Phase 8: MSI Installer and Distribution
**Rationale:** Depends on all release binaries being stable. WiX multi-binary manifest has specific gotchas (UpgradeCode, RemoveExistingProducts timing, Defender exclusion).
**Delivers:** cargo-wix MSI, PATH registration, Windows Defender exclusion post-install, Chocolatey/Scoop/winget package manifests
**Avoids:** Pitfall 8 (PATH not reflected in running terminals), Pitfall 17 (Defender scan penalty)

### Phase Ordering Rationale

- Phase 1 must be absolute first: three critical pitfalls are architectural decisions that cannot be retrofitted
- Phase 2 before Phase 3: simple utilities prove the pipeline cheaply before adding filesystem complexity
- Phase 3 before Phase 4: encoding and file I/O patterns must be proven before text processing depends on them
- Phase 5 after 3 and 4: find -exec and xargs require path normalization, encoding, and subprocess quoting from prior phases
- Phase 6 last among utilities: tokio must be isolated; C dependencies need MSVC validation
- Phase 7 and 8 are gates, not feature phases -- they require all utilities to exist

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 5 (find/xargs):** Windows process creation and argument quoting is complex; not well-documented outside uutils internals; spike recommended
- **Phase 6 (curl):** TLS/SChannel on corporate Windows (proxies, certificate stores) is a known landmine; research native-tls + reqwest on Windows specifically
- **Phase 8 (installer):** WiX v3 multi-binary WXS generation for 20+ binaries needs a concrete template; cargo-wix default single-binary workflow does not scale directly

Phases with standard patterns (skip research-phase):
- **Phase 1:** Workspace setup, gow-core pattern, static CRT build config -- fully documented by uutils/coreutils
- **Phase 2:** Simple stateless utilities -- straightforward Rust, no Windows-specific complexity
- **Phase 4:** Text processing with regex/bstr -- standard patterns, well-documented

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core crates verified against crates.io live API 2026-04-20; MSRV 1.85 confirmed; compression/HTTP crate versions not verified (noted in STACK.md) |
| Features | HIGH | Derived from GOW GitHub issues (verified) and uutils capability survey; MVP scope well-defined |
| Architecture | HIGH | Mirrors uutils/coreutils source exactly; Cargo workspace patterns from official docs; build order from explicit dependency graph |
| Pitfalls | HIGH | All critical pitfalls reference specific GOW issues, Rust stdlib issues, or uutils design docs with HIGH confidence |

**Overall confidence:** HIGH

### Gaps to Address

- **Compression crate versions** (flate2, bzip2, tar, xz2): Not verified against crates.io. Use cargo add at implementation time; do not pin in the roadmap.
- **HTTP/async crate versions** (reqwest, tokio, native-tls): Same gap. Verify at Phase 6 planning.
- **Multicall vs. individual binaries decision**: Load-bearing architecture decision (binary size, AV scan penalty, installer complexity) not resolved in research. Recommendation: start with individual binaries, revisit multicall in Phase 8 if Defender latency proves unacceptable.
- **Path translation heuristics**: No off-the-shelf solution. path-slash handles slash normalization but not MSYS2-style /c/Users mount paths. Custom logic in gow-core::path required; complexity is MEDIUM-HIGH.
- **GNU test suite runner on Windows**: Requires Git Bash or WSH. Exact CI setup not fully researched; Phase 7 planning needs a spike.

## Sources

### Primary (HIGH confidence)
- uutils/coreutils repository: https://github.com/uutils/coreutils
- crates.io live API (clap, thiserror, anyhow, windows-sys, assert_cmd, predicates, snapbox, notify, cargo-wix, crossterm, regex): verified 2026-04-20
- GOW open issues #75, #85, #169, #203, #208, #209, #244, #280: https://github.com/bmatzelle/gow/issues
- uutils-args design doc on problems with clap: https://github.com/tertsdiepraam/uutils-args/blob/main/docs/design/problems_with_clap.md
- Rust stdlib issues #138688, #56171, #54118, #66260: https://github.com/rust-lang/rust/issues/
- Cargo Workspaces official docs: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Microsoft MAX_PATH documentation: https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation
- windows-sys vs windows-rs comparison: https://microsoft.github.io/windows-rs/book/rust-getting-started/windows-or-windows-sys.html
- notify-rs (ReadDirectoryChangesW): https://github.com/notify-rs/notify
- BurntSushi/termcolor: https://github.com/BurntSushi/termcolor

### Secondary (MEDIUM confidence)
- WiX upgrade side-by-side discussion #8817: https://github.com/orgs/wixtoolset/discussions/8817
- MSI PATH broadcast behavior: https://social.technet.microsoft.com/Forums/en-US/4db362ce-ce9c-49c8-991c-c38f5b740cb7
- uutils extending blog post (Feb 2025): https://uutils.github.io/blog/2025-02-extending/

---
*Research completed: 2026-04-20*
*Ready for roadmap: yes*