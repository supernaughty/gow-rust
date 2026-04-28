use std::ffi::OsString;

// Force liblzma linkage to prove the static feature compiles on MSVC (compile canary).
#[allow(unused_imports)]
use liblzma::read::XzDecoder;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    eprintln!("xz: not implemented");
    1
}
