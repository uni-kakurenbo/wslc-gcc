# wslc-gcc

Windows から WSLC 上の Linux GCC を呼ぶ、GCC 互換のコンパイラコマンドです。`-E`、`-S`、`-c`、リンク、問い合わせを GCC に渡し、入出力・終了コード・生成ファイルを返します。コンパイラを指定できるツールから通常の GCC と同じように利用できます。

## 必要な環境とインストール

x64 Windows、WSL Containers（WSL 2.9.3 以上）、ビルド用の公式 Rust 1.99.0 が必要です。WSLC の基盤には `wslc-runtime` を利用します。初回は検証済み SDK と GCC イメージを取得します。

```sh
cargo install --path . --locked
wslc-gcc -dumpmachine
wslc-gcc --version
```

このコマンドの全引数は GCC の引数です。独自の `--compiler-arg`、言語標準の追加、バージョンの強制、生成物の自動実行はありません。`--help` と `--version` も GCC に渡します。

## コンパイルする

プロジェクトのディレクトリで、入力・出力とも相対パスを使います。`-o` の親ディレクトリは呼び出し側で用意します。

```sh
wslc-gcc -std=c11 -O2 -Isrc -S src/runtime.c -o build/runtime.s
wslc-gcc -std=c11 -c src/main.c -o build/main.o
wslc-gcc build/main.o -lm -o build/program
wslc-gcc -E -P -x c -
```

結果のオブジェクトと実行ファイルは Linux 用です。Windows 上で直接実行しません。

作業ディレクトリを `/work` に読み書き可能でマウントします。`WSLC_WORKSPACE` で共有するルートを指定すると、その配下のサブディレクトリからも対応する作業ディレクトリで実行できます。相対パスは GCC へそのまま渡すため、パスの区切りには `/` を使います。

Windows の絶対パスや UNC パスを GCC の引数へ自動変換する機能はありません。イメージ内の `/usr/include` などの絶対パスは Linux 側の意味のままです。`-I`、`-o`、`-MF`、応答ファイル内のパスもこの契約に従います。診断と依存ファイルも書き換えません。

標準入力・標準出力・標準エラーを TTY なしで中継し、UTF-8 のデコードや改行の変換は行いません。GCC の成功・失敗の終了コードを返します。基盤の失敗は診断を標準エラーへ出し、終了コード1を返します。

## 環境設定と並列ビルド

| 環境変数 | 意味 | 既定値 |
| --- | --- | --- |
| `WSLC_GCC_IMAGE` | GCC を含むイメージ。digest の指定も可能 | docker.io/library/gcc:16.2.0 |
| `WSLC_GCC_DRIVER` | イメージ内のドライバー | gcc |
| `WSLC_GCC_TIMEOUT` | 一回の呼び出しの制限秒数 | 1800 |
| `WSLC_WORKSPACE` | ホストで共有するルート | 作業ディレクトリ |

SDK・保存先・既存セッションは `wslc-runtime` の `WSLC_CACHE`、`WSLC_STORAGE`、`WSLC_SESSION` などで設定します。C++ のリンクには `WSLC_GCC_DRIVER=g++` を設定できます。

単独起動は自分のセッションを所有し、終了時に解放します。同じ保存先を使う単独呼び出しは直列化されます。並列ビルドでは `wslc-runtime` の `wslc-session` などが一つのセッションを保持し、各コンパイラ呼び出しが接続して利用します。

```sh
wslc-session -- your-build-tool
```

接続する側は SDK の取得やセッションの終了を行いません。親が子の完了までセッションを保持します。永続的なサービスや既定セッションは必要ありません。

## コンパイルして実行する

別コマンドの `wslc-gcc-run` は言語・規格の指定、一時出力、実行引数、TTY などを扱います。同じリポジトリの GCC 実行 API と `wslc-runtime` を使い、GCC 互換コマンドの引数とは分けています。

```sh
wslc-gcc-run examples/hello.c
wslc-gcc-run examples/hello.cpp --standard c++26 --compiler-arg -O2 -- "hello world"
wslc-gcc-run main.s
wslc-gcc-run --help
```

実行用のプロジェクトは `/src` に読み取り専用でマウントし、コンパイル結果はコンテナの一時領域に置きます。`--project-dir`、`--compiler-arg`、`--run-arg`、`--image`、`--interactive`、`--tty` を使えます。Rust からは `compile::Request` を利用できます。Linux の既存環境向けには `native` の実行補助を提供します。

## 検証

```sh
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked -- --ignored --test-threads 1
```

最後のコマンドは実際の WSLC を使います。同じイメージの GCC に直接渡した場合との出力・診断・終了コードの一致、標準入力、不正 UTF-8、空白と日本語を含むパス、応答ファイル、依存ファイル、アセンブリと ELF の生成、既存セッション内の並列コンパイルを検証します。合成した C/C++/アセンブリだけを使い、特定の利用プロジェクトには依存しません。

[MIT License](LICENSE)。実行基盤は独立した `wslc-runtime` リポジトリにあります。

## Install from Git

```sh
cargo install --git https://github.com/uni-kakurenbo/wslc-gcc --locked
```
