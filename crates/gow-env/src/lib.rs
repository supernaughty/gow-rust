//! `uu_env`: GNU `env` ported to Windows with UTF-8 + VT.
//! Stub (RED phase) — real `uumain` lands in Task 2.

mod split;

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("env: not yet implemented");
    1
}
