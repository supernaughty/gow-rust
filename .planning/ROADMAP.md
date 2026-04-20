# Roadmap: gow-rust

**Project:** gow-rust — GNU On Windows, Rust rewrite
**Created:** 2026-04-20
**Granularity:** Standard (5-8 phases)
**Coverage:** 60/60 v1 requirements mapped

---

## Phases

- [ ] **Phase 1: Foundation** — gow-core shared library and Cargo workspace infrastructure
- [ ] **Phase 2: Stateless Utilities** — Simple utilities that validate workspace and gow-core integration
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
**Plans**: TBD

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
**Plans**: TBD

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
**Plans**: TBD
**UI hint**: yes

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
| 1. Foundation | 0/? | Not started | - |
| 2. Stateless Utilities | 0/? | Not started | - |
| 3. Filesystem Utilities | 0/? | Not started | - |
| 4. Text Processing | 0/? | Not started | - |
| 5. Search and Navigation | 0/? | Not started | - |
| 6. Archive, Compression, and Network | 0/? | Not started | - |

---

## Requirement Coverage

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOUND-01 | Phase 1 | Pending |
| FOUND-02 | Phase 1 | Pending |
| FOUND-03 | Phase 1 | Pending |
| FOUND-04 | Phase 1 | Pending |
| FOUND-05 | Phase 1 | Pending |
| FOUND-06 | Phase 1 | Pending |
| FOUND-07 | Phase 1 | Pending |
| WIN-01 | Phase 1 | Pending |
| WIN-02 | Phase 1 | Pending |
| WIN-03 | Phase 1 | Pending |
| UTIL-01 | Phase 2 | Pending |
| UTIL-02 | Phase 2 | Pending |
| UTIL-03 | Phase 2 | Pending |
| UTIL-04 | Phase 2 | Pending |
| UTIL-05 | Phase 2 | Pending |
| UTIL-06 | Phase 2 | Pending |
| UTIL-07 | Phase 2 | Pending |
| UTIL-08 | Phase 2 | Pending |
| UTIL-09 | Phase 2 | Pending |
| TEXT-03 | Phase 2 | Pending |
| FILE-06 | Phase 2 | Pending |
| FILE-07 | Phase 2 | Pending |
| FILE-08 | Phase 2 | Pending |
| WHICH-01 | Phase 2 | Pending |
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
*Last updated: 2026-04-20 after initial creation*
