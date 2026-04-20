//! gow-probe: internal integration test binary for gow-core validation.
//!
//! NOT for distribution. Used only by `cargo test -p gow-probe` integration tests.
//!
//! Subcommands:
//!   gow-probe                     — init smoke test; exits 0, prints "gow-probe: init ok"
//!   gow-probe path <arg>          — prints MSYS-converted path; exits 0
//!   gow-probe exit-code <n>       — exits with specified code (for testing)
//!   gow-probe --bad-flag          — triggers clap error; exits 1 (GNU behavior)

use clap::{Arg, ArgAction, Command};

fn main() {
    // First line of every gow-rust binary: initialize platform primitives.
    gow_core::init();

    let cmd = Command::new("gow-probe")
        .about("gow-core integration test harness (not for distribution)")
        .subcommand(
            Command::new("path")
                .about("Convert an argument path using gow-core path conversion")
                .arg(Arg::new("input").required(true).action(ArgAction::Set)),
        )
        .subcommand(
            Command::new("exit-code")
                .about("Exit with the specified exit code")
                .arg(
                    Arg::new("code")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .action(ArgAction::Set),
                ),
        );

    let matches = gow_core::args::parse_gnu(cmd, std::env::args_os());

    match matches.subcommand() {
        Some(("path", sub)) => {
            let input = sub.get_one::<String>("input").unwrap();
            let converted = gow_core::path::try_convert_msys_path(input);
            println!("{converted}");
            std::process::exit(0);
        }
        Some(("exit-code", sub)) => {
            let code = sub.get_one::<i32>("code").copied().unwrap_or(0);
            std::process::exit(code);
        }
        _ => {
            // Default: init smoke test
            println!("gow-probe: init ok");
            std::process::exit(0);
        }
    }
}
