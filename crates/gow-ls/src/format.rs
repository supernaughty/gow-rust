//! Long-format and short-format entry rendering.
//!
//! Permission synthesis per D-31: read-only bit → `r--` / `rw-`.
//! Type prefix: `d` for directory, `l` for symlink OR junction (D-37), `-` otherwise.
//! Exec x-bit per D-35: set on directories + executable extensions
//! (`.exe .cmd .bat .ps1 .com`, via `gow_core::fs::has_executable_extension`).

use std::fs::Metadata;
use std::path::Path;

use gow_core::fs::{has_executable_extension, is_readonly, link_type, normalize_junction_target, LinkType};

/// Build the 10-character permission string: `{type}{owner}{group}{other}`
/// where `type` is `d`/`l`/`-` and each triad is `rwx`-style per D-31.
///
/// - **Type prefix (D-37):** `d` if dir, `l` if `is_link` true, `-` otherwise.
/// - **rw bits (D-31):** `r` always; `w` present only when the read-only bit is clear.
/// - **x bit (D-35):** `x` for directories or files with a known executable extension.
///
/// Windows has no per-class (owner/group/other) distinction, so the triad is
/// repeated three times — matching GNU tools that "synthesize" the same bits.
pub fn format_permissions(md: &Metadata, name_path: &Path, is_link: bool) -> String {
    let mut s = String::with_capacity(10);

    // Type prefix.
    if is_link {
        s.push('l');
    } else if md.is_dir() {
        s.push('d');
    } else {
        s.push('-');
    }

    // rw bits per D-31.
    let ro = is_readonly(md);
    let w_bit = if ro { '-' } else { 'w' };
    let x_bit = if md.is_dir() || has_executable_extension(name_path) {
        'x'
    } else {
        '-'
    };

    for _ in 0..3 {
        s.push('r');
        s.push(w_bit);
        s.push(x_bit);
    }
    s
}

/// Format the `mtime` column as a deterministic `YYYY-MM-DD HH:MM` UTC-style
/// timestamp. Avoids pulling a date crate — `ls` mtime is display-only.
pub fn format_mtime(md: &Metadata) -> String {
    use std::time::UNIX_EPOCH;
    let secs = md
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    format_secs_utc(secs)
}

/// Minimal UTC-style date formatter (`YYYY-MM-DD HH:MM`). Deterministic.
fn format_secs_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let hh = rem / 3600;
    let mm = (rem % 3600) / 60;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02} {hh:02}:{mm:02}")
}

/// Convert days-from-Unix-epoch to `(year, month, day)` using Howard Hinnant's
/// civil-from-days algorithm. Pure integer arithmetic; no dependencies.
fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = y + i64::from(m <= 2);
    (y, m, d)
}

/// Return the ` -> target` (or ` -> target [junction]`) suffix for a link entry,
/// or `None` for non-links.
///
/// Per D-37:
/// - Symlink (file or dir): ` -> {target}`
/// - Junction: ` -> {target} [junction]` (with the NT `\??\` prefix stripped
///   via [`normalize_junction_target`])
pub fn format_link_target(path: &Path) -> Option<String> {
    let lt = link_type(path)?;
    match lt {
        LinkType::SymlinkFile | LinkType::SymlinkDir => {
            let t = std::fs::read_link(path).ok()?;
            Some(format!(" -> {}", t.display()))
        }
        LinkType::Junction => {
            // read_link succeeds on junctions for Rust >= 1.83; fall back to a
            // placeholder if the target cannot be read.
            let raw = std::fs::read_link(path)
                .ok()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<junction>".to_string());
            let normalized = normalize_junction_target(&raw).to_string();
            Some(format!(" -> {normalized} [junction]"))
        }
        LinkType::HardLink => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perms_regular_file_rw() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("x.txt");
        std::fs::write(&f, b"x").unwrap();
        let md = std::fs::metadata(&f).unwrap();
        let s = format_permissions(&md, &f, false);
        assert_eq!(&s[..1], "-", "regular file should start with '-'");
        assert_eq!(&s[1..4], "rw-", "default file perms should be rw-");
    }

    #[test]
    fn perms_dir_has_d_and_x() {
        let tmp = tempfile::tempdir().unwrap();
        let md = std::fs::metadata(tmp.path()).unwrap();
        let s = format_permissions(&md, tmp.path(), false);
        assert_eq!(&s[..1], "d", "dir should start with 'd'");
        assert!(s.contains('x'), "dirs should have x bit");
    }

    #[test]
    fn perms_ro_file_shows_r_dash() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("x.txt");
        std::fs::write(&f, b"x").unwrap();
        let mut p = std::fs::metadata(&f).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        {
            p.set_readonly(true);
        }
        std::fs::set_permissions(&f, p).unwrap();
        let md = std::fs::metadata(&f).unwrap();
        let s = format_permissions(&md, &f, false);
        assert_eq!(&s[1..4], "r--", "RO file owner slot should be r--");

        // Restore writable for TempDir cleanup on Windows.
        let mut p2 = std::fs::metadata(&f).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        {
            p2.set_readonly(false);
        }
        std::fs::set_permissions(&f, p2).unwrap();
    }

    #[test]
    fn perms_exe_extension_has_x() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("foo.exe");
        std::fs::write(&f, b"x").unwrap();
        let md = std::fs::metadata(&f).unwrap();
        let s = format_permissions(&md, &f, false);
        assert_eq!(s.chars().nth(3), Some('x'), "foo.exe should have x in owner slot");
    }

    #[test]
    fn perms_link_prefix_is_l() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("regular.txt");
        std::fs::write(&f, b"x").unwrap();
        let md = std::fs::metadata(&f).unwrap();
        let s = format_permissions(&md, &f, true);
        assert_eq!(&s[..1], "l", "is_link=true must emit l prefix (D-37)");
    }

    #[test]
    fn format_mtime_stable_shape() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("x.txt");
        std::fs::write(&f, b"x").unwrap();
        filetime::set_file_mtime(&f, filetime::FileTime::from_unix_time(1_500_000_000, 0)).unwrap();
        let md = std::fs::metadata(&f).unwrap();
        let s = format_mtime(&md);
        assert_eq!(s, "2017-07-14 02:40");
    }

    #[test]
    fn format_mtime_epoch() {
        // Unix epoch = 1970-01-01 00:00
        let s = format_secs_utc(0);
        assert_eq!(s, "1970-01-01 00:00");
    }

    #[test]
    fn format_mtime_recent_leap_year() {
        // 2024-02-29 12:00:00 UTC = 1709208000 (leap day)
        let s = format_secs_utc(1_709_208_000);
        assert_eq!(s, "2024-02-29 12:00");
    }

    #[test]
    fn format_link_target_none_for_regular_file() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("x.txt");
        std::fs::write(&f, b"x").unwrap();
        assert_eq!(format_link_target(&f), None);
    }
}
