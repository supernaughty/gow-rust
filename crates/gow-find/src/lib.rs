//! `uu_find`: GNU find — Windows port (R015 / FIND-01, FIND-02, FIND-03).
//!
//! This file is a SCAFFOLD STUB. Plan 05-02 replaces this body with the real
//! implementation (clap Cli struct, WalkDir traversal, predicates, -exec).

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("find: not implemented");
    1
}
