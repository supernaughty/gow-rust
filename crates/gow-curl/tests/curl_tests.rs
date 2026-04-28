//! Integration tests for gow-curl (R020: HTTP/HTTPS requests, TLS 1.2/1.3, proxy).
//!
//! Offline tests (no network required): run in CI — always included.
//! Network tests: marked #[ignore = "requires network access"] — skipped in CI.
//!
//! Run all tests (including network): `cargo test -p gow-curl -- --include-ignored`
//! Run offline only:                  `cargo test -p gow-curl`

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Helper: create a `curl` Command pointing at the cargo binary.
fn curl_cmd() -> Command {
    Command::cargo_bin("curl").expect("curl binary not found — run `cargo build -p gow-curl` first")
}

// ────────────────────────────────────────────────────────────
// Offline tests — no network access required; always run in CI
// ────────────────────────────────────────────────────────────

/// `curl --help` must exit 0 and print usage information.
#[test]
fn cli_help_exits_0() {
    curl_cmd()
        .arg("--help")
        .assert()
        .success();
}

/// `curl` with no arguments (no URL) must exit non-zero because URL is required.
#[test]
fn cli_missing_url_exits_nonzero() {
    curl_cmd()
        .assert()
        .failure();
}

// ────────────────────────────────────────────────────────────
// Network tests — require internet access; marked #[ignore]
// ────────────────────────────────────────────────────────────

/// GET http://httpbin.org/get returns a JSON body containing the request URL.
/// Verifies: basic HTTP GET works, response body is written to stdout.
#[test]
#[ignore = "requires network access"]
fn get_httpbin_returns_200() {
    curl_cmd()
        .arg("http://httpbin.org/get")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"url\""));
}

/// GET https://httpbin.org/get succeeds using Windows SChannel TLS.
/// Verifies: HTTPS with native-tls (SChannel) works without OpenSSL.
#[test]
#[ignore = "requires network access"]
fn get_https_tls_works() {
    curl_cmd()
        .arg("https://httpbin.org/get")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"url\""));
}

/// `curl -o <file> <url>` writes the response body to a file, not stdout.
/// Verifies: -o flag works; file is created and non-empty.
#[test]
#[ignore = "requires network access"]
fn output_to_file_writes_body() {
    let tmp = tempdir().unwrap();
    let output_path = tmp.path().join("response.json");

    curl_cmd()
        .args(["-o", output_path.to_str().unwrap(), "http://httpbin.org/get"])
        .assert()
        .success();

    let contents = fs::read(&output_path).expect("output file should exist");
    assert!(!contents.is_empty(), "output file should be non-empty");
}

/// `curl -I <url>` sends a HEAD request and prints response headers to stdout.
/// Verifies: -I flag works; content-type header appears in output.
#[test]
#[ignore = "requires network access"]
fn head_request_prints_headers() {
    curl_cmd()
        .args(["-I", "http://httpbin.org/get"])
        .assert()
        .success()
        .stdout(predicate::str::contains("content-type"));
}
