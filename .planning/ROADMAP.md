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

- [ ] **Phase 05: search-and-navigation** — Search and Navigation
- [ ] **Phase 06: archive-compression-and-network** — Archive, Compression, and Network
