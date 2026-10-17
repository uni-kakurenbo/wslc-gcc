mod cli;

use cli::{Action, Options};
use std::process::ExitCode;
use wslc_gcc::compile::Prepared;
use wslc_runtime::process::report;

#[cfg(windows)]
fn invoke(options: &Options, prepared: Option<&Prepared>) -> wslc_runtime::Result<i32> {
    use std::time::Duration;
    use wslc_runtime::container::{SessionOptions, executable, with_connection};
    use wslc_runtime::process::Runner;

    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut session = SessionOptions::from_env()?;
    session.cache.clone_from(&options.cache);
    if std::env::var_os("WSLC_STORAGE").is_none() {
        session.storage = options.cache.join("session-storage");
    }
    if let Some(executable) = &options.executable {
        session.executable = Some(executable.clone());
    }

    if options.action == Action::Doctor {
        let executable = executable(session.executable.as_deref())?;
        eprintln!("WSLC: {}", executable.display());
        eprintln!("SDK: {} (verified archive)", session.sdk.version);
        eprintln!("Cache: {}", session.cache.display());
        return Runner { root: &root }.stream(
            &executable,
            &["version".into()],
            Duration::from_secs(120),
        );
    }

    let prepared = prepared.ok_or("Compile request was not prepared")?;
    with_connection(&root, &session, |context| {
        Ok(prepared.run(context, options.timeout, false)?.code)
    })
}

#[cfg(not(windows))]
fn invoke(_options: &Options, _prepared: Option<&Prepared>) -> wslc_runtime::Result<i32> {
    Err("WSL Containers execution requires Windows".into())
}

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let options = match cli::parse(&arguments) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };

    match options.action {
        Action::Help => {
            print!("{}", cli::HELP);
            return ExitCode::SUCCESS;
        }
        Action::Version => {
            println!("wslc-gcc-run {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        _ => {}
    }

    let prepared = if options.action == Action::Run {
        match options.request.prepare() {
            Ok(request) => Some(request),
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::from(2);
            }
        }
    } else {
        None
    };

    report(invoke(&options, prepared.as_ref()))
}
