//! Build script for gow-core.
//!
//! Embeds a Windows application manifest that enables:
//! - `activeCodePage = UTF-8` (WIN-01): process-wide ANSI APIs (`fopen`, `CreateFileA`, etc.)
//!   operate on UTF-8. Complements `SetConsoleOutputCP(65001)` at runtime.
//! - `longPathAware = true` (WIN-02): bypasses the MAX_PATH 260-character limit on
//!   Windows 10+ without the `\\?\` prefix hack.
//!
//! IMPORTANT (see RESEARCH.md Pitfall 4): a Windows application manifest must be
//! embedded in EACH binary `.exe` — Windows reads it from the PE resource section at
//! process startup. Linking a library with an embedded manifest does NOT propagate
//! the manifest to its consumers.
//!
//! `embed-manifest` emits `cargo:rustc-link-arg-bins=...` directives, which cargo
//! only accepts on packages that define at least one `[[bin]]` target. Since
//! `gow-core` is a pure library crate in Phase 1, we skip the embed call here and
//! require every utility crate from Phase 2 onward to replicate this build script
//! (see Pattern 8 in RESEARCH.md). The full manifest call is retained below in the
//! gated branch so that copy-paste into utility crates is mechanical.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Only embed on Windows builds that actually produce a binary. See module-level
    // doc above — `gow-core` is lib-only, so this branch never runs for gow-core,
    // but the literal call is what utility crates (Phase 2+) will inline into their
    // own build.rs files.
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() && has_bin_target() {
        embed_manifest::embed_manifest(
            embed_manifest::new_manifest("Gow.Rust")
                .active_code_page(embed_manifest::manifest::ActiveCodePage::Utf8)
                .long_path_aware(embed_manifest::manifest::Setting::Enabled),
        )
        .expect("unable to embed manifest");
    }
}

/// Returns true if this crate's Cargo.toml declares at least one `[[bin]]` target
/// (or a `src/bin/` directory, or a default `src/main.rs`). When false, the
/// `embed-manifest` crate's `cargo:rustc-link-arg-bins=...` directive would be
/// rejected by cargo as "invalid instruction" because there is no bin target to
/// attach it to.
fn has_bin_target() -> bool {
    let manifest_dir = match std::env::var_os("CARGO_MANIFEST_DIR") {
        Some(v) => std::path::PathBuf::from(v),
        None => return false,
    };

    // src/main.rs or src/bin/ implies a bin target even without [[bin]] in Cargo.toml.
    if manifest_dir.join("src").join("main.rs").exists() {
        return true;
    }
    if manifest_dir.join("src").join("bin").is_dir() {
        return true;
    }

    // Otherwise scan Cargo.toml for a `[[bin]]` table header.
    let cargo_toml = manifest_dir.join("Cargo.toml");
    match std::fs::read_to_string(&cargo_toml) {
        Ok(text) => text.lines().any(|line| line.trim_start().starts_with("[[bin]]")),
        Err(_) => false,
    }
}
