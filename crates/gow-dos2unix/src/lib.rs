//! `uu_dos2unix`: Convert CRLF to LF — stub. Real implementation arrives in Phase 3 Wave 1-5.
//!
//! Covers: CONV-01

pub mod transform;

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("dos2unix: not yet implemented");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_one() {
        let args: Vec<OsString> = vec![OsString::from("dos2unix")];
        assert_eq!(uumain(args), 1);
    }
}
