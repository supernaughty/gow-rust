//! `uu_true`: exits 0 regardless of arguments. GNU-compatible (UTIL-08, D-22).

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    0
}
