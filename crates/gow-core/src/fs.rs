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
//!
//! # Phase 3 additions (D-47, D-36, D-38, D-34, D-35, D-45, D-42)
//!
//! Seven helpers added for Phase 3 filesystem utilities:
//! - [`atomic_rewrite`] — same-directory temp file + rename for sed/dos2unix style in-place edits (D-47)
//! - [`create_link`] / [`LinkKind`] / [`LinkOutcome`] — hard/symbolic link creation with D-36 junction fallback
//! - [`is_hidden`] — dot-prefix OR Windows FILE_ATTRIBUTE_HIDDEN (D-34)
//! - [`is_readonly`] — read-only bit check for ls -l permissions synthesis
//! - [`has_executable_extension`] — .exe/.cmd/.bat/.ps1/.com detection (D-35)
//! - [`clear_readonly`] — clear read-only attr for rm -f (D-45)
//! - [`is_drive_root`] — refuse rm / on Windows drive roots (D-42)

use std::io;
use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;

use crate::error::GowError;

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

    // Check for Windows junction (directory reparse point).
    // MUST check before is_symlink() because Rust's is_symlink() returns true
    // for junctions on Windows.
    #[cfg(target_os = "windows")]
    {
        if junction::get_target(path).is_ok() {
            return Some(LinkType::Junction);
        }
    }

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

// ==========================================================================
// Phase 3 additions (D-47, D-36, D-38, D-34, D-35, D-45, D-42).
// ==========================================================================

/// Atomically rewrite `path` by reading its bytes through `transform` and
/// writing the result back. Uses a same-directory temp file + rename so
/// readers in other processes never observe a half-written state.
///
/// # Errors
/// Source read failure, temp file creation failure, `transform` failure,
/// or `persist` failure (target locked by exclusive writer).
///
/// # Windows atomicity
/// `persist` calls `std::fs::rename` which on Windows is
/// `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` — atomic when source and
/// destination are on the same volume. D-47.
pub fn atomic_rewrite<F>(path: &Path, transform: F) -> Result<(), GowError>
where
    F: FnOnce(&[u8]) -> Result<Vec<u8>, GowError>,
{
    let original = std::fs::read(path).map_err(|e| GowError::Io {
        path: path.display().to_string(),
        source: e,
    })?;

    let transformed = transform(&original)?;

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent).map_err(|e| GowError::Io {
        path: parent.display().to_string(),
        source: e,
    })?;
    tmp.write_all(&transformed).map_err(|e| GowError::Io {
        path: tmp.path().display().to_string(),
        source: e,
    })?;
    tmp.as_file_mut().sync_all().map_err(|e| GowError::Io {
        path: tmp.path().display().to_string(),
        source: e,
    })?;

    tmp.persist(path).map_err(|e| GowError::Io {
        path: path.display().to_string(),
        source: e.error,
    })?;

    Ok(())
}

/// Kind of link to create via [`create_link`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    /// Hard link (`ln` without `-s`). Same-volume only (D-38).
    Hard,
    /// Symbolic link (`ln -s`). On directories, falls back to junction per D-36.
    Symbolic,
}

/// Outcome of a [`create_link`] call.
///
/// A `Junction` result signals to the caller that D-36 fallback occurred and
/// the caller should print a one-time stderr warning.
#[derive(Debug, PartialEq, Eq)]
pub enum LinkOutcome {
    Symlink,
    Junction,
    Hardlink,
}

/// Create a filesystem link from `target` (existing) to `link_path` (new).
///
/// # Windows D-36 behavior
/// For `LinkKind::Symbolic` where `target` is a directory:
/// 1. Try `std::os::windows::fs::symlink_dir`.
/// 2. If it fails with `ERROR_PRIVILEGE_NOT_HELD` (raw OS err 1314), fall
///    back to `junction::create`. Returns `LinkOutcome::Junction`.
/// 3. Other errors propagate as `io::Error`.
///
/// # Windows D-38 behavior
/// `LinkKind::Hard` returns `ErrorKind::CrossesDevices` (Rust 1.85+) or
/// `raw_os_error() == Some(17)` (ERROR_NOT_SAME_DEVICE) on cross-volume.
pub fn create_link(
    target: &Path,
    link_path: &Path,
    kind: LinkKind,
) -> io::Result<LinkOutcome> {
    match kind {
        LinkKind::Hard => {
            std::fs::hard_link(target, link_path)?;
            Ok(LinkOutcome::Hardlink)
        }
        LinkKind::Symbolic => {
            #[cfg(target_os = "windows")]
            {
                let target_is_dir = std::fs::metadata(target)
                    .map(|m| m.is_dir())
                    .unwrap_or(false);

                if target_is_dir {
                    use std::os::windows::fs::symlink_dir;
                    match symlink_dir(target, link_path) {
                        Ok(()) => Ok(LinkOutcome::Symlink),
                        Err(e) if e.raw_os_error() == Some(1314) => {
                            // ERROR_PRIVILEGE_NOT_HELD — D-36 fallback.
                            junction::create(target, link_path)?;
                            Ok(LinkOutcome::Junction)
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    use std::os::windows::fs::symlink_file;
                    symlink_file(target, link_path)?;
                    Ok(LinkOutcome::Symlink)
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                std::os::unix::fs::symlink(target, link_path)?;
                Ok(LinkOutcome::Symlink)
            }
        }
    }
}

/// Return true if the entry is "hidden" per D-34: dot-prefix OR Windows
/// `FILE_ATTRIBUTE_HIDDEN` set. The two conventions are treated as a union.
pub fn is_hidden(path: &Path) -> bool {
    if path
        .file_name()
        .and_then(|f| f.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
    {
        return true;
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
        if let Ok(md) = std::fs::symlink_metadata(path) {
            return md.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
        }
    }
    false
}

/// Return true if the entry is read-only. D-31 uses this to synthesize the
/// `ls -l` permissions column.
pub fn is_readonly(md: &std::fs::Metadata) -> bool {
    md.permissions().readonly()
}

/// Return true if the extension is in the gow-rust executable set (D-35):
/// `.exe`, `.cmd`, `.bat`, `.ps1`, `.com`. Case-insensitive. Does NOT
/// consult system `PATHEXT` — test determinism.
pub fn has_executable_extension(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_ascii_lowercase().as_str(),
            "exe" | "cmd" | "bat" | "ps1" | "com"
        ),
        None => false,
    }
}

/// Clear the `FILE_ATTRIBUTE_READONLY` bit. Used by `rm -f` (D-45) and
/// `cp -p` when the source was read-only.
///
/// On Windows the read-only bit is a single-bit attribute independent of the
/// Unix mode word — clearing it via `Permissions::set_readonly(false)` just
/// toggles that bit, which is exactly what we want. The clippy warning about
/// world-writable Unix files does not apply here because this function is a
/// no-op on non-Windows targets.
#[allow(clippy::permissions_set_readonly_false)]
pub fn clear_readonly(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let md = std::fs::metadata(path)?;
        let mut perms = md.permissions();
        perms.set_readonly(false);
        std::fs::set_permissions(path, perms)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Ok(())
    }
}

/// Return true if `path` points at a Windows drive root (`C:\`, `C:/`,
/// `C:`) or UNC share root (`\\server\share`). Used by `rm` to refuse
/// destructive operations per D-42.
pub fn is_drive_root(path: &Path) -> bool {
    let s = path.to_string_lossy();

    if s.len() >= 2 && s.as_bytes()[1] == b':' {
        let after = &s[2..];
        if after.is_empty() || after == "\\" || after == "/" {
            return true;
        }
        // No other drive-letter form counts; C:\Windows etc. fall through.
        return false;
    }

    // UNC \\server\share or \\server\share\
    if s.starts_with(r"\\") || s.starts_with("//") {
        let tail = s.trim_start_matches(['\\', '/']).trim_end_matches(['\\', '/']);
        let comps: Vec<&str> = tail.split(['\\', '/']).filter(|c| !c.is_empty()).collect();
        if comps.len() == 2 {
            return true;
        }
    }

    false
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

    // =========================================================================
    // Phase 3 tests — atomic_rewrite, create_link, is_hidden, is_readonly,
    // has_executable_extension, clear_readonly, is_drive_root.
    // =========================================================================

    #[test]
    fn test_atomic_rewrite_roundtrip() {
        let tmp = TempDir::new().expect("tempdir");
        let p = tmp.path().join("x.txt");
        fs::write(&p, b"hello world").unwrap();

        atomic_rewrite(&p, |bytes| Ok(bytes.to_vec())).unwrap();

        assert_eq!(fs::read(&p).unwrap(), b"hello world");
    }

    #[test]
    fn test_atomic_rewrite_transform() {
        let tmp = TempDir::new().expect("tempdir");
        let p = tmp.path().join("x.txt");
        fs::write(&p, b"abc").unwrap();

        atomic_rewrite(&p, |bytes| Ok(bytes.to_ascii_uppercase())).unwrap();

        assert_eq!(fs::read(&p).unwrap(), b"ABC");
    }

    #[test]
    fn test_atomic_rewrite_transform_error_preserves_original() {
        let tmp = TempDir::new().expect("tempdir");
        let p = tmp.path().join("x.txt");
        fs::write(&p, b"intact").unwrap();

        let result = atomic_rewrite(&p, |_| Err(GowError::Custom("nope".into())));
        assert!(result.is_err());
        assert_eq!(fs::read(&p).unwrap(), b"intact");
    }

    #[test]
    fn test_create_link_hard_same_volume() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("target.txt");
        let link = tmp.path().join("link.txt");
        fs::write(&target, b"content").unwrap();

        let outcome = create_link(&target, &link, LinkKind::Hard).expect("hard link");
        assert_eq!(outcome, LinkOutcome::Hardlink);
        assert_eq!(fs::read(&link).unwrap(), b"content");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_create_link_symlink_file_privilege_skip() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("target.txt");
        let link = tmp.path().join("link.txt");
        fs::write(&target, b"data").unwrap();

        match create_link(&target, &link, LinkKind::Symbolic) {
            Ok(LinkOutcome::Symlink) => {
                assert_eq!(fs::read(&link).unwrap(), b"data");
            }
            Err(e) if e.raw_os_error() == Some(1314) => {
                eprintln!("[skip] symlink privilege missing");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_create_link_symlink_dir_junction_fallback() {
        let tmp = TempDir::new().expect("tempdir");
        let target_dir = tmp.path().join("target_dir");
        let link = tmp.path().join("link_dir");
        fs::create_dir(&target_dir).unwrap();
        fs::write(target_dir.join("inside.txt"), b"hi").unwrap();

        // Under D-36: either Symlink (privileged) OR Junction (fallback) is acceptable.
        match create_link(&target_dir, &link, LinkKind::Symbolic) {
            Ok(LinkOutcome::Symlink) | Ok(LinkOutcome::Junction) => {
                // Both outcomes are valid per D-36.
                assert!(link.exists(), "link path must exist after successful create");
            }
            Ok(other) => panic!("unexpected outcome for dir symlink: {other:?}"),
            Err(e) => panic!("create_link failed: {e}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_create_link_symlink_unix() {
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("target.txt");
        let link = tmp.path().join("link.txt");
        fs::write(&target, b"data").unwrap();

        let outcome = create_link(&target, &link, LinkKind::Symbolic).unwrap();
        assert_eq!(outcome, LinkOutcome::Symlink);
    }

    #[test]
    fn test_is_hidden_dot_prefix() {
        let tmp = TempDir::new().expect("tempdir");
        let p = tmp.path().join(".hidden");
        fs::write(&p, b"").unwrap();
        assert!(is_hidden(&p));
    }

    #[test]
    fn test_is_hidden_not_hidden() {
        let tmp = TempDir::new().expect("tempdir");
        let p = tmp.path().join("visible.txt");
        fs::write(&p, b"").unwrap();
        assert!(!is_hidden(&p));
    }

    #[test]
    fn test_is_readonly_default() {
        let tmp = TempDir::new().expect("tempdir");
        let p = tmp.path().join("file.txt");
        fs::write(&p, b"").unwrap();
        let md = fs::metadata(&p).unwrap();
        assert!(!is_readonly(&md));
    }

    // Test-only cleanup path needs set_readonly(false) to allow TempDir::drop
    // to succeed on Windows (RO files block deletion). Unix tests don't touch
    // the RO bit meaningfully; the clippy lint about world-writable files is
    // a theoretical concern that doesn't apply to ephemeral tempdir content.
    #[allow(clippy::permissions_set_readonly_false)]
    #[test]
    fn test_is_readonly_after_set() {
        let tmp = TempDir::new().expect("tempdir");
        let p = tmp.path().join("file.txt");
        fs::write(&p, b"").unwrap();
        let md = fs::metadata(&p).unwrap();
        let mut perms = md.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&p, perms).unwrap();

        let md2 = fs::metadata(&p).unwrap();
        assert!(is_readonly(&md2));

        // Restore writable so TempDir::drop can clean up on Windows (RO files
        // in the temp dir refuse deletion).
        let mut perms2 = md2.permissions();
        perms2.set_readonly(false);
        fs::set_permissions(&p, perms2).unwrap();
    }

    #[test]
    fn test_has_executable_extension() {
        assert!(has_executable_extension(Path::new("a.exe")));
        assert!(has_executable_extension(Path::new("A.EXE")));
        assert!(has_executable_extension(Path::new("foo.EXE")));
        assert!(has_executable_extension(Path::new("a.ps1")));
        assert!(has_executable_extension(Path::new("a.cmd")));
        assert!(has_executable_extension(Path::new("a.bat")));
        assert!(has_executable_extension(Path::new("a.com")));
        assert!(!has_executable_extension(Path::new("a.txt")));
        assert!(!has_executable_extension(Path::new("a")));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_clear_readonly() {
        let tmp = TempDir::new().expect("tempdir");
        let p = tmp.path().join("file.txt");
        fs::write(&p, b"").unwrap();

        // Set read-only.
        let md = fs::metadata(&p).unwrap();
        let mut perms = md.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&p, perms).unwrap();
        assert!(is_readonly(&fs::metadata(&p).unwrap()));

        // Clear it.
        clear_readonly(&p).unwrap();
        assert!(!is_readonly(&fs::metadata(&p).unwrap()));
    }

    #[test]
    fn test_is_drive_root_variants() {
        assert!(is_drive_root(Path::new("C:\\")));
        assert!(is_drive_root(Path::new("C:/")));
        assert!(is_drive_root(Path::new("C:")));
        assert!(is_drive_root(Path::new("Z:\\")));
        assert!(!is_drive_root(Path::new("C:\\Windows")));
        assert!(!is_drive_root(Path::new("C:\\Windows\\")));
        assert!(is_drive_root(Path::new(r"\\server\share")));
        assert!(is_drive_root(Path::new(r"\\server\share\")));
        assert!(!is_drive_root(Path::new(r"\\server\share\sub")));
        assert!(!is_drive_root(Path::new(r"\\server")));
    }
}
