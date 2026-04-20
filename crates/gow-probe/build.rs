//! Build script for gow-probe.
//!
//! Embeds a Windows application manifest that enables:
//! - `activeCodePage = UTF-8` (WIN-01): process-wide ANSI APIs operate on UTF-8.
//!   Complements `SetConsoleOutputCP(65001)` at runtime.
//! - `longPathAware = true` (WIN-02): bypasses the MAX_PATH 260-character limit on
//!   Windows 10+ without the `\\?\` prefix hack.
//!
//! This is a binary crate, so `embed_manifest()` is called unconditionally on
//! Windows (no bin-target gate, unlike gow-core which is lib-only). See
//! RESEARCH.md Pitfall 4 — the manifest must be embedded in EACH binary `.exe`.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest::embed_manifest(
            embed_manifest::new_manifest("Gow.Rust")
                .active_code_page(embed_manifest::manifest::ActiveCodePage::Utf8)
                .long_path_aware(embed_manifest::manifest::Setting::Enabled),
        )
        .expect("unable to embed manifest");
    }
}
