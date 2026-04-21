//! `uu_false`: exits 1 regardless of arguments. GNU-compatible (UTIL-09, D-22).

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    1
}
