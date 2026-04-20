//! gow-core: shared Windows platform primitives for gow-rust utilities.
//!
//! Every utility binary must call `gow_core::init()` as its very first line.

pub mod args;
pub mod color;
pub mod encoding;
pub mod error;
pub mod fs;
pub mod path;

/// Initialize Windows platform primitives.
///
/// Must be called as the first line of every utility binary's main().
/// Sets UTF-8 console code pages and enables VT100/ANSI terminal mode.
pub fn init() {
    encoding::setup_console_utf8();
    color::enable_vt_mode();
}

#[cfg(test)]
mod tests {
    //! Smoke tests for the Phase 1 Plan 01 scaffold. Real functional tests arrive
    //! in Plans 02/03 when the module stubs are filled in.

    #[test]
    fn init_does_not_panic() {
        // init() wires two stubs in Plan 01. Invoking it here proves the module
        // graph links, the platform-gated code paths are reachable, and nothing
        // in the scaffolding panics at startup.
        super::init();
    }
}
