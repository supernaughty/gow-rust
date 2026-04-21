//! Recursive listing with per-directory section headers (GNU `ls -R` format).
//!
//! Strategy: walk the tree once with `walkdir` (deterministically sorted via
//! `sort_by_file_name`) to collect every directory in lexical order, then
//! dispatch to `format_entries` for each one. Each directory is preceded by a
//! `{path}:` header and separated from the previous listing by a blank line.

use std::path::Path;

use walkdir::WalkDir;

use crate::{format_entries, ListOpts};

/// Emit the recursive listing for `root`. Returns the exit code: `0` on full
/// success, `1` if any directory could not be accessed. Matches GNU `ls -R`
/// which continues past inaccessible directories instead of aborting.
pub fn list_recursive(root: &Path, opts: &ListOpts) -> i32 {
    let mut exit_code = 0;
    let mut dirs_to_list = vec![root.to_path_buf()];

    for entry in WalkDir::new(root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
    {
        match entry {
            Ok(e) => {
                if e.file_type().is_dir() && e.path() != root {
                    dirs_to_list.push(e.path().to_path_buf());
                }
            }
            Err(err) => {
                let p = err
                    .path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                let msg = err
                    .io_error()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| err.to_string());
                eprintln!("ls: cannot access '{p}': {msg}");
                exit_code = 1;
            }
        }
    }

    for (i, d) in dirs_to_list.iter().enumerate() {
        if i > 0 {
            println!();
        }
        println!("{}:", d.display());
        if let Err(e) = format_entries(d, opts) {
            eprintln!("ls: cannot access '{}': {e}", d.display());
            exit_code = 1;
        }
    }

    exit_code
}
