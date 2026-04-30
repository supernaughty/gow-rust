//! `uu_uname`: GNU uname — Windows port. Prints system information via Win32 APIs (U2-02).
//!
//! Uses RtlGetVersion (NOT GetVersionExW) to get the real Windows version.
//! GetVersionExW lies on Windows 8.1+ without an app-compat manifest (returns 6.2 instead of 10.0).
//! RtlGetVersion always returns the true kernel version.
//!
//! T-11-06-02: uname reveals Windows build number; standard uname behavior, not a security issue.
//! T-11-06-03: GetVersionExW compat shim avoided — RtlGetVersion used exclusively.

use std::ffi::OsString;
use std::io::{self, Write};

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser};
use windows_sys::Win32::System::SystemInformation::{
    GetNativeSystemInfo, OSVERSIONINFOW, PROCESSOR_ARCHITECTURE_AMD64,
    PROCESSOR_ARCHITECTURE_ARM64, PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
};
use windows_sys::Win32::System::WindowsProgramming::GetComputerNameW;
use windows_sys::Wdk::System::SystemServices::RtlGetVersion;

#[derive(Parser, Debug)]
#[command(
    name = "uname",
    about = "GNU uname — Windows port. Prints system information.",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,
    #[arg(long, action = ArgAction::Version)]
    version: Option<bool>,
    /// Print all information (same as -snrvma)
    #[arg(short = 'a', long = "all", action = ArgAction::SetTrue)]
    all: bool,
    /// Print the kernel name (always "Windows_NT")
    #[arg(short = 's', long = "kernel-name", action = ArgAction::SetTrue)]
    kernel_name: bool,
    /// Print the network node hostname
    #[arg(short = 'n', long = "nodename", action = ArgAction::SetTrue)]
    nodename: bool,
    /// Print the kernel release (MAJOR.MINOR.BUILD)
    #[arg(short = 'r', long = "kernel-release", action = ArgAction::SetTrue)]
    release: bool,
    /// Print the kernel version (always "#1")
    #[arg(short = 'v', long = "kernel-version", action = ArgAction::SetTrue)]
    version_flag: bool,
    /// Print the machine hardware name (x86_64/i686/aarch64)
    #[arg(short = 'm', long = "machine", action = ArgAction::SetTrue)]
    machine: bool,
    /// Print the processor type (same as -m on Windows)
    #[arg(short = 'p', long = "processor", action = ArgAction::SetTrue)]
    processor: bool,
    /// Print the hardware platform (same as -m on Windows)
    #[arg(short = 'i', long = "hardware-platform", action = ArgAction::SetTrue)]
    platform: bool,
}

/// Get the real Windows OS version using RtlGetVersion.
///
/// CRITICAL: Uses RtlGetVersion NOT GetVersionExW.
/// GetVersionExW returns 6.2 (Windows 8) on Windows 8.1/10/11 without an app-compat manifest.
/// RtlGetVersion always returns the true kernel version (e.g. 10.0.22631 for Windows 11 23H2).
/// Returns (major, minor, build).
fn get_os_version() -> (u32, u32, u32) {
    let mut info: OSVERSIONINFOW = unsafe { core::mem::zeroed() };
    info.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOW>() as u32;
    // SAFETY: info is properly initialized with dwOSVersionInfoSize set to the struct size.
    // RtlGetVersion always returns STATUS_SUCCESS on NT kernels and fills the struct.
    // Cast to *mut _ satisfies the RTL_OSVERSIONINFOW pointer type.
    unsafe { RtlGetVersion(&mut info as *mut _ as *mut _) };
    (info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber)
}

/// Get native machine architecture using GetNativeSystemInfo.
///
/// Unlike GetSystemInfo, GetNativeSystemInfo reports the NATIVE architecture
/// even when running under WOW64 (32-bit process on 64-bit OS).
fn get_machine_arch() -> &'static str {
    let mut si: SYSTEM_INFO = unsafe { core::mem::zeroed() };
    // SAFETY: si is a valid zeroed SYSTEM_INFO output buffer.
    // GetNativeSystemInfo always succeeds and fills the struct.
    unsafe { GetNativeSystemInfo(&mut si) };
    // SAFETY: Anonymous union access — wProcessorArchitecture is always a valid field in SYSTEM_INFO.
    let arch = unsafe { si.Anonymous.Anonymous.wProcessorArchitecture };
    match arch {
        PROCESSOR_ARCHITECTURE_AMD64 => "x86_64",
        PROCESSOR_ARCHITECTURE_INTEL => "i686",
        PROCESSOR_ARCHITECTURE_ARM64 => "aarch64",
        _ => "unknown",
    }
}

/// Get the computer's NetBIOS name via GetComputerNameW.
///
/// Buffer is 256 characters — well above MAX_COMPUTERNAME_LENGTH (15 for NetBIOS, up to 255 for DNS).
/// Returns "unknown" on API failure (should not occur in practice).
fn get_computer_name() -> String {
    let mut buf = [0u16; 256];
    let mut size = buf.len() as u32;
    // SAFETY: buf is a valid output buffer; size is the buffer capacity.
    // On success, size is updated to characters written NOT including the null terminator.
    let ok = unsafe { GetComputerNameW(buf.as_mut_ptr(), &mut size) };
    if ok == 0 {
        return "unknown".to_string();
    }
    String::from_utf16_lossy(&buf[..size as usize])
}

fn run(cli: &Cli) -> i32 {
    // Resolve effective flags
    let print_all = cli.all;
    let s = cli.kernel_name || print_all;
    let n = cli.nodename || print_all;
    let r = cli.release || print_all;
    let v = cli.version_flag || print_all;
    let m = cli.machine || cli.processor || cli.platform || print_all;

    // If no flag is selected, default to -s (kernel name only — GNU uname default)
    let any_flag = s || n || r || v || m;

    // Collect system information
    let sysname = "Windows_NT".to_string();
    let (major, minor, build) = get_os_version();
    let release_str = format!("{}.{}.{}", major, minor, build);
    let nodename = get_computer_name();
    let arch = get_machine_arch().to_string();
    let version_str = "#1".to_string();

    // Build output parts in GNU uname order: sysname nodename release version machine
    let mut parts: Vec<String> = Vec::new();

    if s || !any_flag {
        parts.push(sysname);
    }
    if n {
        parts.push(nodename);
    }
    if r {
        parts.push(release_str);
    }
    if v {
        parts.push(version_str);
    }
    if m {
        parts.push(arch.clone());
        if print_all {
            // GNU uname -a prints machine field twice (machine + processor positions)
            // On Windows, processor type = machine type, so we repeat arch.
            parts.push(arch);
        }
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    if let Err(e) = writeln!(out, "{}", parts.join(" ")) {
        eprintln!("uname: write error: {e}");
        return 1;
    }
    0
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();
    let args_vec: Vec<OsString> = args.into_iter().collect();
    let matches = gow_core::args::parse_gnu(Cli::command(), args_vec);
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("uname: {e}");
            return 2;
        }
    };
    run(&cli)
}
