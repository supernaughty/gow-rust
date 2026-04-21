//! `uu_ln`: GNU ln — stub. Real implementation arrives in Phase 3 Wave 1-5.
//!
//! Covers: FILE-09

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("ln: not yet implemented");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_one() {
        let args: Vec<OsString> = vec![OsString::from("ln")];
        assert_eq!(uumain(args), 1);
    }
}
