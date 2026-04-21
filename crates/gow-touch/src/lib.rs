//! `uu_touch`: GNU `touch` ported to Windows with UTF-8 + VT.
//! Stub — real implementation lands in Task 2. Task 1 introduces the `date`
//! module and its parsers (RED tests).

mod date;

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("touch: not yet implemented");
    1
}
