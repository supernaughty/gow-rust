//! Windows symlink/junction abstraction layer.
//! Covers: FOUND-07

// Placeholder — full implementation in Plan 03

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
