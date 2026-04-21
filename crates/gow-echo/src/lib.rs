//! `uu_echo`: GNU `echo` ported to Windows with UTF-8 + VT.
//! Stub — real implementation lands in Task 2 of plan 02-03.

mod escape;

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("echo: not yet implemented");
    1
}
