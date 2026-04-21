//! `uu_tee`: GNU `tee` ported to Windows with UTF-8 + VT.
//! Stub — real uumain body lands in Task 2 of this plan; the `mod signals;`
//! declaration here makes the platform-gated Ctrl+C helper compilable and
//! unit-testable independently of the split-writer loop.

mod signals;

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("tee: not yet implemented");
    1
}
