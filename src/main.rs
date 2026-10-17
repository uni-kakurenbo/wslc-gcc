//! GCC-compatible entry point: every argument belongs to the compiler.
use std::process::ExitCode;

fn main() -> ExitCode {
    #[cfg(windows)]
    let result = wslc_gcc::driver::run(&std::env::args_os().skip(1).collect::<Vec<_>>());
    #[cfg(not(windows))]
    let result = Err("wslc-gcc requires Windows; use a native or cross GCC on this host".into());
    wslc_runtime::process::report(result)
}
