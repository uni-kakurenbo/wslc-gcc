# wslc-gcc

Use Linux GCC from Windows through WSL Containers (WSLC). `wslc-gcc` supports preprocessing, assembly output, object compilation, linking, and compiler queries. It forwards GCC arguments, standard streams, exit status, and generated files to the calling build tool.

Use `wslc-gcc-run` to compile source files and run the resulting Linux program in a container.

## Install

Requires x64 Windows, WSL 2.9.3 or later, and Rust 1.99.0 for the build. The first invocation downloads and verifies the WSLC SDK and downloads the GCC image.

```sh
cargo install --git https://github.com/uni-kakurenbo/wslc-gcc.git --locked
wslc-gcc -dumpmachine
wslc-gcc --version
```

To install from a checkout, run `cargo install --path . --locked`.

Every `wslc-gcc` argument goes to GCC, including `--help` and `--version`. Select language standards and compiler options with ordinary GCC flags.

## Compile

Run from the project directory and create the output directory before invoking GCC:

```sh
wslc-gcc -std=c11 -O2 -Iinclude -S src/main.c -o build/main.s
wslc-gcc -std=c11 -Iinclude -c src/main.c -o build/main.o
wslc-gcc build/main.o -lm -o build/program
wslc-gcc -E -P -x c -
```

The last command reads C source from standard input. Generated objects and executables target Linux; run executables in a Linux environment or container.

The workspace is mounted read-write at `/work`. It defaults to the current directory. Set `WSLC_WORKSPACE` to a project root to invoke the compiler from subdirectories; the corresponding relative working directory is preserved inside the container.

Use project-relative paths with forward slashes for sources, outputs, include directories, dependency files, and response files. Absolute Linux paths such as `/usr/include` refer to the container. Windows absolute paths and UNC paths are unsupported as compiler arguments. Diagnostics and dependency files retain the paths reported by GCC.

Standard streams are forwarded as bytes with no terminal allocation, UTF-8 decoding, or newline conversion. The command returns GCC's exit status. Session or container failures print a diagnostic to stderr and return exit code 1.

## Configure the compiler

| Environment variable | Purpose | Default |
| --- | --- | --- |
| `WSLC_GCC_IMAGE` | Image containing GCC; a digest may be used | docker.io/library/gcc:16.2.0 |
| `WSLC_GCC_DRIVER` | Compiler executable inside the image | gcc |
| `WSLC_GCC_TIMEOUT` | Timeout per invocation in seconds | 1800 |
| `WSLC_WORKSPACE` | Host directory mounted at `/work` | Current directory |

Set `WSLC_GCC_DRIVER=g++` for C++ compilation and linking. SDK cache, session storage, and connection settings use the `WSLC_CACHE`, `WSLC_STORAGE`, and `WSLC_SESSION` variables documented by [wslc-runtime](https://github.com/uni-kakurenbo/wslc-runtime#configuration-and-parallel-execution).

## Parallel builds

A standalone invocation creates a session and releases it on exit. Invocations sharing the same session storage wait for each other.

For parallel builds, install `wslc-session` and run the build tool inside one shared session:

```sh
cargo install --git https://github.com/uni-kakurenbo/wslc-runtime.git --locked wslc-runtime
wslc-session -- your-build-tool
```

The launcher passes connection variables to its children. Each `wslc-gcc` invocation connects to that session, allowing multiple compiler processes to run concurrently. The launcher keeps the session alive until the build command finishes and then releases it.

## Compile and run

`wslc-gcc-run` compiles C, C++, or assembly source and runs the result in a container:

```sh
wslc-gcc-run examples/hello.c
wslc-gcc-run examples/hello.cpp --standard c++26 --compiler-arg -O2 -- "hello world"
wslc-gcc-run main.s
wslc-gcc-run --help
```

The source directory is mounted read-only at `/src`, and compiler outputs go to a temporary container directory. The source extension selects the language; default standards are C23 and C++23. Use `--project-dir`, `--language`, `--standard`, `--compiler-arg`, and `--run-arg` to configure compilation and program arguments. `--image`, `--interactive`, and `--tty` configure the container and terminal.

Rust callers can use `driver::Driver` to invoke GCC in a WSLC session or `compile::Request::prepare` followed by `compile::Prepared::run` to compile and run a program. The `native` module runs host commands and assembles source with the host's GCC.

## Verify

```sh
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked -- --ignored --test-threads 1
```

The last command uses a real WSLC session. It compares output, diagnostics, and exit status against GCC invoked directly in the same image. Tests also cover standard input, invalid UTF-8, paths containing spaces and Unicode, response files, dependency files, assembly and ELF generation, and parallel compiler calls in a shared session.

[MIT License](LICENSE).
