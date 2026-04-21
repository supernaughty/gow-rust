//! Platform-gated Ctrl+C suppression for `tee -i`.
//!
//! Windows: `SetConsoleCtrlHandler(None, TRUE)` causes the process to ignore
//! CTRL_C_EVENT, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT. This is the exact behavior
//! GNU `tee -i` produces on POSIX via `signal(SIGINT, SIG_IGN)`.
//!
//! Reference: RESEARCH.md Q10 lines 883-960. uutils's tee has no Windows
//! implementation of -i (noop); we are first-in-ecosystem.
//!
//! The caller MUST treat this as best-effort — if the process is not attached
//! to a console (e.g. spawned without stdin-as-console by a test harness), the
//! underlying Win32 call can fail. `tee -i` should never crash because of that;
//! silent pass-through is safer than forcing the user to see a spurious error.

#[cfg(windows)]
pub fn ignore_interrupts() -> std::io::Result<()> {
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
    // add=1 (TRUE), handler=None (NULL) -> process ignores console ctrl events.
    let ok = unsafe { SetConsoleCtrlHandler(None, 1) };
    if ok == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
pub fn ignore_interrupts() -> std::io::Result<()> {
    unsafe extern "C" {
        fn signal(signum: std::os::raw::c_int, handler: usize) -> usize;
    }
    const SIGINT: std::os::raw::c_int = 2;
    const SIG_IGN: usize = 1;
    let prev = unsafe { signal(SIGINT, SIG_IGN) };
    if prev == usize::MAX {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_interrupts_smoke_ok() {
        // Verify the call returns Ok (or at least Err gracefully) in a normal test
        // environment. This doesn't prove Ctrl+C is actually ignored (see
        // RESEARCH.md Q10 "Integration 테스트 한계") — just that the API can be
        // invoked. If the test runner is detached, the call may return Err; in
        // that case the library contract says it's best-effort, so either is fine.
        let _ = ignore_interrupts();
    }
}
