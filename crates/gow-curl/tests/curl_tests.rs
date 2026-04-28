// Wave 0: scaffold tests — placeholder to verify crate compiles.
// Wave 2 will replace these with real network integration tests (R020).
// Network tests are marked #[ignore] by default to avoid CI network dependency.

#[test]
fn wave0_placeholder() {
    // This test exists to verify the crate scaffolding compiles cleanly.
    // Real network integration tests are added in Wave 2.
    assert!(true);
}

#[test]
#[ignore = "requires network access"]
fn get_httpbin_returns_200() {
    // Placeholder for Wave 2 network test.
    // Will verify: curl http://httpbin.org/get returns HTTP 200.
}
