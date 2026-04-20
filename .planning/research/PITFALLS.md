# Domain Pitfalls

**Domain:** Rust-based GNU utility reimplementation for Windows (gow-rust)
**Researched:** 2026-04-20

---

## Critical Pitfalls

Mistakes that cause rewrites, broken compatibility, or user-facing failures.

---

### Pitfall 1: Clap Does Not Implement GNU Argument Semantics

**What goes wrong:** Using `clap` as-is for GNU-compatible option parsing produces wrong behavior in subtle but user-visible ways. GNU coreutils use `getopt`/`getopt_long` which has specific semantics that clap does not replicate.

**Why it happens:** Clap is designed for modern CLI conventions, not POSIX/GNU legacy parsing. The gaps are documented by the uutils project's own postmortem:

- **Exit code mismatch**: clap exits with code `2` on bad args; GNU tools exit with `1`.
- **Short-option value parsing**: `cut -d=` — the `=` should be the delimiter value, not a separator. Clap does not handle this correctly.
- **Deprecated numeric syntax**: `head -5` (shorthand for `-n 5`), `tail +5` — clap cannot model these idioms cleanly.
- **Option permutation**: By default, GNU `getopt` permutes options so `ls file -la` works. POSIX mode stops at the first non-option. Clap's default differs.
- **Help flag differences**: clap adds `-h` automatically; GNU coreutils only support `--help`. Scripts that test `cmd -h` for help vs. `cmd -h <arg>` break.
- **Optional argument syntax**: GNU-style optional values require `=` (e.g., `--color=always`), but clap does not enforce this — `--color always` may be parsed differently.
- **Options can override/negate each other**: e.g., `ls -l -C -l` — last `-l` wins. Clap's model assumes each option is independent.

**Consequences:** Existing user scripts break. GNU test suite fails. Users report "wrong output" with valid GNU invocations.

**Prevention:**
- Use `clap` with manual `ArgAction` and custom validators for GNU-critical tools (grep, find, ls, sed)
- For tools with complex GNU parsing (cut, head, tail, sort), consider `lexopt` (a minimal, getopt-faithful lexer) as the parsing layer, with manual dispatch on top
- Implement a shared `gnu_exit` utility that maps clap errors to exit code `1`
- Test against the actual GNU testsuite using CI (see uutils' approach with `run_tests.sh`)
- Reference: [uutils-args design doc on clap problems](https://github.com/tertsdiepraam/uutils-args/blob/main/docs/design/problems_with_clap.md)

**Detection:** Run `cmd badarg 2>/dev/null; echo $?` — if it prints `2` instead of `1`, clap is leaking its exit code.

**Phase:** Phase 1 (foundation) — establish a `parse_gnu_args` abstraction before building any individual utility.

---

### Pitfall 2: Windows Path Separator Confusion and Automatic Conversion

**What goes wrong:** GOW's original path translation converts Unix-style `/c/Users` to `C:\Users`, but this causes silent corruption of command-line switches. The documented GOW Issue #244 shows `cmd /c "echo foobar"` being executed as `c:\Windows\system32\cmd.exe c:/ "echo foobar"` — the `/c` flag becomes a drive path.

**Why it happens:** A naive regex like "replace `/letter/` with `letter:\`" matches option flags. The conversion logic has no awareness of argument context.

**Consequences:** Windows programs called via `find -exec`, `xargs`, or shell scripts receive wrong arguments. Silent data corruption in pipelines.

**Prevention:**
- Implement path conversion only when an argument is confirmed to be a filesystem path (exists, or is explicitly prefixed)
- Never convert arguments that follow known option prefixes (e.g., arguments immediately after flags like `-c`, `-f`, `-o`)
- Use `std::path::Path::new(arg).exists()` combined with heuristics to distinguish paths from option values
- Expose a `--no-path-conversion` flag for escape hatch
- Always test with `cmd /c`, `powershell -c`, and similar invocations in CI

**Detection:** Test `cmd /c "echo test"` — if the output is an error about `c:/` not being recognized, path translation is corrupting flags.

**Phase:** Phase 2 (path/encoding layer) — implement before any tool that invokes subprocesses.

---

### Pitfall 3: UTF-8 / Windows Codepage Mismatch at Console I/O Boundaries

**What goes wrong:** Windows uses UTF-16 internally. The console codepage defaults to a legacy codepage (e.g., CP932 on Japanese Windows, CP1252 on Western). Rust's `std::io` does NOT automatically re-encode. Reading filenames or stdin on a non-UTF-8 console produces `?` replacement characters or panics on surrogate pairs.

**Why it happens:** Windows `OsStr` is WTF-8 (potentially-ill-formed UTF-16 sequences), not UTF-8. `Path::to_str()` returns `None` for filenames with unpaired surrogates. Rust's stdlib issues 12056, 56171, and 23344 document this gap.

**Specific risks:**
- `fs::read_dir()` + `entry.file_name().to_str()` silently skips files with non-UTF-8-representable names
- Writing to stdout when the console codepage is not UTF-8 produces mojibake
- Reading filenames from stdin (e.g., `find | xargs`) loses data if shell uses a different encoding

**Prevention:**
- At startup, call `SetConsoleOutputCP(65001)` and `SetConsoleCP(65001)` via `winapi` to force UTF-8 (Windows 10 1903+ supports this reliably)
- Use `std::os::windows::ffi::OsStrExt` and `OsStringExt` for round-trip-safe filename handling — never `to_str().unwrap()`
- Add a manifest entry setting `activeCodePage` to `UTF-8` (Windows 10 1903+ Application Manifest)
- Use the `encoding_rs` crate for converting legacy codepage input (piped from older tools)
- Write integration tests with filenames that contain CJK, emoji, and Latin-extended characters

**Detection:** Create a file named `테스트.txt` or `résumé.txt`, then run `ls` — if the name is garbled, codepage handling is broken.

**Phase:** Phase 1 (UTF-8 foundation crate) — must be solved before any tool emits filenames.

---

### Pitfall 4: Windows Symlinks, Junctions, and Reparse Points Are Not Unix Symlinks

**What goes wrong:** Windows has three distinct link types — symlinks (file vs. directory, require privilege by default), junction points (directory only, same volume implied), and hard links. GNU tools treat all of these as "symlink" (`l` in `ls -l`, `-L` in `find`), but the Windows behavior differs:

- `std::fs::metadata()` follows symlinks; `symlink_metadata()` does not — confusion between these causes `ls -l` to show wrong file sizes or types
- `std::os::windows::fs::symlink_file` has a known bug (Rust 1.85, Issue #138688) with absolute paths
- Junction points behave like symlinks for traversal but `readlink` on them returns a device path like `\??\C:\target`, not the original path
- Creating symlinks on Windows 10/11 without Developer Mode enabled requires elevation — `ln -s` must fail gracefully with a clear error, not a cryptic OS error code

**Prevention:**
- Use `symlink_metadata()` for `ls -l` type inspection; `metadata()` for size/content operations
- Normalize junction point targets: strip `\??\` prefix when returning link targets to users
- Check `DeviceIoControl` for reparse point tag to distinguish junction from symlink
- Gate symlink creation on privilege check; return a user-friendly error if elevation needed
- Test with all three link types in CI (hard link, junction, symlink — both file and dir variants)

**Detection:** `ls -la` on a directory containing a junction point — if it shows `->` with a `\??\` device path, reparse point handling is wrong.

**Phase:** Phase 2 (filesystem operations) — before implementing `ls`, `find`, `cp -r`, `rm -r`.

---

### Pitfall 5: `tail -f` on Windows Requires Polling, Not inotify

**What goes wrong:** `tail -f` on the original GOW returns "Bad file descriptor" and exits immediately (GOW issues #75, #169). The root cause: inotify does not exist on Windows. `ReadDirectoryChangesW` watches directories, not individual files. A naive port that checks for an inotify-equivalent will silently fall back to no-op or fail at runtime.

**Why it happens:** `notify-rs` (the standard Rust filesystem notification crate) uses `ReadDirectoryChangesW` on Windows, which only works on directories. To watch a file, you must watch its parent directory and filter events by filename. Additionally, Windows file writes generate multiple events (FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE) — naive handling causes duplicate output or missed updates.

**Prevention:**
- Use `notify-rs` watching the parent directory, filter events by the target filename
- Implement debouncing (50-100ms window) to collapse multiple rapid events into one read
- For polling fallback (network shares, FAT32), implement a stat-based loop checking file size/mtime
- Handle log rotation: when file shrinks or inode changes (check `file_index` from `GetFileInformationByHandle`), restart from offset 0 or emit a warning
- Test on NTFS local, network share (SMB), and FAT32 (USB) filesystems

**Detection:** `tail -f /tmp/test.log` while `echo line >> /tmp/test.log` runs in another terminal — if output stops after first read, the watcher is not working.

**Phase:** Phase 3 (streaming I/O tools) — tail is a high-priority GOW pain point.

---

### Pitfall 6: Windows File Locking Breaks Tools That Rewrite Files In-Place

**What goes wrong:** On Windows, open file handles take implicit read/write/delete locks. This breaks:
- `sed -i` — creates a temp file, renames over original; rename fails if another process holds the file open
- `cp --force` over an existing file — fails if target is open (requires clearing read-only permissions first; see uutils PR #9624)
- Log rotation tools — cannot `mv` or `rm` a file that is open elsewhere

**Specific Rust issue:** Opening a file in `append` mode on Windows acquires SYNCHRONIZE flags that conflict with exclusive locking (`LockFileEx`). Using `.write(true)` + `.seek(End)` avoids this (Rust issue #54118).

**Prevention:**
- For `sed -i`: write to temp file in same directory, then use `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING` (not rename) which has retry-on-sharing-violation semantics
- For `cp --force`: check if target has read-only attribute, clear it before overwrite (`SetFileAttributes`)
- Use the `file-guard` crate for advisory locking instead of relying on OS exclusive locking
- Add retry logic (3x with 100ms backoff) for rename operations on Windows
- Never use `.append(true)` when you need to also lock the file

**Detection:** `sed -i 's/foo/bar/' file.txt` while the file is open in Notepad — if it silently fails or produces a partial file, locking is broken.

**Phase:** Phase 2 (file mutation tools) — sed, cp, mv implementation.

---

## Moderate Pitfalls

### Pitfall 7: MAX_PATH (260 Character) Limit Silently Truncates Operations

**What goes wrong:** Windows API calls fail with `ERROR_PATH_NOT_FOUND` (code 3) or `ERROR_FILENAME_EXCED_RANGE` (code 206) for paths longer than 260 characters. Rust's std does not automatically prepend `\\?\` to enable extended paths. Tools like `find`, `cp -r`, and `tar -x` fail silently or panic on deep directory trees (common in `node_modules`).

**Prevention:**
- Add a manifest entry: `<longPathAware>true</longPathAware>` in the application manifest for each binary
- When constructing paths programmatically, use `dunce::canonicalize()` (from the `dunce` crate) instead of `std::fs::canonicalize()` — the latter prepends `\\?\` unnecessarily in many cases
- For explicit long-path construction, use `std::path::Path::new(r"\\?\").join(path)` when paths exceed 200 characters as a precaution
- Check path length before operations in recursive utilities (find, cp -r, tar)

**Detection:** Create a nested directory structure 30+ levels deep with 10-character names, then run `find . -type f` — if it silently stops listing at a certain depth, MAX_PATH is not handled.

**Phase:** Phase 2 (filesystem utilities) — add manifest to all binaries before release.

---

### Pitfall 8: MSI Installer PATH Not Reflected in Running Terminals

**What goes wrong:** After MSI installation updates the system `PATH`, existing `cmd.exe` and PowerShell windows do not see the change. Windows broadcasts `WM_SETTINGCHANGE` via `BroadcastSystemMessage`, but Explorer's message pump often does not receive it. Users open a new shell expecting to run `ls` and get "command not found."

**Specific sub-pitfalls:**
- `WiX` MSI upgrade: if `UpgradeCode` is wrong, upgrade installs side-by-side instead of replacing (wix discussion #8817)
- `RemoveExistingProducts` timing: if scheduled too early in the sequence, the new version's files are removed by the old version's uninstall
- PATH entry left behind after uninstall if the component is not correctly reference-counted
- Version numbering: Windows Installer only uses the first 3 parts of the version string for upgrade detection

**Prevention:**
- Add a post-install note in the UI: "Open a new terminal window to use the updated PATH"
- Use `SendMessageTimeout(HWND_BROADCAST, WM_SETTINGCHANGE, ...)` with `SMTO_ABORTIFHUNG` in a custom action as a best-effort refresh
- Keep `UpgradeCode` constant across all releases; change `ProductCode` per version
- Set `InstallAfterUninstall` (schedule `RemoveExistingProducts` after `InstallFiles`)
- Use WiX `<Environment>` component for PATH management — do not use custom actions for PATH
- Test fresh install, upgrade from v0.x, and uninstall + reinstall scenarios in CI

**Detection:** Install via MSI, open a pre-existing cmd window, run `ls` — if "not recognized", PATH broadcast is not working.

**Phase:** Phase 5 (installer) — design WiX component structure carefully from the first installer build.

---

### Pitfall 9: Rust Binary Size and Startup Time With Many Independent Executables

**What goes wrong:** Each Rust binary statically links the Rust standard library. With 20+ utilities each at 1-4MB, the install footprint becomes 40-80MB uncompressed. More importantly, Windows Defender performs real-time scanning on first execution of each binary, adding 200-500ms to the first invocation after install.

**Prevention:**
- Use release profile with `strip = true`, `opt-level = "z"`, `lto = true`, `codegen-units = 1`, `panic = "abort"` — reduces binary size by 40-60%
- Statically link CRT (`target-feature=+crt-static`) to avoid VCRUNTIME140.dll dependency
- Consider a "multicall binary" architecture (single `gow.exe` dispatches by argv[0] or first argument), with individual `.exe` files as thin shims — this mirrors BusyBox and uutils' approach and dramatically reduces total disk footprint
- Add GOW binaries to Windows Defender exclusion path in the MSI installer (requires admin, document clearly)
- Compress individual binaries with UPX as a packaging step (tradeoff: some AV flags UPX-compressed binaries)

**Detection:** Time `ls.exe --version` in a fresh PowerShell after install — if >200ms, AV scanning overhead is present.

**Phase:** Phase 1 (architecture decision: multicall vs. independent) — this decision is load-bearing and hard to reverse.

---

### Pitfall 10: ANSI Color Codes and Terminal Detection

**What goes wrong:** GNU tools auto-detect whether output is a TTY and suppress color when piped. On Windows, `IsTerminal` (via `std::io::IsTerminal`) returns `false` for Windows Terminal, ConEmu, and some PowerShell configurations that proxy the handle. This causes `grep --color=auto` to never produce color in terminals that do support ANSI, while `grep --color=always` injects ANSI codes into piped output breaking downstream tools.

**Specific sub-pitfalls:**
- Old cmd.exe (pre-Win10 1903) does not support VT100/ANSI by default — tools must call `SetConsoleMode(handle, ENABLE_VIRTUAL_TERMINAL_PROCESSING)` to enable it
- MSYS2/Git Bash use a pseudo-TTY layer — `IsTerminal` may return `true` even for piped input
- `--color=always` output piped to `less` without `-R` flag shows raw escape sequences

**Prevention:**
- Use the `supports-color` crate (or `colored` / `nu-ansi-term`) which handles Windows VT100 enablement and terminal detection
- Enable VT100 processing explicitly via `enable-ansi-support` crate or manual `SetConsoleMode` call at startup
- Respect `NO_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE` environment variables per the de-facto standard
- Test in: Windows Terminal, legacy cmd.exe, PowerShell, MSYS2 bash, piped to `|cat`

**Detection:** Run `grep --color=auto pattern file` in Windows Terminal — if no color appears, VT100 is not enabled.

**Phase:** Phase 1 (shared output utility crate) — all tools must use the same color/TTY detection.

---

### Pitfall 11: `find -exec` and Subprocess Spawning on Windows

**What goes wrong:** GOW issue #208. On Windows, command line argument parsing is the responsibility of each application (no `execv` kernel call). Spaces in paths passed via `-exec` are not reliably quoted, causing the subprocess to receive split arguments. Additionally, `find -exec {} \;` must spawn a new process per file, which is expensive on Windows due to process creation overhead.

**Specific issues:**
- Windows `CreateProcess` requires a single command-line string, not an argv array. Rust's `std::process::Command` handles quoting, but only if you use the builder API correctly — passing pre-built strings bypasses quoting
- Paths with spaces in `{}` substitution must be double-quoted in the assembled command line
- `find -exec cmd /c {} \;` — the `{}` substitution inside a cmd shell invocation requires additional escaping layers

**Prevention:**
- Always use `std::process::Command::new(program).args(args)` (argv array) — never assemble a command string and pass to `cmd /c`
- For `{}` substitution, wrap the substituted path in double quotes before inserting into the command string when falling back to string-mode
- Implement `-exec ... +` (batch mode, passes multiple files at once) to reduce process spawn overhead
- Test `find . -name "*.txt" -exec grep foo {} \;` with a path containing spaces

**Detection:** `find /path with spaces -type f -exec ls {} \;` — if ls receives a split argument error, subprocess quoting is broken.

**Phase:** Phase 3 (find implementation) — subprocess spawning must be tested with space-containing paths from day one.

---

### Pitfall 12: Case Sensitivity Mismatches in Filename Comparisons

**What goes wrong:** Windows NTFS is case-insensitive (but case-preserving). GNU tools running on Linux are case-sensitive. Tools that compare filenames for deduplication, sorting, or pattern matching behave differently:

- `sort -u` on filenames: on Linux `Foo` and `foo` are different; on Windows they refer to the same file
- `find -name "*.TXT"` vs `find -name "*.txt"` — on Linux these match different sets; on GOW users expect them to match the same set
- `Rust Path::starts_with()` is documented as NOT being case-insensitive on Windows (Rust issue #66260)

**Prevention:**
- For filesystem operations (find, ls), use Windows API's case-insensitive comparison (`CompareStringOrdinal` with `NORM_IGNORECASE`) rather than Rust's `==` on `OsStr`
- For text-processing tools (sort, uniq, grep), maintain case-sensitive behavior to preserve GNU compatibility — document this explicitly
- Provide `-i` flag for case-insensitive matching in grep/find as GNU does

**Detection:** Create files `foo.txt` and `FOO.txt` (impossible on NTFS — they're the same file). Test `find . -name "*.TXT"` — should match `foo.txt` on Windows; if it doesn't, case folding is missing.

**Phase:** Phase 2 (find, ls) — establish case comparison strategy before implementing filesystem traversal.

---

## Minor Pitfalls

### Pitfall 13: `sed` In-Place Edit Leaves Temp Files on Failure

**What goes wrong:** GOW issue #203. `sed -i` creates a temp file in the working directory (`sed*` prefix). If sed fails mid-write (disk full, permission error), the temp file is left behind. On some Windows configurations the rename also fails silently.

**Prevention:**
- Create the temp file in the same directory as the target file (ensures same filesystem, atomic rename)
- Use a UUID-prefixed temp name to avoid collisions
- Implement a cleanup guard (Rust `Drop` trait) that deletes the temp file if the operation fails
- Use `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` for the final rename step

**Phase:** Phase 3 (sed) — handled within the sed implementation.

---

### Pitfall 14: POSIXLY_CORRECT and Option Permutation Behavior

**What goes wrong:** When `POSIXLY_CORRECT` is set, GNU tools stop processing options at the first non-option argument. If gow-rust ignores this environment variable, scripts that depend on POSIX strict mode break. Conversely, if gow-rust does not implement GNU's default permutation, `ls file -la` (options after file argument) fails.

**Prevention:**
- Check `POSIXLY_CORRECT` at startup; propagate to the arg parsing layer
- Default to GNU permutation (options anywhere), switch to POSIX strict mode when `POSIXLY_CORRECT` is set
- Test both modes in the GNU compatibility test suite

**Phase:** Phase 1 (arg parsing foundation) — implement alongside the clap/lexopt layer.

---

### Pitfall 15: `--` End-of-Options Not Handled in All Tools

**What goes wrong:** `grep -- -pattern` should treat `-pattern` as a literal string, not an option. This is a common GNU convention. Tools that do not explicitly handle `--` as an arg terminator will mis-parse inputs that look like flags.

**Prevention:**
- Ensure all tools explicitly handle `--` as end-of-options separator
- Test with filenames starting with `-`: `ls -- -mydir`, `rm -- -f`

**Phase:** Phase 1 (shared arg parsing) — enforce as a coding standard.

---

### Pitfall 16: Binary CRT Dependency (VCRUNTIME140.dll)

**What goes wrong:** Without static CRT linking, binaries require `VCRUNTIME140.dll`. On systems without Visual C++ Redistributable installed, the binary fails with a DLL not found error. This is a silent failure for users — the binary simply won't start.

**Prevention:**
- Always build with `RUSTFLAGS="-C target-feature=+crt-static"` for release builds
- Add to `.cargo/config.toml`:
  ```toml
  [target.x86_64-pc-windows-msvc]
  rustflags = ["-C", "target-feature=+crt-static"]
  ```
- Verify the built binary has no vcruntime dependency with `dumpbin /dependents *.exe`

**Phase:** Phase 1 (build configuration) — set in CI before any binary is published.

---

### Pitfall 17: Startup Performance — Windows Defender Real-Time Scan on First Run

**What goes wrong:** Windows Defender scans new/modified executables on first run. For a suite of 20+ binaries, this means every `ls`, `grep`, `find` invocation in a fresh terminal takes 200-1000ms extra on first use. This is especially painful in script loops.

**Prevention:**
- MSI installer should add the installation directory to Defender exclusions via PowerShell post-install action: `Add-MpPreference -ExclusionPath "$InstallDir"` (requires admin; handle gracefully if denied)
- Document this step in the installer UI
- Alternatively, the multicall binary approach (one large exe, symlinks/shims) means only one binary gets the first-scan penalty

**Detection:** Run `Measure-Command { ls.exe }` 3 times; if the first invocation is 10x slower than subsequent ones, AV scanning is the cause.

**Phase:** Phase 5 (installer) — add Defender exclusion to MSI post-install action.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Architecture decision (Phase 1) | Multicall vs. individual binaries — affects binary size, AV scanning, installer complexity | Decide before writing any tool; multicall is harder to retrofit |
| Arg parsing layer (Phase 1) | Clap exit codes, option permutation, `-5` shorthand | Build `gnu_arg` abstraction over lexopt; do not expose raw clap to individual tools |
| UTF-8 / codepage (Phase 1) | Console codepage mojibake, OsStr surrogate pairs | Implement startup codepage setup and OsStr utilities before first tool |
| Filesystem tools: ls, find (Phase 2) | Symlink/junction/reparse confusion, case sensitivity | Test all three Windows link types; establish case-comparison strategy |
| Path conversion (Phase 2) | Corrupting `/c` flag into drive path | Path conversion must be context-aware, not regex-based |
| File mutation: sed, cp, mv (Phase 2-3) | File locking, temp file cleanup, rename failures | Use MoveFileExW, implement Drop-based cleanup |
| Streaming: tail -f (Phase 3) | ReadDirectoryChangesW watches directories only | Use notify-rs watching parent dir + filename filter + debounce |
| Subprocess: find -exec, xargs (Phase 3) | Argument quoting with spaces, process creation overhead | Use Command::new().args() API, never string assembly |
| Color output (all phases) | VT100 not enabled, TTY detection unreliable | Single shared color utility crate using supports-color |
| MSI installer (Phase 5) | PATH not updated in running terminals, upgrade side-by-side | Correct UpgradeCode, WM_SETTINGCHANGE custom action, clear install notes |

---

## Sources

- [uutils-args: Problems with clap](https://github.com/tertsdiepraam/uutils-args/blob/main/docs/design/problems_with_clap.md) — HIGH confidence (official project design doc)
- [uutils/coreutils: Exit code issue #3102](https://github.com/uutils/coreutils/issues/3102) — HIGH confidence
- [GOW issue #244: Path translation corrupts switches](https://github.com/bmatzelle/gow/issues/244) — HIGH confidence
- [GOW issue #75, #169: tail -f bad file descriptor](https://github.com/bmatzelle/gow/issues/75) — HIGH confidence
- [GOW issue #85: grep color broken](https://github.com/bmatzelle/gow/issues/85) — HIGH confidence
- [GOW issue #203: sed temp files](https://github.com/bmatzelle/gow/issues/203) — HIGH confidence
- [Rust issue #138688: symlink_file absolute path bug](https://github.com/rust-lang/rust/issues/138688) — HIGH confidence
- [Rust issue #56171: OsStr non-UTF-8 on Windows](https://github.com/rust-lang/rust/issues/56171) — HIGH confidence
- [Rust issue #54118: append mode breaks locking](https://github.com/rust-lang/rust/issues/54118) — HIGH confidence
- [Rust issue #66260: Path::starts_with case insensitivity](https://github.com/rust-lang/rust/issues/66260) — HIGH confidence
- [Microsoft: Maximum Path Length Limitation](https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation) — HIGH confidence (official MS docs)
- [MSI bug: PATH not updated after install](https://social.technet.microsoft.com/Forums/en-US/4db362ce-ce9c-49c8-991c-c38f5b740cb7) — MEDIUM confidence
- [notify-rs: Cross-platform filesystem notification](https://github.com/notify-rs/notify) — HIGH confidence
- [WiX upgrade side-by-side issue #8817](https://github.com/orgs/wixtoolset/discussions/8817) — MEDIUM confidence
- [Symbolic Link Pitfalls with symlink_metadata (Medium/Rustaceans)](https://medium.com/rustaceans/symbolic-link-pitfalls-with-std-fs-symlink-metadata-bfa62bdd2a37) — MEDIUM confidence
- [Windows Defender slowdown on .cargo/bin](https://github.com/rust-lang/cargo/issues/5028) — HIGH confidence (Rust official repo issue)
- [GNU coreutils test report 01/20 2026](https://github.com/uutils/coreutils/issues/10390) — HIGH confidence
- [uutils/coreutils cp force flag Windows PR #9624](https://github.com/uutils/coreutils/pull/9624) — HIGH confidence
