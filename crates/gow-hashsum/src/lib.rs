use std::ffi::OsString;

// Linkage canary — these imports force md-5/sha1/sha2/digest/hex to compile
// and link during the scaffold wave. Wave 4 replaces with real usage.
#[allow(unused_imports)]
use digest::Digest;
#[allow(unused_imports)]
use md5::Md5;
#[allow(unused_imports)]
use sha1::Sha1;
#[allow(unused_imports)]
use sha2::Sha256;

pub fn uumain<I: IntoIterator<Item = OsString>>(_args: I) -> i32 {
    gow_core::init();
    // Touch hex::encode at runtime so the symbol is referenced (linker keeps it).
    let _ = hex::encode([0u8; 0]);
    eprintln!("hashsum: not implemented");
    1
}
