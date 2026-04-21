//! Thin wrapper over `filetime` for `touch` semantics.
//!
//! Per RESEARCH.md Q2: `filetime::set_symlink_file_times` ALREADY handles the
//! Windows `FILE_FLAG_OPEN_REPARSE_POINT` + `SetFileTime` pattern — no
//! `gow_core::fs` wrapper needed (this corrects CONTEXT.md D-19e's assumption).

use filetime::{set_file_times, set_symlink_file_times, FileTime};
use std::path::Path;

/// Apply atime+mtime to `path`. If `no_deref` is true, modify the symlink itself
/// rather than the target.
pub fn apply(
    path: &Path,
    atime: FileTime,
    mtime: FileTime,
    no_deref: bool,
) -> std::io::Result<()> {
    if no_deref {
        set_symlink_file_times(path, atime, mtime)
    } else {
        set_file_times(path, atime, mtime)
    }
}
