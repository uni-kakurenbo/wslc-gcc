//! Run Linux GCC with the caller's arguments, standard streams, and exit status.

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use wslc_runtime::container::{Context, Mount, Pull, RunOptions, SessionOptions, with_connection};

#[cfg(windows)]
pub struct Driver {
    pub workspace: PathBuf,
    pub workdir: String,
    pub image: String,
    pub compiler: String,
}

#[cfg(windows)]
impl Driver {
    /// Read the workspace, image, and compiler from the `WSLC_*` environment variables.
    /// `cwd` must be inside the workspace mounted at `/work`.
    pub fn from_env(cwd: &Path) -> crate::Result<Self> {
        let workspace = std::env::var_os("WSLC_WORKSPACE")
            .map(PathBuf::from)
            .unwrap_or_else(|| cwd.to_owned());
        let workspace = workspace
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let cwd = cwd.canonicalize().map_err(|error| error.to_string())?;
        let relative = cwd
            .strip_prefix(&workspace)
            .map_err(|_| "Working directory must be inside WSLC_WORKSPACE")?;
        let relative = relative.to_string_lossy().replace('\\', "/");

        Ok(Self {
            workspace,
            workdir: if relative.is_empty() {
                "/work".into()
            } else {
                format!("/work/{relative}")
            },
            image: std::env::var("WSLC_GCC_IMAGE")
                .unwrap_or_else(|_| crate::compile::DEFAULT_IMAGE.into()),
            compiler: std::env::var("WSLC_GCC_DRIVER").unwrap_or_else(|_| "gcc".into()),
        })
    }

    pub fn request(&self, arguments: &[OsString]) -> RunOptions {
        RunOptions {
            image: self.image.clone(),
            mounts: vec![Mount {
                host: self.workspace.clone(),
                guest: "/work".into(),
                read_only: false,
            }],
            workdir: self.workdir.clone(),
            entrypoint: "/usr/bin/env".into(),
            arguments: std::iter::once(OsString::from(&self.compiler))
                .chain(arguments.iter().cloned())
                .collect(),
            pull: Pull::Missing,
            interactive: true,
            tty: false,
        }
    }

    /// Run GCC with inherited standard input, output, and error handles.
    pub fn run(
        &self,
        context: &Context<'_>,
        arguments: &[OsString],
        timeout: Duration,
    ) -> crate::Result<i32> {
        let mut request = self.request(arguments);
        request.interactive = wslc_runtime::process::stdin_available();
        Ok(context.run(&request, timeout, false)?.code)
    }
}

#[cfg(windows)]
pub fn run(arguments: &[OsString]) -> crate::Result<i32> {
    let root = std::env::current_dir().map_err(|error| error.to_string())?;
    let driver = Driver::from_env(&root)?;
    let seconds = std::env::var("WSLC_GCC_TIMEOUT")
        .unwrap_or_else(|_| "1800".into())
        .parse::<u64>()
        .map_err(|_| "WSLC_GCC_TIMEOUT must be a positive number of seconds")?;
    if seconds == 0 {
        return Err("WSLC_GCC_TIMEOUT must be positive".into());
    }

    with_connection(&root, &SessionOptions::from_env()?, |context| {
        driver.run(context, arguments, Duration::from_secs(seconds))
    })
}
