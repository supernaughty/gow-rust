use anyhow::{Context, Result};
use clap::Parser;
use diffy::{apply, Patch};
use gow_core::fs::atomic_rewrite;
use std::ffi::OsString;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "patch",
    about = "GNU patch — Windows port.",
    version = env!("CARGO_PKG_VERSION")
)]
struct Args {
    /// Strip NUM leading path components from file names in patch (default: 1)
    #[arg(short = 'p', long = "strip", value_name = "NUM", default_value = "1")]
    strip: usize,

    /// Apply patch in reverse (undo)
    #[arg(short = 'R', long = "reverse")]
    reverse: bool,

    /// Check patch applicability without modifying files
    #[arg(long = "dry-run")]
    dry_run: bool,

    /// Read patch from FILE instead of stdin
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    input: Option<PathBuf>,

    /// Optional override file(s) to patch (normally taken from patch headers)
    files: Vec<PathBuf>,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args: Vec<OsString> = args.into_iter().collect();
    let parsed = match Args::try_parse_from(&args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    };

    match run(parsed) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("patch: {}", e);
            1
        }
    }
}

/// Strip `strip` leading path components from a path string.
///
/// Splits on '/' or '\', drops the first `strip` components, and rejoins
/// with the OS path separator.
fn strip_path(path: &str, strip: usize) -> &str {
    if strip == 0 {
        return path;
    }
    // Find the position after `strip` separators
    let mut sep_count = 0;
    let mut start = 0;
    for (i, c) in path.char_indices() {
        if c == '/' || c == '\\' {
            sep_count += 1;
            if sep_count == strip {
                start = i + 1;
                break;
            }
        }
    }
    if sep_count < strip {
        // Not enough components to strip — return the original
        // (will likely fail to find the file, which is the correct behavior)
        path
    } else {
        &path[start..]
    }
}

/// Read patch content from either stdin or the file specified by `-i`.
fn read_patch_input(args: &Args) -> Result<String> {
    match &args.input {
        Some(path) => {
            let converted = gow_core::path::try_convert_msys_path(&path.to_string_lossy());
            std::fs::read_to_string(converted)
                .with_context(|| format!("patch: {}: cannot open patch file", path.display()))
        }
        None => {
            let mut s = String::new();
            io::stdin()
                .read_to_string(&mut s)
                .context("patch: failed to read stdin")?;
            Ok(s)
        }
    }
}

fn run(args: Args) -> Result<()> {
    let patch_text = read_patch_input(&args)?;

    // Parse the patch
    let patch = Patch::from_str(&patch_text)
        .map_err(|e| anyhow::anyhow!("patch: malformed patch: {}", e))?;

    // Get the target file path from the patch header, or from positional args
    let target_path_raw = if let Some(override_file) = args.files.first() {
        override_file.to_string_lossy().into_owned()
    } else {
        // Extract filename from the appropriate patch header
        // With -R, we apply the reverse: the "modified" file is what we start from
        // and the "original" file is what we want to produce. The actual file to
        // modify is what the patch was applied to, i.e. the modified side for reverse.
        let header_path = if args.reverse {
            patch
                .modified()
                .ok_or_else(|| anyhow::anyhow!("patch: cannot determine target file from patch header (no +++ line)"))?
        } else {
            patch
                .original()
                .ok_or_else(|| anyhow::anyhow!("patch: cannot determine target file from patch header (no --- line)"))?
        };
        header_path.to_string()
    };

    // Strip leading path components
    let stripped = strip_path(&target_path_raw, args.strip);
    // Convert MSYS/Unix paths to Windows paths
    let converted = gow_core::path::try_convert_msys_path(stripped);
    let target = Path::new(&converted).to_path_buf();

    // Apply the patch (possibly reversed)
    let effective_patch = if args.reverse { patch.reverse() } else { patch };

    if args.dry_run {
        // Validate patch applicability without modifying files
        let content = std::fs::read_to_string(&target)
            .with_context(|| format!("patch: {}: cannot open file to patch", target.display()))?;

        apply(&content, &effective_patch)
            .map_err(|e| anyhow::anyhow!("patch: {} (dry-run): patch failed: {}", target.display(), e))?;

        eprintln!("checking file {}", target.display());
        return Ok(());
    }

    // Apply atomically via gow_core::fs::atomic_rewrite
    atomic_rewrite(&target, |input| {
        let content = String::from_utf8_lossy(input).into_owned();
        let patched = apply(&content, &effective_patch)
            .map_err(|e| gow_core::error::GowError::Custom(format!(
                "patch: {}: patch failed: {}",
                target.display(),
                e
            )))?;
        Ok(patched.into_bytes())
    })
    .with_context(|| format!("patch: failed to patch {}", target.display()))?;

    eprintln!("patching file {}", target.display());
    Ok(())
}
