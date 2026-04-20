# Feature Landscape

**Domain:** GNU utility suite for Windows (native, Rust-based)
**Researched:** 2026-04-20
**Confidence:** HIGH (core issues verified via GitHub issues, supported by ecosystem research)

---

## Table Stakes

Features users expect. Missing = product feels incomplete or users revert to Cygwin/WSL.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| ls, cat, cp, mv, rm, mkdir, pwd, echo | Muscle memory from every Unix tutorial; used daily | Low | uutils/coreutils covers most; Windows quirks in cp/mv (NTFS ACLs, symlinks) |
| head, tail | Log inspection is the #1 daily use case | Low–Med | tail -f is the critical sub-feature; see Critical Pitfalls |
| grep (with --color) | Text search; used in pipelines constantly | Med | Color output requires ENABLE_VIRTUAL_TERMINAL_PROCESSING or Windows Terminal; GOW #85 |
| find (with -exec, spaces in paths) | File discovery; scripts expect GNU semantics | Med–High | Windows paths with spaces are the failure mode (GOW #208, #209); -print0 support required |
| wc, sort, uniq, tr, cut, tee | Core data pipeline primitives | Low | Mostly stateless transforms; straightforward |
| sed (stream editing, -i in-place) | Script glue, config templating | Med | In-place on Windows needs temp-file dance; Windows line endings (CRLF) complicate patterns |
| dos2unix / unix2dos | Line ending normalization; Windows↔Unix file exchange | Low | Often the first tool users need on a new Windows machine |
| UTF-8 by default | Non-ASCII filenames, international content; GOW #280, #77 | Med | Windows console historically CP437/CP1252; must force UTF-8 at process start or use WTF-8 |
| PATH auto-registration at install | Users shouldn't have to touch environment variables | Low | MSI/NSIS installer writes to HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment |
| Single-binary-per-utility structure | Familiar GOW mental model; scripts call bare names (grep, not busybox grep) | Low | Compile each utility as its own .exe |
| which | PATH inspection; used to debug "why is the wrong grep running?" | Low | Windows has 'where' but developers trained on 'which' |
| curl | HTTP/HTTPS download; #1 scripting primitive | Med | Modern TLS required; GOW ships old curl; should use reqwest under the hood |
| diff | Code review, patch workflow | Med | Colorized diff output expected; --color flag |
| tar / gzip / bzip2 | Archive extraction and creation | Med | tar on Windows has historically been fragile |
| sort, uniq with locale awareness | Sorting non-ASCII text correctly | Med | Requires proper Unicode collation; not just byte sort |

---

## Differentiators

Features that set gow-rust apart from GOW 0.8.0, GnuWin32, and busybox-w32. Not baseline-expected, but meaningfully valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| tail -f with Windows file-change API | Real-time log following works reliably; GOW #169, #75, #89 entirely broken | Med | Use `notify` crate (ReadDirectoryChangesW backend); inotify semantics on Windows |
| Correct path translation (no slash corruption) | Scripts using `cmd /c`, `reg.exe /d`, etc. no longer break; GOW #244, #246 | Med–High | Must distinguish Unix mount-point paths (/c/Users) from CLI flags (/c, /v); heuristics required |
| ANSI color output without user setup | grep --color, ls colors, diff colors work out-of-box in Windows Terminal and modern ConHost | Low–Med | Call SetConsoleMode(ENABLE_VIRTUAL_TERMINAL_PROCESSING) at startup or detect terminal support; enables the full colored experience GOW #85 never had |
| find -print0 / xargs -0 pipeline | Spaces in Windows paths handled correctly end-to-end; GOW #209 | Med | NUL delimiter support in both find and xargs is essential; Windows paths with spaces are the norm |
| find -exec working correctly | Scripts ported from Linux work without rewriting; GOW #208 | Med | Argument parsing edge cases on Windows (semicolons, quoting) need explicit handling |
| Modern binary versions | Current TLS in curl, current regex engine in grep, current tar format support | High | Biggest complaint in GOW issues: "binary X is 10 years old" |
| PowerShell pipeline integration | Output can be piped into PowerShell Select-String, ForEach-Object, etc. | Med | Stdout as UTF-8 text without BOM; avoid CR-only line endings; test in pwsh.exe explicitly |
| Chocolatey / Scoop / winget package | Install via `choco install gow-rust` or `scoop install gow-rust`; no manual download | Med | Distribution channel as differentiator; GOW only has NSIS installer and is not in winget |
| Structured error output (stderr/exit codes) | Pipelines and scripts can detect failures reliably | Low | Follow GNU conventions strictly: exit 1 on error, message to stderr only |
| xargs utility | Essential companion to find; GOW ships gfind but no usable xargs | Med | NUL mode (-0), parallel execution (-P) expected by power users |
| Progress indication for long operations | cp/mv of large files, tar extraction show progress; not silent | Low | Optional --progress flag; respect NO_COLOR / non-TTY detection |
| Manifest / version subcommand | `gow --version` listing all bundled tools and their versions | Low | Trust signal: shows this isn't shipping decade-old binaries |

---

## Anti-Features

Features to explicitly NOT build. These would increase scope without proportional user value.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| bash / zsh shell | Separate project scale; nushell, Git Bash, MSYS2 shell all exist | Document that gow-rust pairs well with Git Bash or pwsh |
| vim / nano editor | Helix, Neovim, VS Code exist; editors are not utilities | Exclude; point users to dedicated editors |
| make / cmake | Cargo, just, nmake cover this; niche audience overlap | Out of scope per PROJECT.md |
| putty / SSH client | Windows 10+ ships OpenSSH natively; maintaining SSH is security-critical overhead | Excluded |
| bison / flex / indent | Parser generators and code formatters are separate toolchains | Out of scope per PROJECT.md |
| PHP / scripting runtimes | GOW was asked (issue #167); this is not a runtime suite | Hard no; scope creep |
| POSIX emulation layer (Cygwin-style) | Creates DLL dependency hell; MSYS2 already does this; defeats "native" goal | Stay native Win32/MSVC; no POSIX shim |
| Package manager (pacman/apt style) | Users already have Chocolatey, Scoop, winget | Publish to those registries instead |
| Auto-update daemon | Background processes conflict with enterprise GPO; creates uninstall complexity | Version bump via package manager only |
| IRC / chat client | Requested in GOW #131; completely out of domain | Hard no |
| Busybox-style multicall binary | Appealing for size but breaks "bare name" invocation; complicates PATH | Individual binaries per GOW tradition; optional symlinks later if requested |
| ripgrep as grep replacement | rg is already excellent and installable via winget; don't reinvent | Focus on GNU grep compatibility; users who want rg can install it separately |

---

## Feature Dependencies

```
UTF-8 encoding support
  └── All text-processing utilities (grep, sed, head, tail, cat, sort, wc)
      depends on: process-level UTF-8 mode (manifest or SetConsoleCP at startup)

ANSI color output
  └── grep --color, ls --color, diff --color
      depends on: Windows VT processing enabled at runtime

tail -f (follow mode)
  └── depends on: notify crate / ReadDirectoryChangesW integration

find with spaces + -exec
  └── depends on: correct Windows argument quoting (CommandLineToArgvW semantics)
  └── xargs -0 depends on: find -print0

Path translation (Unix↔Windows)
  └── depends on: heuristic parser that distinguishes mount-point paths from CLI flags
  └── used by: find paths, cp/mv src/dst, grep file args

PowerShell integration
  └── depends on: UTF-8 stdout without BOM + correct exit codes
  └── enhanced by: ANSI color (Windows Terminal renders it)

MSI installer / Chocolatey package
  └── depends on: stable binaries for all utilities
  └── PATH registration depends on: MSI component

curl (HTTPS)
  └── depends on: modern TLS stack (rustls or native-tls with SChannel)
```

---

## MVP Recommendation

**Priority 1 — Core coreutils** (ship this to validate)
- ls, cat, cp, mv, rm, mkdir, echo, pwd, head, wc, sort, uniq, tr, tee, basename, dirname, env, yes, true, false
- UTF-8 enforced at binary startup
- PATH registration via MSI installer

**Priority 2 — High-signal bug fixes** (justify the "Rust rewrite" story)
- tail (including -f via notify crate)
- grep (with --color working in Windows Terminal)
- find (spaces in paths, -exec, -print0)
- xargs (with -0 support)
- dos2unix / unix2dos

**Priority 3 — Power-user completeness**
- sed (-i in-place, CRLF-aware)
- diff (--color output)
- curl (modern TLS via rustls)
- tar / gzip / bzip2
- which
- less (pager with ANSI color)

**Defer for later phases:**
- gawk — complex grammar; consider linking to existing gawk Windows build or wrapping
- Chocolatey/Scoop/winget publishing — valuable but can follow after binaries stabilize
- Path translation heuristics — implement incrementally; start with documented behavior, not magic
- PowerShell module wrapper — nice-to-have after core utilities proven stable

---

## Sources

- GOW open issues (verified via GitHub): https://github.com/bmatzelle/gow/issues
  - #280: UTF-8 characters don't display correctly
  - #244: Unix to Windows path translation corrupts command switches
  - #169: tail -f fails with "Bad file descriptor"
  - #85: grep --color outputs raw control characters
  - #209: find with spaces in filenames breaks xargs pipeline
  - #208: find -exec "missing argument" error
- uutils/coreutils (Rust coreutils, 94.74% GNU test suite pass rate as of 0.8.0): https://github.com/uutils/coreutils
- notify-rs (cross-platform file watching, uses ReadDirectoryChangesW on Windows): https://github.com/notify-rs/notify
- MSYS2 PATH management warning (DLL incompatibility risk from mixing environments): https://www.msys2.org/wiki/How-does-MSYS2-differ-from-Cygwin/
- busybox-w32 (single-binary model, forward-slash handling): https://github.com/rmyorston/busybox-w32
- Windows VT processing (ANSI color in CMD/PowerShell): https://learn.microsoft.com/en-us/powershell/module/Microsoft.PowerShell.Core/about/about_ansi_terminals
- Windows package managers 2025: Chocolatey, Scoop, winget comparison (XDA Developers)
