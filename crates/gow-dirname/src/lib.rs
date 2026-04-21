//! `uu_dirname`: GNU `dirname` ported to Windows with UTF-8 + VT.
//! Stub — real implementation lands in a later Wave 2/3/4 plan.

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("dirname: not yet implemented");
    1
}
