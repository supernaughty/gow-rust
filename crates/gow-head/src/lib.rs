//! `uu_head`: GNU head — stub. Real implementation arrives in Phase 3 Wave 1-5.
//!
//! Covers: TEXT-01

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("head: not yet implemented");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_one() {
        let args: Vec<OsString> = vec![OsString::from("head")];
        assert_eq!(uumain(args), 1);
    }
}
