# Phase 05: Search and Navigation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-26
**Phase:** 05-search-and-navigation
**Areas discussed:** find: -name case sensitivity, find: -exec support level, less: feature depth, xargs: parallel mode

---

## find: -name case sensitivity

| Option | Description | Selected |
|--------|-------------|----------|
| GNU-exact: case-sensitive -name, add -iname | Matches GNU behavior precisely. Scripts using -iname get insensitive matching; -name stays strict. Most portable for cross-platform scripts. | ✓ |
| Windows-aware: -name is case-insensitive | Matches Windows filesystem semantics. Diverges from GNU. | |
| Auto-detect from filesystem | Adapt based on target filesystem. Complex to implement correctly. | |

**User's choice:** GNU-exact — case-sensitive -name, add -iname

| Option | Description | Selected |
|--------|-------------|----------|
| Standard POSIX: * ? [...] only | Covers all real-world usage. Use globset crate. | ✓ |
| Extended: add ** recursive glob | Non-GNU behavior — could confuse users expecting standard find. | |

**User's choice:** Standard POSIX wildcards only

---

## find: -exec support level

| Option | Description | Selected |
|--------|-------------|----------|
| -exec cmd {} \; only | Covers GOW #208 and #209. Simpler on Windows without a shell. | ✓ |
| -exec cmd {} \; and -exec cmd {} + | Also includes arg-accumulating form. | |
| Full: \;, +, and -execdir | Most complete GNU compatibility. Most complex. | |

**User's choice:** `\;` form only — defer `+` and -execdir to gap-closure

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-quote with CreateProcess | Pass argv array directly to CreateProcess. Fixes GOW #209. | ✓ |
| Shell-escape via cmd.exe | Fragile with special chars, ties to Windows shell. | |

**User's choice:** CreateProcess direct — no shell intermediary

**Predicates selected (all four):**
- `-type f/d/l`
- `-size +N/-N`
- `-mtime/-atime/-ctime`
- `-maxdepth/-mindepth`

---

## less: feature depth

| Option | Description | Selected |
|--------|-------------|----------|
| Core pager: scroll, search, quit | Arrow keys, PgUp/PgDn, q, / search with n/N, G/g jump. | ✓ |
| Full less: + line numbers, marks, LESS env var | Significantly more scope. | |

**User's choice:** Core pager feature set

| Option | Description | Selected |
|--------|-------------|----------|
| Pass ANSI codes through raw | Detect ANSI escapes and render them. `grep --color \| less` shows color. | ✓ |
| Strip ANSI codes | Always strip — simpler but loses color output. | |
| You decide | Claude picks the approach. | |

**User's choice:** ANSI passthrough (like less -R)

| Option | Description | Selected |
|--------|-------------|----------|
| Streaming/buffered — never load full file | Lazy read, sliding window buffer. Handles 1GB+ files. | ✓ |
| Load fully into memory | Simple but OOMs on large files. | |

**User's choice:** Streaming/buffered

---

## xargs: parallel mode

| Option | Description | Selected |
|--------|-------------|----------|
| Skip -P, ship serial-only | Core flags: -0, -I {}, -n, -L. -P is complex on Windows, defer to gap-closure. | ✓ |
| Include -P with basic parallel | N concurrent processes. Significant added complexity. | |

**User's choice:** Serial-only, defer -P

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed {} only | Standard, covers all common scripts. | ✓ |
| Configurable -I STR | Custom replacement string. Small extra complexity. | |

**User's choice:** Fixed `{}` only

---

## Claude's Discretion

- Internal buffering strategy for less (ring buffer vs. line index)
- globset vs. manual glob matching for find
- Integration test structure
- Whether to add `-print0` to find (needed for xargs -0 interop — yes, include it)

## Deferred Ideas

- `find -exec {} +` and `-execdir`
- `xargs -P N` parallel mode
- `xargs -I STR` configurable replacement
- `less` line numbers, marks, LESS env var, -e auto-exit
