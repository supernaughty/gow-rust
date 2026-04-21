//! `uu_which`: GNU `which` ported to Windows with UTF-8 + VT.
//! Real implementation landing in Task 2 — Task 1 only introduces the `pathext` module.

mod pathext;

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("which: not yet implemented");
    1
}
