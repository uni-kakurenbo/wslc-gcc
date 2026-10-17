//! Compile with Linux GCC through WSLC, or compile and run source files.
//!
//! `driver` forwards GCC arguments and standard streams. `compile::Request` configures
//! compilation followed by execution. `native` runs commands with the host's tools.

pub mod compile;
pub mod native;

pub use wslc_runtime::Result;

pub mod driver;
