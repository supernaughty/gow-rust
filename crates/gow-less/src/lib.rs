//! `uu_less`: GNU less — Windows port (R017 / LESS-01).
//!
//! This file is a SCAFFOLD STUB. Plan 05-04 replaces this body with the real
//! pager (crossterm raw mode, LineIndex, event loop).

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("less: not implemented");
    1
}
