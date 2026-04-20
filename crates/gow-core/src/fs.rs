//! Windows symlink and junction abstraction layer.
//!
//! Windows has three types of filesystem links that GNU tools need to handle:
//! - Symbolic links (files and directories) — created by `mklink`
//! - Junctions — directory-only links (no privilege required in modern Windows)
//! - Hard links — same inode, different directory entries
//!
//! `std::fs::symlink_metadata()` returns the metadata of the link itself
//! (not the target), which is what GNU tools need for `ls -l` display.
//!
//! Covers: FOUND-07

use std::path::Path;

/// The type of a Windows filesystem link.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LinkType {
    /// A symbolic link pointing to a file.
    SymlinkFile,
    /// A symbolic link pointing to a directory.
    SymlinkDir,
    /// A directory junction (Windows-specific; no SeCreateSymbolicLink privilege required).
    Junction,
    /// A hard link (same file content, different directory entry).
    HardLink,
}

/// Determine the link type for a path, or `None` if the path is not a link.
///
/// Uses `symlink_metadata()` to inspect the link itself without following it.
/// Returns `None` if the path does not exist or an I/O error occurs.
///
/// # Note on hard links
/// Hard links cannot be reliably detected on Windows using metadata alone
/// (the `nlink` count is not exposed by `std::fs` in a cross-platform way).
/// This function will not return `LinkType::HardLink` on stable Rust today;
/// the variant is reserved for future use when the API stabilizes.
pub fn link_type(path: &Path) -> Option<LinkType> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    let file_type = meta.file_type();

    if file_type.is_symlink() {
        // Determine whether the symlink target is a directory or a file.
        // Follow the link to check the target type. If the target does not exist
        // (broken symlink), treat as SymlinkFile.
        if std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false) {
            return Some(LinkType::SymlinkDir);
        } else {
            return Some(LinkType::SymlinkFile);
        }
    }

    // Check for Windows junction (directory reparse point).
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        // FILE_ATTRIBUTE_REPARSE_POINT = 0x400
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
        if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Some(LinkType::Junction);
        }
    }

    None
}

/// Strip the `\??\` device path prefix from a junction target.
///
/// When `DeviceIoControl` or `GetFinalPathNameByHandleW` returns a junction
/// target, it includes the NT device path prefix: `\??\C:\target`.
/// This function strips that prefix to produce a user-displayable path.
///
/// # Examples
/// ```
/// use gow_core::fs::normalize_junction_target;
/// assert_eq!(normalize_junction_target(r"\??\C:\target"), r"C:\target");
/// assert_eq!(normalize_junction_target(r"C:\already\clean"), r"C:\already\clean");
/// ```
pub fn normalize_junction_target(raw: &str) -> &str {
    raw.strip_prefix(r"\??\").unwrap_or(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_link_type_none_for_regular_file() {
        let dir = TempDir::new().expect("tempdir");
        let file = dir.path().join("regular.txt");
        fs::write(&file, b"hello").expect("write");
        assert_eq!(link_type(&file), None);
    }

    #[test]
    fn test_link_type_none_for_nonexistent_path() {
        let path = std::path::PathBuf::from("this_path_does_not_exist_9f3k2j");
        assert_eq!(link_type(&path), None);
    }

    #[test]
    fn test_link_type_none_for_directory() {
        let dir = TempDir::new().expect("tempdir");
        assert_eq!(link_type(dir.path()), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_link_type_file_symlink() {
        use std::os::windows::fs::symlink_file;
        let dir = TempDir::new().expect("tempdir");
        let target = dir.path().join("target.txt");
        let link = dir.path().join("link.txt");
        fs::write(&target, b"content").expect("write target");
        // CreateSymbolicLink requires SeCreateSymbolicLinkPrivilege or Developer Mode.
        // Skip the test if symlink creation fails (common in CI without privilege).
        if symlink_file(&target, &link).is_ok() {
            assert_eq!(link_type(&link), Some(LinkType::SymlinkFile));
        } else {
            eprintln!("Skipping symlink test: insufficient privilege");
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_link_type_file_symlink_unix() {
        use std::os::unix::fs::symlink;
        let dir = TempDir::new().expect("tempdir");
        let target = dir.path().join("target.txt");
        let link = dir.path().join("link.txt");
        fs::write(&target, b"content").expect("write target");
        symlink(&target, &link).expect("create symlink");
        assert_eq!(link_type(&link), Some(LinkType::SymlinkFile));
    }

    #[test]
    fn test_normalize_junction_target_strips_prefix() {
        assert_eq!(
            normalize_junction_target(r"\??\C:\target"),
            r"C:\target"
        );
    }

    #[test]
    fn test_normalize_junction_target_already_clean() {
        assert_eq!(
            normalize_junction_target(r"C:\already\clean"),
            r"C:\already\clean"
        );
    }

    #[test]
    fn test_link_type_enum_equality() {
        assert_eq!(LinkType::SymlinkFile, LinkType::SymlinkFile);
        assert_ne!(LinkType::SymlinkFile, LinkType::SymlinkDir);
        assert_ne!(LinkType::SymlinkDir, LinkType::Junction);
    }
}
