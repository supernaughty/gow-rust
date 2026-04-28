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

- [ ] **Phase 06: archive-compression-and-network** — Archive, Compression, and Network
  - **Goal:** Implement archive and compression utilities (tar, gzip, bzip2, xz, gunzip, zcat) and a curl replacement with HTTPS, proxy, and Windows SChannel TLS — each as independent binaries.
  - **Requirements:** R018, R019, R020
  - **Plans:** TBD

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
**Plans**: TBD
