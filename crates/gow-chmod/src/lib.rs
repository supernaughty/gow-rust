//! `uu_chmod`: GNU chmod — stub. Real implementation arrives in Phase 3 Wave 1-5.
//!
//! Covers: FILE-10

use std::ffi::OsString;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("chmod: not yet implemented");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_one() {
        let args: Vec<OsString> = vec![OsString::from("chmod")];
        assert_eq!(uumain(args), 1);
    }
}
