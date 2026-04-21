//! `uu_env`: GNU `env` ported to Windows with UTF-8 + VT.
//! Stub (RED phase) — real `uumain` lands in Task 2.

#[allow(dead_code)] // Wired up in Task 2; public once lib.rs calls split::split.
mod split;

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("env: not yet implemented");
    1
}
