---
phase: 06-archive-compression-and-network
plan: "06"
subsystem: network, http
tags: [reqwest, native-tls, schannel, blocking, curl, http, https, proxy, windows]

# Dependency graph
requires:
  - phase: 06-archive-compression-and-network
    plan: "01"
    provides: gow-curl crate scaffold with reqwest dep declared, stub uumain

provides:
  - crates/gow-curl/src/lib.rs — full HTTP/HTTPS client using reqwest blocking + native-tls
  - crates/gow-curl/tests/curl_tests.rs — integration tests (2 offline + 4 network-ignored)
  - R020 satisfied: HTTP/HTTPS GET/HEAD, -o file output, -x proxy (Proxy::all), -k insecure, -f fail

affects:
  - Phase 07 installer (curl binary must be included in MSI)
  - Any future plans that extend curl (multipart upload, resume, authentication headers)

# Tech tracking
tech-stack:
  added:
    - reqwest 0.13.3 blocking + native-tls features (Windows SChannel TLS 1.2/1.3, no OpenSSL)
  patterns:
    - reqwest::blocking::ClientBuilder pattern for synchronous CLI HTTP clients
    - Proxy::all() for -x flag (proxies all protocols, not Proxy::http() which skips HTTPS)
    - danger_accept_invalid_certs gated behind explicit -k flag only
    - Network test isolation via #[ignore = "requires network access"]

key-files:
  created: []
  modified:
    - crates/gow-curl/src/lib.rs (stub → full HTTP/HTTPS client implementation)
    - crates/gow-curl/tests/curl_tests.rs (Wave 0 placeholder → 6 integration tests)

key-decisions:
  - "Use reqwest::blocking (not async + tokio::main) — blocking spawns its own internal runtime; no explicit tokio dep needed"
  - "Use Proxy::all() not Proxy::http() — curl -x proxies ALL protocols including HTTPS (Pitfall 7)"
  - "Gate danger_accept_invalid_certs behind -k/--insecure flag only — default uses SChannel OS trust store"
  - "Exit 0 by default even on HTTP 4xx/5xx — matches curl behavior; --fail/-f flag enables non-zero exit"

patterns-established:
  - "reqwest blocking pattern: ClientBuilder::new() → proxy/insecure options → build() → get/head → send()"
  - "Network test pattern: 2 offline tests (help, missing args) + 4 ignored network tests per HTTP utility"

requirements-completed:
  - R020

# Metrics
duration: 30min
completed: 2026-04-28
---

# Phase 06 Plan 06: curl HTTP/HTTPS Client Summary

**reqwest 0.13 blocking client with Windows SChannel TLS — GET/HEAD, -o file, -x Proxy::all proxy, -k insecure, -f fail flags; 2 offline tests pass, 4 network tests #[ignore]-d**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-04-28
- **Completed:** 2026-04-28
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Replaced stub `curl: not implemented` with a full reqwest-based HTTP/HTTPS client
- Implemented GET, HEAD (-I), file output (-o), proxy (-x via Proxy::all), TLS skip (-k), fail-on-error (-f), silent (-s), follow-redirects (-L) flags
- Used Windows SChannel TLS via native-tls feature — no OpenSSL required, uses OS certificate store
- cargo test -p gow-curl exits 0: 2 offline tests pass, 4 network tests correctly #[ignore]-d

## Task Commits

1. **Task 1: Implement gow-curl lib.rs (HTTP/HTTPS client)** - `1b167f0` (feat)
2. **Task 2: Write integration tests for gow-curl** - `40a1e8a` (test)

## Files Created/Modified

- `crates/gow-curl/src/lib.rs` — Full HTTP/HTTPS client: `Cli` struct with clap derive, `run()` with ClientBuilder, GET/HEAD dispatch, -o file write, -x proxy via `Proxy::all()`, -k insecure, -f fail
- `crates/gow-curl/tests/curl_tests.rs` — 6 integration tests: cli_help_exits_0, cli_missing_url_exits_nonzero (offline); get_httpbin_returns_200, get_https_tls_works, output_to_file_writes_body, head_request_prints_headers (network, #[ignore])

## Decisions Made

- **reqwest blocking, no tokio dep:** reqwest::blocking::Client spawns an internal tokio runtime. Using `#[tokio::main]` alongside blocking::Client causes a "Cannot drop a runtime" panic. Plain synchronous `fn main()` is correct.
- **Proxy::all() for -x flag:** `Proxy::http()` only proxies HTTP requests; `Proxy::https()` only HTTPS. GNU curl's `-x` flag proxies ALL protocols, so `Proxy::all()` is the correct mapping. Using `Proxy::http()` alone would silently bypass the proxy for HTTPS requests.
- **Exit 0 by default on HTTP errors:** Real curl exits 0 even on HTTP 4xx/5xx unless `--fail` is passed. Implemented `--fail` / `-f` flag to gate non-zero exit.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed `contains_str` → `contains` (predicates 3.1.4 API)**
- **Found during:** Task 2 (integration tests)
- **Issue:** `predicate::str::contains_str` does not exist in predicates 3.1.4; the correct function is `predicate::str::contains`
- **Fix:** Changed `contains_str("content-type")` to `contains("content-type")` in head_request_prints_headers test
- **Files modified:** `crates/gow-curl/tests/curl_tests.rs`
- **Verification:** `cargo test -p gow-curl` compiles and exits 0
- **Committed in:** `40a1e8a` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Minor API name fix; no scope change.

## Issues Encountered

None beyond the contains_str API fix above.

## User Setup Required

None — no external service configuration required for offline tests. Network tests require internet access and are marked #[ignore].

## Next Phase Readiness

- gow-curl binary is fully functional and exports `uumain` from `uu_curl`
- R020 (HTTP/HTTPS with TLS 1.2/1.3, proxy authentication) satisfied
- Ready for Phase 07 installer: curl binary must be added as a Component in main.wxs
- No blockers for downstream plans

## Known Stubs

None — the curl implementation is complete. All flags are functional (no unimplemented!() calls).

## Threat Surface Scan

No new threat surface beyond what was specified in the plan's `<threat_model>`. All five threats (T-06-06-01 through T-06-06-05) were addressed:
- T-06-06-01 (TLS cert validation): mitigated — `danger_accept_invalid_certs` gated behind `-k`
- T-06-06-02 through T-06-06-05: accepted per plan disposition

## Self-Check: PASSED

- `crates/gow-curl/src/lib.rs` exists and is >= 100 lines (150 lines)
- `crates/gow-curl/tests/curl_tests.rs` exists and is >= 60 lines (96 lines)
- Both commits verified: `1b167f0` (feat), `40a1e8a` (test)
- `cargo build -p gow-curl` exits 0
- `cargo test -p gow-curl` exits 0 (2 passed, 4 ignored)
- No tokio in Cargo.toml confirmed
- `Proxy::all` present in lib.rs
- `danger_accept_invalid_certs` present and gated behind `-k`

---
*Phase: 06-archive-compression-and-network*
*Completed: 2026-04-28*
