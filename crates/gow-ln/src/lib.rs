//! `uu_ln`: GNU ln — create hard and symbolic links (FILE-09).
//!
//! Covers: FILE-09

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;

use gow_core::fs::{create_link, link_type, LinkKind, LinkOutcome, LinkType};

#[derive(Parser, Debug)]
#[command(
    name = "ln",
    about = "Create a link to TARGET with the name LINK_NAME.",
    version,
    override_usage = "ln [OPTION]... [-T] TARGET LINK_NAME\n    ln [OPTION]... TARGET\n    ln [OPTION]... TARGET... DIRECTORY\n    ln [OPTION]... -t DIRECTORY TARGET..."
)]
struct Args {
    /// make symbolic links instead of hard links
    #[arg(short, long)]
    symbolic: bool,

    /// remove existing destination files
    #[arg(short, long)]
    force: bool,

    /// explained what is being done
    #[arg(short, long)]
    verbose: bool,

    /// treat LINK_NAME as a normal file if it is a symbolic link to a directory
    #[arg(short = 'n', long = "no-dereference")]
    no_dereference: bool,

    /// treat LINK_NAME as a normal file always
    #[arg(short = 'T', long = "no-target-directory")]
    no_target_directory: bool,

    /// specify the DIRECTORY in which to create the links
    #[arg(short = 't', long = "target-directory", value_name = "DIRECTORY")]
    target_directory: Option<String>,

    /// Target sources and destination
    #[arg(required = true, value_name = "PATH")]
    paths: Vec<String>,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args: Vec<OsString> = args.into_iter().collect();
    let args_strings: Vec<String> = args
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let normalized = gow_core::path::normalize_file_args(&args_strings);

    let parsed = match Args::try_parse_from(normalized) {
        Ok(a) => a,
        Err(e) => {
            if e.use_stderr() {
                eprintln!("{}", e);
                return 1;
            } else {
                println!("{}", e);
                return 0;
            }
        }
    };

    match run(parsed) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("ln: {}", e);
            1
        }
    }
}

fn run(args: Args) -> Result<i32> {
    let mut exit_code = 0;

    let (targets, dest) = if let Some(ref tdir) = args.target_directory {
        (args.paths.as_slice(), Some(Path::new(tdir)))
    } else if args.no_target_directory {
        if args.paths.len() != 2 {
            return Err(anyhow::anyhow!(
                "extra operand '{}' after '{}'",
                if args.paths.len() > 2 { &args.paths[2] } else { "" },
                &args.paths[1]
            ));
        }
        (&args.paths[0..1], Some(Path::new(&args.paths[1])))
    } else if args.paths.len() == 1 {
        (&args.paths[0..1], None)
    } else if args.paths.len() == 2 {
        let last = Path::new(&args.paths[1]);
        if last.is_dir() {
            (&args.paths[0..1], Some(last))
        } else {
            (&args.paths[0..1], Some(last))
        }
    } else {
        let (targets, dest_slice) = args.paths.split_at(args.paths.len() - 1);
        (targets, Some(Path::new(&dest_slice[0])))
    };

    let dest_is_dir = dest.map(|d| {
        if args.no_target_directory {
            false
        } else if args.no_dereference && d.is_symlink() {
            false
        } else {
            d.is_dir()
        }
    }).unwrap_or(false);

    if targets.len() > 1 && !dest_is_dir {
        return Err(anyhow::anyhow!(
            "target '{}' is not a directory",
            dest.unwrap().display()
        ));
    }

    let kind = if args.symbolic {
        LinkKind::Symbolic
    } else {
        LinkKind::Hard
    };

    for target_str in targets {
        let target_path = Path::new(target_str);
        let link_path = match dest {
            Some(d) if dest_is_dir => {
                let name = target_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("invalid target '{}'", target_str))?;
                d.join(name)
            }
            Some(d) => d.to_path_buf(),
            None => {
                let name = target_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("invalid target '{}'", target_str))?;
                PathBuf::from(name)
            }
        };

        if link_path.exists() || link_path.is_symlink() {
            let t_can = fs::canonicalize(target_path).ok();
            let l_can = fs::canonicalize(&link_path).ok();

            if let (Some(t), Some(l)) = (t_can, l_can) {
                if t == l {
                    eprintln!(
                        "ln: '{}' and '{}' are the same file",
                        target_str,
                        link_path.display()
                    );
                    exit_code = 1;
                    continue;
                }
            } else if target_path == link_path {
                eprintln!(
                    "ln: '{}' and '{}' are the same file",
                    target_str,
                    link_path.display()
                );
                exit_code = 1;
                continue;
            }

            if args.force {
                let ltype = link_type(&link_path);
                match ltype {
                    Some(LinkType::Junction) | Some(LinkType::SymlinkDir) => {
                        fs::remove_dir(&link_path)?;
                    }
                    Some(LinkType::SymlinkFile) => {
                        fs::remove_file(&link_path)?;
                    }
                    None if link_path.is_dir() => {
                        eprintln!(
                            "ln: failed to create {} link '{}': Is a directory",
                            if args.symbolic { "symbolic" } else { "hard" },
                            link_path.display()
                        );
                        exit_code = 1;
                        continue;
                    }
                    _ => {
                        // symlink_metadata might fail but exists() was true? 
                        // could be a broken symlink. remove_file should work.
                        if let Err(e) = fs::remove_file(&link_path) {
                            // if remove_file fails with Access Denied, try remove_dir (could be a junction)
                            if e.raw_os_error() == Some(5) {
                                fs::remove_dir(&link_path)?;
                            } else {
                                return Err(e.into());
                            }
                        }
                    }
                }
            } else {
                eprintln!(
                    "ln: failed to create {} link '{}': File exists",
                    if args.symbolic { "symbolic" } else { "hard" },
                    link_path.display()
                );
                exit_code = 1;
                continue;
            }
        }

        match create_link(target_path, &link_path, kind) {
            Ok(outcome) => {
                if outcome == LinkOutcome::Junction {
                    eprintln!(
                        "ln: warning: created junction instead of symlink for '{}'",
                        link_path.display()
                    );
                }
                if args.verbose {
                    println!(
                        "'{}' -> '{}'",
                        link_path.display(),
                        target_path.display()
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "ln: failed to create {} link '{}': {}",
                    if args.symbolic { "symbolic" } else { "hard" },
                    link_path.display(),
                    e
                );
                exit_code = 1;
            }
        }
    }

    Ok(exit_code)
}
