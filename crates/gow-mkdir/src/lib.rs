//! `uu_mkdir`: GNU `mkdir` ported to Windows with UTF-8 + VT.
//! Stub — real implementation lands in a later Wave 2/3/4 plan.

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("mkdir: not yet implemented");
    1
}
