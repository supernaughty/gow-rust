//! `uu_rm`: GNU rm — stub. Real implementation arrives in Phase 3 Wave 1-5.
//!
//! Covers: FILE-05

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("rm: not yet implemented");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_one() {
        let args: Vec<OsString> = vec![OsString::from("rm")];
        assert_eq!(uumain(args), 1);
    }
}
