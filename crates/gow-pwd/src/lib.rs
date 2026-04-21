//! `uu_pwd`: GNU `pwd` ported to Windows with UTF-8 + VT.
//! Stub — canonical module wired for Task 1 unit tests; full uumain lands in Task 2.

mod canonical;

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("pwd: not yet implemented");
    1
}
