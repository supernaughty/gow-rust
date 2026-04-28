use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use anyhow::{Context, Result, bail};
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use bzip2::Compression as BzCompression;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression as GzCompression;
use tar::{Archive, Builder, Header};

// ─── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "tar",
    about = "GNU tar — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    /// Create a new archive
    #[arg(short = 'c', long = "create", action = ArgAction::SetTrue)]
    create: bool,

    /// Extract files from an archive
    #[arg(short = 'x', long = "extract", action = ArgAction::SetTrue)]
    extract: bool,

    /// List the contents of an archive
    #[arg(short = 't', long = "list", action = ArgAction::SetTrue)]
    list: bool,

    /// Filter the archive through gzip
    #[arg(short = 'z', long = "gzip", alias = "gunzip", action = ArgAction::SetTrue)]
    gzip: bool,

    /// Filter the archive through bzip2
    #[arg(short = 'j', long = "bzip2", action = ArgAction::SetTrue)]
    bzip2: bool,

    /// Use archive file or device ARCHIVE
    #[arg(short = 'f', long = "file")]
    file: Option<String>,

    /// Change to DIRECTORY before performing any operations
    #[arg(short = 'C', long = "directory")]
    directory: Option<String>,

    /// Verbosely list files processed
    #[arg(short = 'v', long = "verbose", action = ArgAction::SetTrue)]
    verbose: bool,

    /// Files/directories to archive or (for -x) an extract filter
    #[arg(trailing_var_arg = true)]
    paths: Vec<String>,

    /// Print help information
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    /// Print version information
    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,
}

// ─── Mode / Codec enums ───────────────────────────────────────────────────────

enum Mode {
    Create,
    Extract,
    List,
}

enum Codec {
    Plain,
    Gzip,
    Bzip2,
}

fn detect_mode(cli: &Cli) -> Result<Mode> {
    let count = [cli.create, cli.extract, cli.list]
        .iter()
        .filter(|&&b| b)
        .count();
    if count == 0 {
        bail!("You must specify one of the '-cxt' options");
    }
    if count > 1 {
        bail!("You may not specify more than one '-cxt' option");
    }
    if cli.create {
        Ok(Mode::Create)
    } else if cli.extract {
        Ok(Mode::Extract)
    } else {
        Ok(Mode::List)
    }
}

fn detect_codec(cli: &Cli) -> Codec {
    if cli.gzip {
        Codec::Gzip
    } else if cli.bzip2 {
        Codec::Bzip2
    } else {
        Codec::Plain
    }
}

// ─── Create mode ─────────────────────────────────────────────────────────────

/// Append CLI paths into a builder and flush it.
/// The `finish` closure handles codec-specific finalisation (e.g. GzEncoder::finish).
fn append_paths<W: Write, F: FnOnce(W) -> Result<()>>(
    mut builder: Builder<W>,
    cli: &Cli,
    finish: F,
) -> Result<()> {
    // CRITICAL: GNU tar default is to store symlinks, not dereference them.
    // The tar crate defaults to true (dereference). We MUST override this.
    builder.follow_symlinks(false);

    for path_str in &cli.paths {
        let converted = gow_core::path::try_convert_msys_path(path_str);
        let p = Path::new(&converted);

        if p.is_dir() {
            let name = p.file_name().unwrap_or_else(|| p.as_os_str());
            if let Err(e) = builder.append_dir_all(name, p) {
                eprintln!("tar: {converted}: {e}");
            } else if cli.verbose {
                eprintln!("{}/", converted);
            }
        } else {
            match File::open(p) {
                Ok(mut f) => {
                    let meta = f
                        .metadata()
                        .with_context(|| format!("tar: {converted}: stat"))?;
                    let mut header = Header::new_gnu();
                    header.set_metadata(&meta);
                    let name = p.file_name().unwrap_or_else(|| p.as_os_str());
                    if let Err(e) = header.set_path(name) {
                        eprintln!("tar: {converted}: {e}");
                        continue;
                    }
                    header.set_cksum();
                    if let Err(e) = builder.append(&header, &mut f) {
                        eprintln!("tar: {converted}: {e}");
                    } else if cli.verbose {
                        eprintln!("{converted}");
                    }
                }
                Err(e) => {
                    eprintln!("tar: {converted}: {e}");
                }
            }
        }
    }

    let inner = builder.into_inner()?;
    finish(inner)
}

fn run_create(cli: &Cli, codec: Codec) -> Result<()> {
    match codec {
        Codec::Gzip => {
            if let Some(ref archive_path) = cli.file {
                let out = File::create(archive_path)
                    .with_context(|| format!("tar: {archive_path}: cannot open for write"))?;
                let gz = GzEncoder::new(out, GzCompression::default());
                append_paths(Builder::new(gz), cli, |enc| {
                    enc.finish()?;
                    Ok(())
                })?;
            } else {
                let gz = GzEncoder::new(io::stdout(), GzCompression::default());
                append_paths(Builder::new(gz), cli, |enc| {
                    enc.finish()?;
                    Ok(())
                })?;
            }
        }
        Codec::Bzip2 => {
            if let Some(ref archive_path) = cli.file {
                let out = File::create(archive_path)
                    .with_context(|| format!("tar: {archive_path}: cannot open for write"))?;
                let bz = BzEncoder::new(out, BzCompression::default());
                append_paths(Builder::new(bz), cli, |enc| {
                    enc.finish()?;
                    Ok(())
                })?;
            } else {
                let bz = BzEncoder::new(io::stdout(), BzCompression::default());
                append_paths(Builder::new(bz), cli, |enc| {
                    enc.finish()?;
                    Ok(())
                })?;
            }
        }
        Codec::Plain => {
            if let Some(ref archive_path) = cli.file {
                let out = File::create(archive_path)
                    .with_context(|| format!("tar: {archive_path}: cannot open for write"))?;
                append_paths(Builder::new(out), cli, |_| Ok(()))?;
            } else {
                append_paths(Builder::new(io::stdout()), cli, |_| Ok(()))?;
            }
        }
    }
    Ok(())
}

// ─── Extract mode ────────────────────────────────────────────────────────────

fn run_extract(cli: &Cli, codec: Codec) -> Result<()> {
    let dest = cli.directory.as_deref().unwrap_or(".");

    match codec {
        Codec::Gzip => {
            if let Some(ref archive_path) = cli.file {
                let f = File::open(archive_path)
                    .with_context(|| format!("tar: {archive_path}: cannot open"))?;
                unpack_archive(Archive::new(GzDecoder::new(f)), dest, cli)?;
            } else {
                unpack_archive(Archive::new(GzDecoder::new(io::stdin())), dest, cli)?;
            }
        }
        Codec::Bzip2 => {
            if let Some(ref archive_path) = cli.file {
                let f = File::open(archive_path)
                    .with_context(|| format!("tar: {archive_path}: cannot open"))?;
                unpack_archive(Archive::new(BzDecoder::new(f)), dest, cli)?;
            } else {
                unpack_archive(Archive::new(BzDecoder::new(io::stdin())), dest, cli)?;
            }
        }
        Codec::Plain => {
            if let Some(ref archive_path) = cli.file {
                let f = File::open(archive_path)
                    .with_context(|| format!("tar: {archive_path}: cannot open"))?;
                unpack_archive(Archive::new(f), dest, cli)?;
            } else {
                unpack_archive(Archive::new(io::stdin()), dest, cli)?;
            }
        }
    }
    Ok(())
}

fn unpack_archive<R: Read>(mut archive: Archive<R>, dest: &str, cli: &Cli) -> Result<()> {
    // Use per-entry unpack_in() so we can handle symlink errors gracefully.
    // unpack_in() has the same path-traversal guard as unpack() — it skips
    // entries with ".." components (T-06-05-01 mitigation).
    //
    // T-06-05-02: On Windows, symlink extraction requires SeCreateSymbolicLinkPrivilege.
    // We log a warning and continue so the rest of the archive is still extracted.
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();

        if cli.verbose {
            eprintln!("{}", path.display());
        }

        if let Err(e) = entry.unpack_in(dest) {
            let estr = e.to_string().to_lowercase();
            if estr.contains("symlink")
                || estr.contains("privilege")
                || estr.contains("access is denied")
            {
                eprintln!(
                    "tar: warning: {}: {e} \
                     (symlink extraction may require elevated privileges on Windows)",
                    path.display()
                );
            } else {
                eprintln!("tar: {}: {e}", path.display());
            }
        }
    }
    Ok(())
}

// ─── List mode ───────────────────────────────────────────────────────────────

fn run_list(cli: &Cli, codec: Codec) -> Result<()> {
    match codec {
        Codec::Gzip => {
            if let Some(ref archive_path) = cli.file {
                let f = File::open(archive_path)
                    .with_context(|| format!("tar: {archive_path}: cannot open"))?;
                list_archive(Archive::new(GzDecoder::new(f)))?;
            } else {
                list_archive(Archive::new(GzDecoder::new(io::stdin())))?;
            }
        }
        Codec::Bzip2 => {
            if let Some(ref archive_path) = cli.file {
                let f = File::open(archive_path)
                    .with_context(|| format!("tar: {archive_path}: cannot open"))?;
                list_archive(Archive::new(BzDecoder::new(f)))?;
            } else {
                list_archive(Archive::new(BzDecoder::new(io::stdin())))?;
            }
        }
        Codec::Plain => {
            if let Some(ref archive_path) = cli.file {
                let f = File::open(archive_path)
                    .with_context(|| format!("tar: {archive_path}: cannot open"))?;
                list_archive(Archive::new(f))?;
            } else {
                list_archive(Archive::new(io::stdin()))?;
            }
        }
    }
    Ok(())
}

fn list_archive<R: Read>(mut archive: Archive<R>) -> Result<()> {
    for entry in archive.entries()? {
        let entry = entry?;
        println!("{}", entry.path()?.display());
    }
    Ok(())
}

// ─── Entry point ─────────────────────────────────────────────────────────────

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = Cli::from_arg_matches(&matches).unwrap();

    let mode = match detect_mode(&cli) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("tar: {e}");
            return 2;
        }
    };
    let codec = detect_codec(&cli);

    let result = match mode {
        Mode::Create => run_create(&cli, codec),
        Mode::Extract => run_extract(&cli, codec),
        Mode::List => run_list(&cli, codec),
    };

    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("tar: {e}");
            1
        }
    }
}
