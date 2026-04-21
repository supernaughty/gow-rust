//! `uu_tail`: GNU tail — stub. Real implementation arrives in Phase 3 Wave 1-5.
//!
//! Covers: TEXT-02

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("tail: not yet implemented");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_one() {
        let args: Vec<OsString> = vec![OsString::from("tail")];
        assert_eq!(uumain(args), 1);
    }
}
