//! `uu_tee`: GNU `tee` — split stdin to stdout + N files.
//!
//! Flags:
//!   -a / --append            append to files rather than truncate (O_APPEND semantics)
//!   -i / --ignore-interrupts Windows: SetConsoleCtrlHandler(None, TRUE); Unix: SIGINT SIG_IGN
//!
//! MSYS pre-convert applies to each file operand (D-26) so `tee /c/tmp/log.txt` writes
//! to `C:\tmp\log.txt`.
//!
//! Error handling: if a file cannot be opened, tee reports `tee: {path}: {error}` to
//! stderr and continues writing to remaining sinks. Exit code reflects open failures (1)
//! or success (0). A BrokenPipe on stdout is silent (GNU parity — exit 0).
//!
//! References: RESEARCH.md Q10 (signals module), CONTEXT.md D-25 (fanout semantics),
//! VALIDATION.md Dimensions 1/3/4.

mod signals;

use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use clap::{Arg, ArgAction, Command};

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(uu_app(), args);
    let append = matches.get_flag("append");
    let ignore_int = matches.get_flag("ignore-interrupts");

    if ignore_int {
        // Best-effort: silent failure if stdout is not console-attached. See
        // signals module doc-comment; RESEARCH.md Q10 "Integration 테스트 한계".
        // T-02-07-03 mitigation: only invoked when -i is explicitly requested.
        let _ = signals::ignore_interrupts();
    }

    let operands: Vec<String> = matches
        .get_many::<String>("operands")
        .map(|iter| iter.cloned().collect())
        .unwrap_or_default();

    // Sinks: stdout first, then each successfully-opened file. Box<dyn Write>
    // unifies the fanout vector. Per-write stdout lock is slightly slower than
    // holding StdoutLock, but avoids lifetime coupling gymnastics — acceptable
    // for an I/O-bound utility.
    let mut outputs: Vec<Box<dyn Write>> = Vec::with_capacity(operands.len() + 1);
    outputs.push(Box::new(io::stdout()));

    let mut file_errors = 0u32;
    for op in &operands {
        // D-26: MSYS path pre-convert applies to each file operand.
        let converted = gow_core::path::try_convert_msys_path(op);
        let path = Path::new(&converted);
        match open_output_file(path, append) {
            Ok(file) => outputs.push(Box::new(file)),
            Err(e) => {
                eprintln!("tee: {converted}: {e}");
                file_errors += 1;
            }
        }
    }

    // Fanout loop: read stdin in 8 KiB chunks, write_all to every surviving sink.
    // Per-chunk flush keeps `tail -f foo | tee log` real-time-ish; GNU tee uses
    // block buffering by default but flushes at natural chunk boundaries.
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut buf = [0u8; 8192];

    loop {
        match input.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                outputs.retain_mut(|sink| match sink.write_all(&buf[..n]) {
                    Ok(()) => {
                        // Flush best-effort; failure here implies the sink is dead,
                        // so drop it from the fanout on the next iteration naturally.
                        let _ = sink.flush();
                        true
                    }
                    Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                        // GNU parity: stdout closed (downstream consumer gone) is
                        // not an error — drop the sink silently, continue writing
                        // to remaining files.
                        false
                    }
                    Err(e) => {
                        eprintln!("tee: write error: {e}");
                        false
                    }
                });
                if outputs.is_empty() {
                    // No sinks left (e.g. stdout was the only one and it BrokenPipe'd).
                    // Drain stdin to completion so the producer doesn't block on our
                    // refusal — but do it lazily by just returning; the OS will close
                    // the pipe when the process exits.
                    break;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("tee: read error: {e}");
                return 1;
            }
        }
    }

    // Final flush pass — per-chunk flush already happened, but ensure file
    // buffers are durable before process exit.
    for sink in outputs.iter_mut() {
        let _ = sink.flush();
    }

    if file_errors > 0 { 1 } else { 0 }
}

fn open_output_file(path: &Path, append: bool) -> io::Result<File> {
    let mut opts = OpenOptions::new();
    opts.write(true).create(true);
    if append {
        opts.append(true);
    } else {
        opts.truncate(true);
    }
    opts.open(path)
}

fn uu_app() -> Command {
    Command::new("tee")
        .about("Copy standard input to each FILE, and also to standard output.")
        .arg(
            Arg::new("append")
                .short('a')
                .long("append")
                .action(ArgAction::SetTrue)
                .help("append to the given FILEs, do not overwrite"),
        )
        .arg(
            Arg::new("ignore-interrupts")
                .short('i')
                .long("ignore-interrupts")
                .action(ArgAction::SetTrue)
                .help("ignore interrupt signals"),
        )
        .arg(
            Arg::new("operands")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}
