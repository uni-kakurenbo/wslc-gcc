#![cfg(windows)]

use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::Duration;
use wslc_gcc::driver::Driver;
use wslc_runtime::container::{SessionOptions, with_session};

fn collect(mut command: Command, input: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
#[ignore = "requires Windows WSLC and the GCC image"]
fn gcc_contract_includes_streams_files_queries_and_parallel_borrowers() {
    let root = std::env::temp_dir().join(format!("wslc-driver-{} 日本語", std::process::id()));
    fs::create_dir_all(root.join("build")).unwrap();
    fs::write(
        root.join("input space.c"),
        "int main(void) { return 37; }\n",
    )
    .unwrap();
    fs::write(root.join("bad.c"), "#error expected_failure\n").unwrap();
    fs::write(root.join("flags.rsp"), "-std=c11 -Wall -Werror\n").unwrap();
    let cache = std::env::var_os("WSLC_GCC_TEST_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("wslc-gcc-test-cache"));
    let options = SessionOptions::new(cache);

    with_session(&root, &options, |context| {
        let driver = Driver::from_env(&root)?;
        // Populate the image before comparing the compiler streams.
        let mut ready = driver.request(&["-dumpmachine".into()]);
        ready.interactive = false;
        assert_eq!(context.run(&ready, Duration::from_secs(300), true)?.code, 0);

        let wrapper = |arguments: &[OsString]| {
            let mut command = Command::new(env!("CARGO_BIN_EXE_wslc-gcc"));
            command.current_dir(&root).args(arguments);
            context.configure_child(&mut command);
            command
        };
        for (arguments, input) in [
            (vec!["-dumpmachine"], Vec::new()),
            (vec!["--version"], Vec::new()),
            (vec!["-E", "-P", "input space.c"], Vec::new()),
            (vec!["-fsyntax-only", "bad.c"], Vec::new()),
            (
                vec!["-E", "-P", "-x", "assembler-with-cpp", "-"],
                b".ascii \"\xff\"\n".to_vec(),
            ),
        ] {
            let arguments: Vec<OsString> = arguments.into_iter().map(Into::into).collect();
            let direct = collect(context.command(&driver.request(&arguments))?, &input);
            let wrapped = collect(wrapper(&arguments), &input);
            assert_eq!(direct.status.code(), wrapped.status.code(), "{arguments:?}");
            assert_eq!(direct.stdout, wrapped.stdout, "stdout: {arguments:?}");
            assert_eq!(direct.stderr, wrapped.stderr, "stderr: {arguments:?}");
        }

        for (flags, artifact) in [
            (vec!["-S", "-o", "build/runtime.s"], "build/runtime.s"),
            (
                vec!["-MMD", "-MF", "build/main.d", "-o", "build/program"],
                "build/program",
            ),
        ] {
            let arguments: Vec<OsString> = ["@flags.rsp", "input space.c"]
                .into_iter()
                .chain(flags)
                .map(Into::into)
                .collect();
            let result = collect(wrapper(&arguments), &[]);
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(result.stdout.is_empty());
            assert!(result.stderr.is_empty());
            assert!(root.join(artifact).is_file());
        }
        assert!(
            fs::read(root.join("build/program"))
                .unwrap()
                .starts_with(b"\x7fELF")
        );
        assert!(root.join("build/main.d").is_file());

        let mut children = Vec::new();
        for name in ["left", "right"] {
            let arguments = vec![
                "-c".into(),
                "input space.c".into(),
                "-o".into(),
                format!("build/{name}.o").into(),
            ];
            children.push(
                wrapper(&arguments)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(),
            );
        }
        for child in children {
            let result = child.wait_with_output().unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(result.stdout.is_empty());
            assert!(result.stderr.is_empty());
        }
        assert!(root.join("build/left.o").is_file());
        assert!(root.join("build/right.o").is_file());
        Ok(())
    })
    .unwrap();

    // This test owns only this unique directory under the OS temporary directory.
    assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
    fs::remove_dir_all(root).unwrap();
}
