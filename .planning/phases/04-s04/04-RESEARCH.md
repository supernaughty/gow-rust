# Research: Text Processing (S04)

**Slice:** S04 — Text Processing
**Milestone:** M001
**Date:** 2026-04-25
**Confidence:** HIGH

## Executive Summary

Slice S04 delivers the core GNU text processing suite: `grep`, `sed`, `sort`, `uniq`, `tr`, `cut`, `diff`, `patch`, and `awk`. These utilities form the backbone of text-based automation and data processing. The primary challenge is maintaining high GNU compatibility (especially flags and regex behavior) while ensuring native Windows performance and reliability (fixing in-place editing and terminal color issues).

The implementation leverages the foundation laid in S01-S03, specifically `gow-core` for UTF-8/ANSI initialization and atomic file rewrites. Key library choices include the `grep` crate for performance, `regex` for pattern matching, `similar` for diffing, and `ext-sort` for large-scale sorting. Windows-specific pitfalls like file locking during `sed -i` are mitigated via `gow_core::fs::atomic_rewrite`.

## Natural Seams & Implementation Landscape

The work divides into six logical units based on complexity and shared technology:

### 1. Stream Filters (`tr`, `cut`, `uniq`)
- **Nature**: Simple, high-throughput byte/line filters.
- **Pattern**: Read stdin/file line-by-line or byte-by-byte using `bstr`.
- **Key Files**: New crates `gow-tr`, `gow-cut`, `gow-uniq`.
- **Risk**: Low. Minimal dependencies beyond `gow-core` and `bstr`.

### 2. Pattern Matching (`grep`)
- **Nature**: High-performance regex searching.
- **Pattern**: Use `grep` crate (preferred for performance) or `regex` + `bstr`. Must support `-r` (recursive) via `walkdir` and `--color` via `termcolor`.
- **Key Files**: New crate `gow-grep`.
- **Risk**: High performance expectations. Must handle large directories without exhausting file handles.

### 3. Stream Editing (`sed`)
- **Nature**: Non-interactive stream editor.
- **Pattern**: Regex-based replacement. Support `-i` (in-place) using `gow_core::fs::atomic_rewrite`.
- **Key Files**: New crate `gow-sed`.
- **Risk**: Windows file locking on `-i`. If the file is open in another process, `rename` might fail.

### 4. Sorting (`sort`)
- **Nature**: Order lines of text.
- **Pattern**: Use `ext-sort` for external merge sort to support files larger than RAM. Support numeric, reverse, and field-based sorting (`-k`).
- **Key Files**: New crate `gow-sort`.
- **Risk**: Performance and disk space for temporary files.

### 5. Comparisons (`diff`, `patch`)
- **Nature**: Compare and update text files.
- **Pattern**: Use `similar` for diffing algorithms and `patch` for applying unified diffs.
- **Key Files**: New crates `gow-diff`, `gow-patch`.
- **Risk**: Maintaining exact unified diff format compatibility for interoperability with standard tools.

### 6. Language-based Processing (`awk`)
- **Nature**: Data extraction and reporting.
- **Pattern**: Use `frawk` (fast Rust AWK) or implement a robust subset of POSIX AWK.
- **Key Files**: New crate `gow-awk`.
- **Risk**: Complexity. AWK is a full language.

## Library Research

| Utility | Crate | Purpose | Note |
|---------|-------|---------|------|
| `grep` | `grep` (BurntSushi) | Fast regex search | Includes `grep-searcher` and `grep-matcher`. |
| `sed` | `regex` | Pattern replacement | Custom logic needed for command parsing. |
| `sort` | `ext-sort` | External sort | Handles multi-GB files by spilling to disk. |
| `diff` | `similar` | Diff algorithms | High quality; supports Myers, Patience, etc. |
| `patch` | `patch` | Apply patches | Pure Rust implementation of unified diff parsing. |
| `awk` | `frawk` | AWK implementation | High performance; nearly full AWK support. |
| (all) | `bstr` | Byte-string ops | Safe line iteration on non-UTF-8 inputs. |

## Critical Pitfalls & Risks

### 1. Regex Compatibility (D-02)
- **Problem**: Rust's `regex` crate does not support backreferences or lookaround (to maintain linear-time guarantees). This is a deviation from GNU `grep -E` (ERE) and `sed`.
- **Mitigation**: Document the limitation. For `grep`, consider using `pcre2` feature if needed, but it adds a C dependency. For `sed`, basic POSIX regex features are mostly covered.

### 2. Windows File Locking (D-47)
- **Problem**: `sed -i` on Windows can fail if the file is locked. `std::fs::rename` (MoveFileExW) fails if an exclusive handle is held.
- **Mitigation**: `gow_core::fs::atomic_rewrite` already handles the temp-file-and-rename pattern. Add 3x/100ms retry logic for the rename step to handle transient locks (e.g., from antivirus scanners).

### 3. Encoding & Line Endings (D-48)
- **Problem**: GNU tools are byte-oriented. Windows tools often expect UTF-16 or specific codepages.
- **Mitigation**: Every utility calls `gow_core::init()` for UTF-8 setup. Use `bstr` to process bytes without panicking on invalid UTF-8. Ensure `grep` and `sed` handle `\r\n` vs `\n` gracefully (strip `\r` during matching or handle in regex).

### 4. AWK Complexity
- **Problem**: Implementing a full AWK is significant work.
- **Mitigation**: Leverage `frawk` as the engine. It is mature and highly performant.

## Recommendation for Planner

1.  **Start with Stream Filters**: Implement `tr`, `cut`, and `uniq` first. They are straightforward and validate the `bstr` stream pattern.
2.  **Tackle Grep & Sort**: These are high-signal utilities. Use the `grep` and `ext-sort` crates to achieve high performance with minimal custom code.
3.  **Implement sed with atomic_rewrite**: Ensure `sed -i` is tested heavily against Windows file locking.
4.  **Diff/Patch Integration**: Use `similar` and `patch` crates. Ensure the unified format output matches GNU `diff` exactly.
5.  **AWK Last**: AWK is the most complex. Verify if `frawk` meets all GNU requirements or if a subset is sufficient.

## Verification Plan

- **Unit Tests**: Test core logic (e.g., cut field slicing, tr mapping) inside the crate libraries.
- **Integration Tests**: Use `assert_cmd` and `predicates` to invoke the binaries.
- **Compatibility Snapshots**: Use `snapbox` for tools with complex output (`diff`, `grep --color`).
- **Large File Test**: Verify `sort` can handle a 500MB file on a machine with 256MB RAM (simulated).
- **Encoding Test**: Verify all tools handle UTF-8 and CP949 without corruption.
