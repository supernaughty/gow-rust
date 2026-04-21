# Roadmap: gow-rust

**Project:** gow-rust — GNU On Windows, Rust rewrite
**Created:** 2026-04-20
**Granularity:** Standard (5-8 phases)
**Coverage:** 60/60 v1 requirements mapped

---

## Phases

- [x] **Phase 1: Foundation** — gow-core shared library and Cargo workspace infrastructure — completed 2026-04-21
- [x] **Phase 2: Stateless Utilities** — Simple utilities that validate workspace and gow-core integration — completed 2026-04-21
- [ ] **Phase 3: Filesystem Utilities** — File and directory operations including tail -f
- [ ] **Phase 4: Text Processing** — Search, stream editing, AWK, diff, and patch
- [ ] **Phase 5: Search and Navigation** — find, xargs, less pager, and binary search
- [ ] **Phase 6: Archive, Compression, and Network** — tar, gzip, bzip2, zip, curl

---

## Phase Details

### Phase 1: Foundation
**Goal**: The Cargo workspace exists and gow-core provides all shared platform primitives that every utility crate depends on
**Depends on**: Nothing (first phase)
**Requirements**: FOUND-01, FOUND-02, FOUND-03, FOUND-04, FOUND-05, FOUND-06, FOUND-07, WIN-01, WIN-02, WIN-03
**Success Criteria** (what must be TRUE):
  1. `cargo build --workspace` succeeds with MSVC toolchain on Windows 10/11 and produces no warnings in gow-core
  2. Any utility binary built against gow-core initializes with UTF-8 console output (non-ASCII filenames print correctly in both Windows Terminal and legacy ConHost)
  3. A path like `/c/Users/foo` passed to a gow-core utility converts correctly to `C:\Users\foo` without corrupting flag arguments like `-c`
  4. ANSI color escape codes display in both Windows Terminal and legacy ConHost without raw escape characters appearing in output
  5. GNU argument parsing rejects bad args with exit code 1 (not 2) and respects `--` end-of-options
**Plans**: 4 plans

Plans:
- [x] 01-01-PLAN.md — Cargo workspace scaffold + gow-core crate structure (build.rs, lib.rs stubs) — completed 2026-04-20
- [x] 01-02-PLAN.md — gow-core: encoding, args, color modules with unit tests — completed 2026-04-20
- [x] 01-03-PLAN.md — gow-core: error, path, fs modules with unit tests — completed 2026-04-20
- [x] 01-04-PLAN.md — gow-probe test binary + integration tests + human verification checkpoint — completed 2026-04-21

### Phase 2: Stateless Utilities
**Goal**: Users can run the complete set of simple, stateless GNU utilities and observe correct GNU-compatible behavior on Windows
**Depends on**: Phase 1
**Requirements**: UTIL-01, UTIL-02, UTIL-03, UTIL-04, UTIL-05, UTIL-06, UTIL-07, UTIL-08, UTIL-09, TEXT-03, FILE-06, FILE-07, FILE-08, WHICH-01
**Success Criteria** (what must be TRUE):
  1. `echo -e "\t"` outputs a real tab character; `echo -n` suppresses the trailing newline
  2. `wc -l`, `wc -w`, and `wc -c` return correct counts on files containing non-ASCII UTF-8 content
  3. `which` locates executables on the Windows PATH, including `.exe`/`.cmd` extensions, and returns the correct absolute path
  4. `mkdir -p a/b/c` creates nested directories in a single invocation without error if they already exist
  5. `tee file.txt` writes stdin to both the file and stdout simultaneously; `tee -a` appends to existing content
**Plans**: 11 plans

Plans:
- [x] 02-01-PLAN.md — Workspace prep: add 14 Phase 2 members + snapbox/bstr/filetime workspace deps + stub crate scaffolds — completed 2026-04-21
- [x] 02-02-PLAN.md — gow-true + gow-false + gow-yes (trivial trio; BrokenPipe-safe yes loop) — completed 2026-04-21
- [x] 02-03-PLAN.md — gow-echo with -n/-e/-E + escape state machine (RESEARCH.md Q9) — completed 2026-04-21
- [x] 02-04-PLAN.md — gow-pwd with -L/-P and UNC-safe simplify_canonical (RESEARCH.md Q8) — completed 2026-04-21
- [x] 02-05-PLAN.md — gow-basename + gow-dirname (MSYS pre-convert + suffix strip) — completed 2026-04-21
- [x] 02-06-PLAN.md — gow-mkdir + gow-rmdir (create_dir_all + parent walk loop) — completed 2026-04-21
- [x] 02-07-PLAN.md — gow-tee with split-writer + -i SetConsoleCtrlHandler (RESEARCH.md Q10) — completed 2026-04-21
- [x] 02-08-PLAN.md — gow-wc Unicode-aware via bstr (TEXT-03, ROADMAP criterion 2) — completed 2026-04-21
- [x] 02-09-PLAN.md — gow-env with -i/-u/-C/-S/-0/-v + split-string state machine (RESEARCH.md Q7) — completed 2026-04-21
- [x] 02-10-PLAN.md — gow-touch with -a/-m/-c/-r/-d/-t/-h via jiff+parse_datetime+filetime (RESEARCH.md Q1/Q2) — completed 2026-04-21
- [x] 02-11-PLAN.md — gow-which hybrid PATHEXT resolver (WHICH-01, GOW #276) — completed 2026-04-21

### Phase 3: Filesystem Utilities
**Goal**: Users can perform all core file and directory operations with correct behavior on Windows paths, symlinks, and file locking, including real-time file watching
**Depends on**: Phase 1
**Requirements**: FILE-01, FILE-02, FILE-03, FILE-04, FILE-05, FILE-09, FILE-10, TEXT-01, TEXT-02, CONV-01, CONV-02
**Success Criteria** (what must be TRUE):
  1. `ls -la` lists hidden files (dot-prefix and Windows-hidden-attribute), shows permissions, and colorizes output; symlinks and junctions display as link types
  2. `cp -r src/ dest/` copies directory trees recursively across drives; `cp -p` preserves timestamps
  3. `tail -f logfile` detects new lines appended by another process within 200ms and streams them to stdout without polling (uses ReadDirectoryChangesW)
  4. `cat -n` on a file with non-ASCII UTF-8 content outputs correct numbered lines without mojibake
  5. `dos2unix` converts CRLF to LF in-place on a Windows-native text file; `unix2dos` reverses the conversion
**Plans**: 12 plans
**UI hint**: yes

Plans:
- [ ] 03-01-PLAN.md — Wave 0: workspace prep (walkdir/notify/terminal_size/junction deps) + gow-core::fs 7 new helpers + 11 stub crates
- [ ] 03-02-PLAN.md — Wave 1: gow-cat (FILE-01) — byte-stream concat with -n/-b/-s/-v/-E/-T/-A
- [ ] 03-03-PLAN.md — Wave 1: gow-head (TEXT-01) — first N lines/bytes with multi-file headers + numeric shorthand
- [ ] 03-04-PLAN.md — Wave 1: gow-chmod (FILE-10) — D-32 owner-write bit model + walkdir -R
- [ ] 03-05-PLAN.md — Wave 2: gow-dos2unix (CONV-01) — atomic rewrite (D-47) + shared transform module
- [ ] 03-06-PLAN.md — Wave 3: gow-unix2dos (CONV-02) — depends on 03-05 transform; round-trip test
- [ ] 03-07-PLAN.md — Wave 3: gow-cp (FILE-03) — walkdir recursion + filetime -p preserve + symlink clone
- [ ] 03-08-PLAN.md — Wave 3: gow-rm (FILE-05) — contents_first walk + D-42 drive-root safety + D-45 RO handling
- [ ] 03-09-PLAN.md — Wave 3: gow-ls (FILE-02) — walkdir -R + terminal_size layout + D-31/D-34/D-35/D-37 display
- [ ] 03-10-PLAN.md — Wave 4: gow-ln (FILE-09) — create_link dispatch with D-36 junction fallback
- [ ] 03-11-PLAN.md — Wave 4: gow-mv (FILE-04) — rename + cross-volume fallback with mtime preserve
- [ ] 03-12-PLAN.md — Wave 5: gow-tail (TEXT-02) — notify watcher for -f/-F, ROADMAP criterion 3

### Phase 4: Text Processing
**Goal**: Users can search, filter, transform, and edit text streams with full GNU-compatible regex and in-place editing that works correctly under Windows file locking
**Depends on**: Phase 3
**Requirements**: TEXT-04, TEXT-05, TEXT-06, GREP-01, GREP-02, GREP-03, SED-01, SED-02, AWK-01, AWK-02, DIFF-01, DIFF-02
**Success Criteria** (what must be TRUE):
  1. `grep --color -rn pattern dir/` recurses into directories, highlights matches in color in Windows Terminal and ConHost, and returns exit code 0/1 correctly
  2. `grep -E` supports extended regex; `grep -F` treats pattern as a literal string with no regex interpretation
  3. `sed -i 's/foo/bar/g' file.txt` performs in-place substitution even when the file is locked by another reader; a clean temp file is used and swapped atomically
  4. `awk '{print $1, $NF}` file` processes field separation, associative arrays, and printf correctly on UTF-8 input
  5. `diff -u file1 file2` produces unified diff output; `patch -p0 < file.patch` applies it and produces the expected result
**Plans**: TBD

### Phase 5: Search and Navigation
**Goal**: Users can search for files with complex criteria including paths containing spaces, execute commands on results, page through large files, and perform binary search-and-replace
**Depends on**: Phase 3, Phase 4
**Requirements**: FIND-01, FIND-02, FIND-03, XARGS-01, LESS-01, BIN-01
**Success Criteria** (what must be TRUE):
  1. `find . -name "*.rs" -type f` finds all matching files including in directories whose paths contain spaces
  2. `find . -name "*.log" -exec rm {} \;` correctly executes the specified command once per result; arguments are passed as an array (not a shell string), preventing injection
  3. `find . -print0 | xargs -0 wc -l` processes filenames containing spaces and newlines correctly via null-delimiter protocol
  4. `less largefile.log` opens immediately without loading the full file; user can scroll forward, backward, and search with `/pattern`
  5. `gsar -s "old" -r "new" file.bin` performs binary search-and-replace on a non-text file and writes the result correctly
**Plans**: TBD

### Phase 6: Archive, Compression, and Network
**Goal**: Users can create and extract archives, compress and decompress files with multiple formats, and make HTTPS requests through corporate proxies with modern TLS
**Depends on**: Phase 1
**Requirements**: ARCH-01, ARCH-02, ARCH-03, ARCH-04, ARCH-05, NET-01, NET-02
**Success Criteria** (what must be TRUE):
  1. `tar -czf archive.tar.gz dir/` creates a gzip-compressed tar archive; `tar -xzf archive.tar.gz` extracts it with correct paths and permissions
  2. `gzip file` compresses and removes the original; `gunzip file.gz` decompresses; `gzip -d` works as an alias
  3. `bzip2` and `bunzip2` compress and decompress files; round-trip produces byte-identical output to the original
  4. `curl -L https://example.com -o output.html` follows redirects and writes the response; `curl -x http://proxy:3128` routes through a proxy with TLS 1.2/1.3
  5. `zip archive.zip file1 file2` creates a zip archive readable by Windows Explorer; `unrar x archive.rar` extracts a RAR archive
**Plans**: TBD

---

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 4/4 | Complete | 2026-04-21 |
| 2. Stateless Utilities | 11/11 | Complete | 2026-04-21 |
| 3. Filesystem Utilities | 0/? | Not started | - |
| 4. Text Processing | 0/? | Not started | - |
| 5. Search and Navigation | 0/? | Not started | - |
| 6. Archive, Compression, and Network | 0/? | Not started | - |

---

## Requirement Coverage

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOUND-01 | Phase 1 | Done (Plan 01-01) |
| FOUND-02 | Phase 1 | Done (Plan 01-02) |
| FOUND-03 | Phase 1 | Done (Plan 01-02) |
| FOUND-04 | Phase 1 | Done (Plan 01-02) |
| FOUND-05 | Phase 1 | Done (Plan 01-03) |
| FOUND-06 | Phase 1 | Done (Plan 01-03) |
| FOUND-07 | Phase 1 | Done (Plan 01-03) |
| WIN-01 | Phase 1 | Done (Plan 01-02) — validated in Plan 01-04 |
| WIN-02 | Phase 1 | Done (Plan 01-04 — manifest longPathAware=true verified at PE binary level) |
| WIN-03 | Phase 1 | Done (Plan 01-02) — validated in Plan 01-04 |
| UTIL-01 | Phase 2 | Done (Plan 02-03) |
| UTIL-02 | Phase 2 | Done (Plan 02-04) |
| UTIL-03 | Phase 2 | Done (Plan 02-09) |
| UTIL-04 | Phase 2 | Done (Plan 02-07) |
| UTIL-05 | Phase 2 | Done (Plan 02-05) |
| UTIL-06 | Phase 2 | Done (Plan 02-05) |
| UTIL-07 | Phase 2 | Done (Plan 02-02) |
| UTIL-08 | Phase 2 | Done (Plan 02-02) |
| UTIL-09 | Phase 2 | Done (Plan 02-02) |
| TEXT-03 | Phase 2 | Done (Plan 02-08) |
| FILE-06 | Phase 2 | Done (Plan 02-06) |
| FILE-07 | Phase 2 | Done (Plan 02-06) |
| FILE-08 | Phase 2 | Done (Plan 02-10) |
| WHICH-01 | Phase 2 | Done (Plan 02-11 — GOW #276 resolved) |
| FILE-01 | Phase 3 | Pending |
| FILE-02 | Phase 3 | Pending |
| FILE-03 | Phase 3 | Pending |
| FILE-04 | Phase 3 | Pending |
| FILE-05 | Phase 3 | Pending |
| FILE-09 | Phase 3 | Pending |
| FILE-10 | Phase 3 | Pending |
| TEXT-01 | Phase 3 | Pending |
| TEXT-02 | Phase 3 | Pending |
| CONV-01 | Phase 3 | Pending |
| CONV-02 | Phase 3 | Pending |
| TEXT-04 | Phase 4 | Pending |
| TEXT-05 | Phase 4 | Pending |
| TEXT-06 | Phase 4 | Pending |
| GREP-01 | Phase 4 | Pending |
| GREP-02 | Phase 4 | Pending |
| GREP-03 | Phase 4 | Pending |
| SED-01 | Phase 4 | Pending |
| SED-02 | Phase 4 | Pending |
| AWK-01 | Phase 4 | Pending |
| AWK-02 | Phase 4 | Pending |
| DIFF-01 | Phase 4 | Pending |
| DIFF-02 | Phase 4 | Pending |
| FIND-01 | Phase 5 | Pending |
| FIND-02 | Phase 5 | Pending |
| FIND-03 | Phase 5 | Pending |
| XARGS-01 | Phase 5 | Pending |
| LESS-01 | Phase 5 | Pending |
| BIN-01 | Phase 5 | Pending |
| ARCH-01 | Phase 6 | Pending |
| ARCH-02 | Phase 6 | Pending |
| ARCH-03 | Phase 6 | Pending |
| ARCH-04 | Phase 6 | Pending |
| ARCH-05 | Phase 6 | Pending |
| NET-01 | Phase 6 | Pending |
| NET-02 | Phase 6 | Pending |

**Coverage: 59/59 v1 requirements mapped (Note: actual count is 59, not 52 as pre-stated)**

---

*Roadmap created: 2026-04-20*
*Last updated: 2026-04-21 after Phase 2 planning (11 plans)*
