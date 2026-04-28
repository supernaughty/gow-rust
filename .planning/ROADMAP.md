# Roadmap

## M001: M001

- [x] **Phase 01: foundation** — Foundation
- [x] **Phase 02: stateless** — Stateless
- [x] **Phase 03: s03** — S03
- [x] **Phase 04: s04** — Text Processing (S04) *(Complete: 2026-04-25)*
  - **Goal:** Implement the core GNU text processing suite: grep, sed, sort, uniq, tr, cut, diff, patch, awk — each with high GNU compatibility, Windows-native UTF-8 support, and atomic file operations.
  - **Requirements:** R008, R009, R010, R011, R012, R013, R014
  - **Plans:** 10 plans

  Plans:
  - [x] 04-01-PLAN.md — Scaffold 9 text processing crates and workspace registration
  - [x] 04-02-PLAN.md — Implement tr, cut, uniq stream filters (R009, R010)
  - [x] 04-03-PLAN.md — Implement grep with regex, recursion, and color (R011)
  - [x] 04-04-PLAN.md — Implement sed with s/d/p commands and atomic -i editing (R012)
  - [x] 04-05-PLAN.md — Implement sort with -n -r -u -k key field and external merge (R008)
  - [x] 04-06-PLAN.md — Implement diff (unified format) and patch (atomic apply) (R014)
  - [x] 04-07-PLAN.md — Implement awk interpreter (field separation, printf, arrays) (R013)
  - [x] 04-08-PLAN.md — Gap closure: sort -k key field sorting (R008)
  - [x] 04-09-PLAN.md — Gap closure: sed d command and address ranges (R012)
  - [x] 04-10-PLAN.md — Gap closure: tr POSIX character classes [:alpha:] [:digit:] etc. (R010)

- [x] **Phase 05: search-and-navigation** — Search and Navigation *(Complete: 2026-04-28)*
  - **Goal:** Implement three GNU navigation utilities — find (file search with predicates and -exec), xargs (stdin-to-command-line builder), and less (interactive terminal pager) — each as an independent binary with high GNU compatibility and Windows-native UTF-8 support.
  - **Requirements:** R015, R016, R017
  - **Plans:** 4 plans

  Plans:
  - [x] 05-01-PLAN.md — Scaffold gow-find/gow-xargs/gow-less crates and add crossterm + globset workspace deps (R015, R016, R017)
  - [x] 05-02-PLAN.md — Implement gow-find with -name/-iname/-type/-size/-mtime/-exec/-print0 (R015)
  - [x] 05-03-PLAN.md — Implement gow-xargs with -0/-I/-n/-L plus find-print0 pipeline test (R016)
  - [x] 05-04-PLAN.md — Implement gow-less with crossterm raw mode, LineIndex, ANSI passthrough, non-TTY fallback (R017)

- [x] **Phase 06: archive-compression-and-network** — Archive, Compression, and Network *(Complete: 2026-04-28)*
  - **Goal:** Implement archive and compression utilities (tar, gzip, bzip2, xz, gunzip, zcat) and a curl replacement with HTTPS, proxy, and Windows SChannel TLS — each as independent binaries.
  - **Requirements:** R018, R019, R020
  - **Plans:** 6 plans

  Plans:
  - [x] 06-01-PLAN.md — Scaffold gow-gzip/gow-bzip2/gow-xz/gow-tar/gow-curl crates; add workspace deps; liblzma MSVC compile canary (R018, R019, R020)
  - [x] 06-02-PLAN.md — Implement gow-gzip: gzip/gunzip/zcat with argv[0] dispatch and flate2 streaming (R019)
  - [x] 06-03-PLAN.md — Implement gow-bzip2: bzip2/bunzip2 with MultiBzDecoder and pure-Rust backend (R019)
  - [x] 06-04-PLAN.md — Implement gow-xz: xz/unxz with liblzma XzEncoder/XzDecoder (R019)
  - [x] 06-05-PLAN.md — Implement gow-tar: -c/-x/-t with -z/-j codec dispatch; follow_symlinks(false) (R018)
  - [x] 06-06-PLAN.md — Implement gow-curl: reqwest blocking + native-tls SChannel; -o/-x/-I/-k/-f flags (R020)

## M002: v0.2.0

- [ ] **Phase 07: release-and-ci** — Release & CI/CD
- [ ] **Phase 08: code-review-fixes** — Code Review Fixes & Installer Polish
- [ ] **Phase 09: external-bundling** — External Binary Bundling
- [ ] **Phase 10: new-utilities-wave1** — New Rust Utilities Wave 1
- [ ] **Phase 11: new-utilities-wave2** — New Rust Utilities Wave 2

## Phase Details

### Phase 05: search-and-navigation
**Goal**: Implement three GNU navigation utilities — find (file search with predicates and -exec), xargs (stdin-to-command-line builder), and less (interactive terminal pager) — each as an independent binary with high GNU compatibility and Windows-native UTF-8 support.
**Depends on**: Phase 04
**Requirements**: R015, R016, R017
**Success Criteria** (what must be TRUE):
  1. `find` traverses directory trees with `-name`, `-type`, `-size`, `-mtime` predicates and executes commands via `-exec cmd {} \;`
  2. `xargs` reads stdin and builds command lines with `-0`, `-I {}`, `-n`, `-L` flags
  3. `less` pages files interactively with scroll, `/` search, and ANSI color passthrough
  4. `find -print0 | xargs -0 cmd` pipeline works end-to-end on Windows
  5. All three binaries compile cleanly as independent crates in the workspace
**Plans**: 4 plans (05-01 scaffold, 05-02 find, 05-03 xargs, 05-04 less)

### Phase 06: archive-compression-and-network
**Goal**: Implement archive and compression utilities (tar, gzip, bzip2, xz, gunzip, zcat) and a curl replacement with HTTPS, proxy, and Windows SChannel TLS — each as independent binaries.
**Depends on**: Phase 05
**Requirements**: R018, R019, R020
**Success Criteria** (what must be TRUE):
  1. `tar` creates and extracts archives with `-c`, `-x`, `-t`, `-z`, `-j` flags
  2. `gzip`/`gunzip`/`zcat` compress and decompress files
  3. `curl` performs HTTP/HTTPS requests with TLS 1.2/1.3 via Windows SChannel
  4. All binaries compile cleanly as independent crates in the workspace
**Plans**: 6 plans (06-01 scaffold, 06-02 gzip, 06-03 bzip2, 06-04 xz, 06-05 tar, 06-06 curl)

### Phase 07: release-and-ci
**Goal**: Users can download a v0.1.0 MSI installer from GitHub Releases, and every code change is automatically tested and release builds are automatically published.
**Depends on**: Phase 06
**Requirements**: REL-01, REL-02, REL-03, CI-01, CI-02, CI-03
**Success Criteria** (what must be TRUE):
  1. A v0.1.0 GitHub Release exists with x64 and x86 MSI files as downloadable assets
  2. Every push and pull request triggers `cargo test --workspace` via GitHub Actions and the result is visible on the PR
  3. Pushing a `v*` git tag automatically builds x64 and x86 release MSIs and attaches them to the GitHub Release — no manual steps required
  4. gow-probe.exe is absent from the installer; only user-facing utilities are bundled
  5. README or CONTRIBUTING.md contains ARM64 build instructions so a contributor can produce an ARM64 MSI without guidance from the maintainer
**Plans**: TBD

### Phase 08: code-review-fixes
**Goal**: All seven code review findings from Phase 06 are resolved; gow-tar, gow-xz, gow-gzip, and gow-curl behave correctly on edge cases that previously caused data loss or wrong exit codes.
**Depends on**: Phase 07
**Requirements**: FIX-01, FIX-02, FIX-03, FIX-04, FIX-05, FIX-06, FIX-07
**Success Criteria** (what must be TRUE):
  1. `tar xjf` correctly extracts multi-stream .tar.bz2 archives without truncation
  2. `tar` with invalid arguments prints a GNU-style error message and exits 2 rather than panicking
  3. `tar` exits non-zero when one or more archive entries fail to extract
  4. `xz -d` correctly decompresses concatenated .xz files without silently truncating output
  5. `gzip -d file` (where file lacks .gz suffix) prints a GNU-compatible error and exits non-zero instead of producing a .out file
  6. `curl -I -s` suppresses header output; `curl -s` produces no progress or diagnostic output
  7. `curl -o out_file` removes the partial file when an I/O error occurs mid-download
**Plans**: TBD

### Phase 09: external-bundling
**Goal**: Users who install gow-rust get vim, wget, and nano available on their PATH alongside the Rust binaries, and legacy GOW command names continue to work via batch file shims.
**Depends on**: Phase 08
**Requirements**: BND-01, BND-02, BND-03, BND-04
**Success Criteria** (what must be TRUE):
  1. Running `download-extras.ps1` from the repository root downloads vim portable (v9.2+), wget (v1.21.4), and nano portable (v7.2+) into `extras/bin/`
  2. After installing the MSI, `vim`, `wget`, and `nano` are available on PATH without any manual steps
  3. Legacy names `egrep`, `fgrep`, `bunzip2`, `gawk`, `gfind`, `gsort` invoke the correct Rust binaries via batch file shims
  4. The installer presents an optional "Extras" feature that a user can deselect to skip vim/nano/wget installation
**Plans**: TBD

### Phase 10: new-utilities-wave1
**Goal**: Ten simple GNU utilities — seq, sleep, tac, nl, od, fold, expand, unexpand, du, df, and the hash suite (md5sum, sha1sum, sha256sum) — are implemented as independent Rust binaries and included in the installer.
**Depends on**: Phase 09
**Requirements**: U-01, U-02, U-03, U-04, U-05, U-06, U-07, U-08, U-09, U-10
**Success Criteria** (what must be TRUE):
  1. `seq 10`, `seq 1 2 10`, and `seq 1.5 0.5 3` produce correct GNU-compatible output
  2. `sleep 1` and `sleep 0.5` delay for the specified duration and exit 0
  3. `tac`, `nl`, `od`, `fold`, `expand`, `unexpand` each produce output matching GNU reference for their core options
  4. `du -sh .` and `df -h` report disk usage and free space without errors on Windows volumes
  5. `md5sum -c`, `sha1sum -c`, `sha256sum -c` verify files and exit 0 on match, non-zero on mismatch
  6. All 13 binaries (expand + unexpand counted separately) pass `cargo test --workspace` and appear in the MSI
**Plans**: TBD
**UI hint**: no

### Phase 11: new-utilities-wave2
**Goal**: Ten additional GNU utilities — whoami, uname, paste, join, split, printf, expr, test, fmt, and unlink — are implemented as independent Rust binaries and included in the installer.
**Depends on**: Phase 10
**Requirements**: U2-01, U2-02, U2-03, U2-04, U2-05, U2-06, U2-07, U2-08, U2-09, U2-10
**Success Criteria** (what must be TRUE):
  1. `whoami` prints the current Windows username and exits 0
  2. `uname -a` prints Windows OS name, release, and machine architecture in GNU-compatible format
  3. `paste`, `join`, `split` each produce output matching GNU reference for their core options
  4. `printf "%d\n" 42` and `expr 3 + 4` produce correct output matching GNU behavior
  5. `test -f existing_file` exits 0 and `test -f missing_file` exits 1; `[` alias behaves identically
  6. `fmt`, `unlink` execute without errors on valid inputs and pass `cargo test --workspace`
  7. All 11 binaries are included in the MSI and available on PATH after installation
**Plans**: TBD

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 01. foundation | — | Complete | 2026-04-xx |
| 02. stateless | — | Complete | 2026-04-xx |
| 03. s03 | — | Complete | 2026-04-xx |
| 04. s04 | 10/10 | Complete | 2026-04-25 |
| 05. search-and-navigation | 4/4 | Complete | 2026-04-28 |
| 06. archive-compression-and-network | 6/6 | Complete | 2026-04-28 |
| 07. release-and-ci | 0/? | Not started | - |
| 08. code-review-fixes | 0/? | Not started | - |
| 09. external-bundling | 0/? | Not started | - |
| 10. new-utilities-wave1 | 0/? | Not started | - |
| 11. new-utilities-wave2 | 0/? | Not started | - |
