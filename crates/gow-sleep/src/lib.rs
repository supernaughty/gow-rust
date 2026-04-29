use std::ffi::OsString;
use std::thread;
use std::time::Duration;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};

#[derive(Parser, Debug)]
#[command(
    name = "sleep",
    about = "GNU sleep — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,

    /// Number of seconds to sleep (integer or fractional). Multiple values are summed.
    durations: Vec<String>,
}

fn run(cli: &Cli) -> i32 {
    if cli.durations.is_empty() {
        eprintln!("sleep: missing operand");
        eprintln!("Try 'sleep --help' for more information.");
        return 1;
    }

    let mut total: f64 = 0.0;
    for arg in &cli.durations {
        let secs: f64 = match arg.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("sleep: invalid time interval '{arg}'");
                return 1;
            }
        };
        if secs < 0.0 {
            eprintln!("sleep: invalid time interval '{arg}'");
            return 1;
        }
        total += secs;
    }

    thread::sleep(Duration::from_secs_f64(total));
    0
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sleep: {e}");
            return 2;
        }
    };
    run(&cli)
}
