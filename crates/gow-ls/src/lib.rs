//! `uu_ls`: GNU `ls` — list directory contents (FILE-02).
//!
//! Covers:
//! - **D-31** — RO-bit-driven permission synthesis (rw-/r--)
//! - **D-34** — hidden-file union: dot-prefix OR `FILE_ATTRIBUTE_HIDDEN`
//! - **D-35** — exec x-bit from hardcoded extension set
//! - **D-37** — symlink/junction display as `name -> target[ [junction]]`
//! - **D-46** — `walkdir` for `-R` recursive traversal
//! - Column layout via `terminal_size` (single column when stdout is not a tty)
//!
//! Module layout:
//! - [`format`] — permission synthesis, mtime formatting, link-target rendering
//! - [`layout`] — terminal-width-aware column count computation
//! - [`recurse`] — `-R` recursion with per-directory section headers

mod format;
mod layout;
mod recurse;

use std::ffi::OsString;
use std::fs::Metadata;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, Command};

use gow_core::fs::{is_hidden, link_type, LinkType};

/// How colorization of entry names is decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Colorize only when stdout is a terminal (GNU default).
    Auto,
    /// Always emit ANSI color codes, even when piped.
    Always,
    /// Never emit ANSI color codes.
    Never,
}

/// Parsed options for a single `ls` invocation. Passed by `&ListOpts` to
/// `format_entries` / `render_row_long` / the recurse module.
#[derive(Debug, Clone)]
pub struct ListOpts {
    /// `-l`: long format (one-per-line with perms / size / mtime / name).
    pub long: bool,
    /// `-a`: include hidden entries (dot-prefix OR `FILE_ATTRIBUTE_HIDDEN`).
    pub all: bool,
    /// `-A`: almost-all — like `-a` but excludes `.` and `..` standalone rows.
    pub almost_all: bool,
    /// `-1`: force one entry per line (bypasses column layout).
    pub one_per_line: bool,
    /// `-r`: reverse the sort order.
    pub reverse: bool,
    /// `-R`: recurse into subdirectories with `{path}:` section headers.
    pub recursive: bool,
    /// `--color[=always|auto|never]`.
    pub color: ColorMode,
}

fn uu_app() -> Command {
    Command::new("ls")
        .about("GNU ls — list directory contents")
        .arg(
            Arg::new("long")
                .short('l')
                .action(ArgAction::SetTrue)
                .help("use a long listing format"),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(ArgAction::SetTrue)
                .help("do not ignore entries starting with ."),
        )
        .arg(
            Arg::new("almost-all")
                .short('A')
                .long("almost-all")
                .action(ArgAction::SetTrue)
                .help("do not list implied . and .."),
        )
        .arg(
            Arg::new("one-per-line")
                .short('1')
                .action(ArgAction::SetTrue)
                .help("list one file per line"),
        )
        .arg(
            Arg::new("recursive")
                .short('R')
                .long("recursive")
                .action(ArgAction::SetTrue)
                .help("list subdirectories recursively"),
        )
        .arg(
            Arg::new("reverse")
                .short('r')
                .long("reverse")
                .action(ArgAction::SetTrue)
                .help("reverse order while sorting"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .num_args(0..=1)
                .default_missing_value("always")
                .value_parser(["always", "auto", "never", "yes", "no"])
                .help("colorize the output; WHEN can be 'always', 'auto', or 'never'"),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);

    let color = match matches.get_one::<String>("color").map(|s| s.as_str()) {
        Some("always" | "yes") => ColorMode::Always,
        Some("never" | "no") => ColorMode::Never,
        _ => ColorMode::Auto,
    };

    let opts = ListOpts {
        long: matches.get_flag("long"),
        all: matches.get_flag("all"),
        almost_all: matches.get_flag("almost-all"),
        one_per_line: matches.get_flag("one-per-line"),
        reverse: matches.get_flag("reverse"),
        recursive: matches.get_flag("recursive"),
        color,
    };

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    let targets: Vec<String> = if operands.is_empty() {
        vec![".".to_string()]
    } else {
        operands
    };

    let mut exit_code = 0;
    for (i, t) in targets.iter().enumerate() {
        let converted = gow_core::path::try_convert_msys_path(t);
        let path = Path::new(&converted);

        let md = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("ls: cannot access '{converted}': {e}");
                exit_code = 1;
                continue;
            }
        };

        if targets.len() > 1 && i > 0 {
            println!();
        }
        if targets.len() > 1 && md.is_dir() && !opts.recursive {
            println!("{converted}:");
        }

        if md.is_dir() {
            if opts.recursive {
                let rc = recurse::list_recursive(path, &opts);
                if rc != 0 {
                    exit_code = rc;
                }
            } else if let Err(e) = format_entries(path, &opts) {
                eprintln!("ls: cannot access '{converted}': {e}");
                exit_code = 1;
            }
        } else if let Err(e) = render_single(path, &md, &opts) {
            eprintln!("ls: cannot access '{converted}': {e}");
            exit_code = 1;
        }
    }
    exit_code
}

/// List the contents of a directory, applying hidden-filter, sort, and the
/// appropriate renderer (long or column). `pub(crate)` so `recurse::list_recursive`
/// can call it after printing a `{path}:` header.
pub fn format_entries(dir: &Path, opts: &ListOpts) -> std::io::Result<()> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|r| r.ok().map(|e| e.path()))
        .filter(|p| {
            if opts.all {
                true
            } else if opts.almost_all {
                // std::fs::read_dir never yields . or .. on Windows — this
                // filter is a no-op in practice but keeps semantics explicit.
                !matches!(
                    p.file_name().and_then(|n| n.to_str()),
                    Some(".") | Some("..")
                )
            } else {
                !is_hidden(p)
            }
        })
        .collect();
    entries.sort();
    if opts.reverse {
        entries.reverse();
    }

    if opts.long {
        for e in &entries {
            let md = std::fs::symlink_metadata(e)?;
            render_row_long(e, &md, opts)?;
        }
    } else {
        render_columns(&entries, opts)?;
    }
    Ok(())
}

fn render_single(path: &Path, md: &Metadata, opts: &ListOpts) -> std::io::Result<()> {
    if opts.long {
        render_row_long(path, md, opts)?;
    } else {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        let colored = colorize_name(&name, md, path, opts);
        println!("{colored}");
    }
    Ok(())
}

fn render_row_long(path: &Path, md: &Metadata, opts: &ListOpts) -> std::io::Result<()> {
    let lt = link_type(path);
    let is_link = matches!(
        lt,
        Some(LinkType::SymlinkFile | LinkType::SymlinkDir | LinkType::Junction)
    );
    let perms = format::format_permissions(md, path, is_link);
    let size = if md.is_dir() { 0 } else { md.len() };
    let mtime = format::format_mtime(md);
    let name_display = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    let colored_name = colorize_name(&name_display, md, path, opts);
    let link_suffix = format::format_link_target(path).unwrap_or_default();

    println!("{perms} 1 - - {size:>8} {mtime} {colored_name}{link_suffix}");
    Ok(())
}

fn render_columns(entries: &[PathBuf], opts: &ListOpts) -> std::io::Result<()> {
    let use_color_for_output = should_color(opts);
    let width_opt = layout::detect_width();

    if opts.one_per_line || width_opt.is_none() {
        for e in entries {
            let md = std::fs::symlink_metadata(e).ok();
            let name = e
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| e.display().to_string());
            let colored = if use_color_for_output {
                match md.as_ref() {
                    Some(m) => colorize_name(&name, m, e, opts),
                    None => name.clone(),
                }
            } else {
                name
            };
            println!("{colored}");
        }
        return Ok(());
    }

    let width = width_opt.unwrap_or(80);
    let max_name = entries
        .iter()
        .map(|e| e.file_name().map(|s| s.to_string_lossy().len()).unwrap_or(0))
        .max()
        .unwrap_or(1);
    let cols = layout::compute_columns(width, max_name);
    let col_w = max_name + 2;

    for (i, e) in entries.iter().enumerate() {
        let md = std::fs::symlink_metadata(e).ok();
        let name = e
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| e.display().to_string());
        let colored = if use_color_for_output {
            match md.as_ref() {
                Some(m) => colorize_name(&name, m, e, opts),
                None => name.clone(),
            }
        } else {
            name.clone()
        };
        // Pad based on the RAW (uncolored) length so ANSI codes don't count.
        let pad = col_w.saturating_sub(name.len());
        print!("{colored}{}", " ".repeat(pad));
        if (i + 1) % cols == 0 {
            println!();
        }
    }
    if !entries.is_empty() && entries.len() % cols != 0 {
        println!();
    }
    Ok(())
}

fn should_color(opts: &ListOpts) -> bool {
    match opts.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => std::io::stdout().is_terminal(),
    }
}

fn colorize_name(name: &str, md: &Metadata, path: &Path, opts: &ListOpts) -> String {
    if !should_color(opts) {
        return name.to_string();
    }
    use gow_core::fs::has_executable_extension;
    // ANSI codes chosen per dircolors GNU defaults:
    //   dir=01;34 (bold blue), ln=01;36 (bold cyan), ex=01;32 (bold green)
    let code = if md.is_dir() {
        "\x1b[1;34m"
    } else if md.file_type().is_symlink() {
        "\x1b[1;36m"
    } else if link_type(path) == Some(LinkType::Junction) {
        "\x1b[1;36m"
    } else if has_executable_extension(path) {
        "\x1b[1;32m"
    } else {
        ""
    };
    if code.is_empty() {
        name.to_string()
    } else {
        format!("{code}{name}\x1b[0m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_mode_default_is_auto_mapping() {
        // Map test — runtime exercised through integration tests.
        let opts = ListOpts {
            long: false,
            all: false,
            almost_all: false,
            one_per_line: false,
            reverse: false,
            recursive: false,
            color: ColorMode::Always,
        };
        assert!(should_color(&opts));
        let opts_never = ListOpts {
            color: ColorMode::Never,
            ..opts.clone()
        };
        assert!(!should_color(&opts_never));
    }

    #[test]
    fn colorize_never_returns_plain() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("a.txt");
        std::fs::write(&f, b"x").unwrap();
        let md = std::fs::symlink_metadata(&f).unwrap();
        let opts = ListOpts {
            long: false,
            all: false,
            almost_all: false,
            one_per_line: false,
            reverse: false,
            recursive: false,
            color: ColorMode::Never,
        };
        let out = colorize_name("a.txt", &md, &f, &opts);
        assert_eq!(out, "a.txt");
    }

    #[test]
    fn colorize_always_wraps_dir_with_blue() {
        let tmp = tempfile::tempdir().unwrap();
        let md = std::fs::symlink_metadata(tmp.path()).unwrap();
        let opts = ListOpts {
            long: false,
            all: false,
            almost_all: false,
            one_per_line: false,
            reverse: false,
            recursive: false,
            color: ColorMode::Always,
        };
        let out = colorize_name("sub", &md, tmp.path(), &opts);
        assert!(out.contains("\x1b[1;34m"), "dir should emit bold-blue code");
        assert!(out.ends_with("\x1b[0m"), "dir should end with reset code");
    }

    #[test]
    fn colorize_always_wraps_exe_with_green() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("a.exe");
        std::fs::write(&f, b"x").unwrap();
        let md = std::fs::symlink_metadata(&f).unwrap();
        let opts = ListOpts {
            long: false,
            all: false,
            almost_all: false,
            one_per_line: false,
            reverse: false,
            recursive: false,
            color: ColorMode::Always,
        };
        let out = colorize_name("a.exe", &md, &f, &opts);
        assert!(out.contains("\x1b[1;32m"), "exe should emit bold-green code");
    }
}
