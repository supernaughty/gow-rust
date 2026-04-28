//! `uu_curl`: HTTP/HTTPS client (curl replacement) — Windows port (R020).
//!
//! Uses reqwest blocking client with native-tls (Windows SChannel) for TLS 1.2/1.3.
//! No tokio dependency — reqwest::blocking spawns an internal runtime automatically.
//!
//! Flags: -o (output file), -x (proxy), -I (HEAD), -k (insecure), -s (silent),
//!        -L (follow redirects, default), -f (fail on HTTP 4xx/5xx).

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};

use anyhow::Result;
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use reqwest::blocking::ClientBuilder;
use reqwest::Proxy;

/// curl — HTTP/HTTPS client (Windows port).
#[derive(Parser, Debug)]
#[command(
    name = "curl",
    about = "HTTP/HTTPS client — Windows port.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help, help = "Print help")]
    help: Option<bool>,

    #[arg(long, action = ArgAction::Version, help = "Print version")]
    version: Option<bool>,

    /// URL to fetch (required positional argument).
    #[arg(required = true)]
    url: String,

    /// Write output to <file> instead of stdout.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<String>,

    /// Route requests through proxy URL (proxies all protocols — matches curl -x behavior).
    /// Example: http://user:pass@proxy:8080
    #[arg(short = 'x', long = "proxy", value_name = "URL")]
    proxy: Option<String>,

    /// Send a HEAD request and print response headers to stdout.
    #[arg(short = 'I', long = "head", action = ArgAction::SetTrue)]
    head: bool,

    /// Skip TLS certificate validation (DANGER: disables security checks — use with caution).
    #[arg(short = 'k', long = "insecure", action = ArgAction::SetTrue)]
    insecure: bool,

    /// Suppress progress/status messages (errors still appear on stderr).
    #[arg(short = 's', long = "silent", action = ArgAction::SetTrue)]
    silent: bool,

    /// Follow HTTP redirects (reqwest follows redirects by default; this flag is accepted
    /// for compatibility but has no additional effect).
    #[arg(short = 'L', long = "location", action = ArgAction::SetTrue)]
    location: bool,

    /// Fail with exit code 1 on HTTP 4xx/5xx responses (GNU curl --fail behavior).
    #[arg(short = 'f', long = "fail", action = ArgAction::SetTrue)]
    fail: bool,
}

/// Perform the HTTP/HTTPS request and write the response body to stdout or a file.
fn run(cli: Cli) -> Result<i32> {
    let mut client_builder = ClientBuilder::new();

    // Apply proxy if -x / --proxy was given.
    // Use Proxy::all() to proxy ALL protocols (HTTP + HTTPS) — matches curl -x behavior.
    // Do NOT use Proxy::http() alone: it would silently bypass the proxy for HTTPS (Pitfall 7).
    if let Some(ref proxy_url) = cli.proxy {
        client_builder = client_builder.proxy(Proxy::all(proxy_url)?);
    }

    // Apply TLS certificate skip ONLY when -k / --insecure is explicitly passed.
    // By default, Windows SChannel validates certificates against the OS trust store.
    if cli.insecure {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }

    let client = client_builder.build()?;

    // HEAD request: send HEAD, print response headers, return 0.
    if cli.head {
        let response = client.head(&cli.url).send()?;
        let status = response.status();
        // Print status line in HTTP/1.1 format.
        if !cli.silent {
            println!("HTTP/1.1 {}", status);
        }
        for (name, value) in response.headers() {
            println!("{}: {}", name, value.to_str().unwrap_or("<binary>"));
        }
        return Ok(0);
    }

    // GET request: send GET, write body to file or stdout.
    let mut response = client.get(&cli.url).send()?;
    let status = response.status();

    if let Some(ref output_path) = cli.output {
        // Write response body to file.
        let mut file = File::create(output_path)?;
        io::copy(&mut response, &mut file)?;
    } else {
        // Write response body bytes directly to stdout.
        // Use bytes() to get the full body then write all at once to preserve binary safety.
        let bytes = response.bytes()?;
        io::stdout().write_all(&bytes)?;
    }

    // --fail: exit 1 on HTTP 4xx/5xx (matches GNU curl --fail behavior).
    // Default curl behavior (without --fail): exit 0 even on HTTP errors.
    if cli.fail && !status.is_success() {
        if !cli.silent {
            eprintln!("curl: The requested URL returned error: {status}");
        }
        return Ok(1);
    }

    Ok(0)
}

/// Entry point called from src/main.rs.
///
/// CRITICAL: This is a plain synchronous function. Do NOT annotate main() with
/// `#[tokio::main]` — reqwest::blocking::Client spawns an internal tokio runtime
/// automatically. Combining `#[tokio::main]` with blocking::Client causes a
/// "Cannot drop a runtime in a context where blocking is not allowed" panic.
pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let matches = gow_core::args::parse_gnu(Cli::command(), args);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("curl: {e}");
            return 2;
        }
    };

    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("curl: {e}");
            1
        }
    }
}
