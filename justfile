#!/usr/bin/env -S direnv exec / just --justfile

# エイリアス定義
alias r := ready
alias f := fix
alias c := check

# デフォルトはヘルプを表示
_default:
  @just --list -u

# Make sure you have cargo-binstall and pnpm installed.
# You can download the pre-compiled binary from <https://github.com/cargo-bins/cargo-binstall#installation>
# or install via `cargo install cargo-binstall`
# Initialize the project by installing all the necessary tools.
init:
  # Rust related init
  cargo binstall watchexec-cli cargo-insta typos-cli cargo-shear dprint -y
  
  # npm install -g pnpm typescript @napi-rs/cli @antfu/ni
  # Node.js related init
  yarn install
  # Linuxの依存関係をインストール
  just install-linux-deps

# 全てのチェックを実行
check: check-format check-lint

# フォーマットチェック（Rust + TS）
check-format: check-rust-format check-ts-format

# リントチェック（Rust + TS）
check-lint: check-rust-lint check-ts-lint check-types

# Rustのフォーマットチェック
check-rust-format:
  @echo "🔍 Rustのフォーマットをチェック中..."
  cargo fmt --all -- --check

# Rustのリントチェック
check-rust-lint:
  @echo "🔍 Rustのリントをチェック中..."
  cargo clippy -- -D warnings

check-all-platforms:
  @echo "🔍 全てのプラットフォームでチェック中..."
  rustup target add aarch64-apple-darwin
  export CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER="$(xcrun --find clang)" && \
    cargo build --release --target aarch64-apple-darwin
  cargo build --release --target x86_64-pc-windows-msvc
  cargo build --release --target x86_64-unknown-linux-gnu
  cargo check --target x86_64-apple-darwin
  cargo check --target x86_64-pc-windows-msvc
  cargo check --target x86_64-unknown-linux-gnu

# TypeScriptのフォーマットチェック
check-ts-format:
  @echo "🔍 TypeScriptのフォーマットをチェック中..."
  yarn check:format:ts

# TypeScriptのリントチェック
check-ts-lint:
  @echo "🔍 TypeScriptのリントをチェック中..."
  yarn check:lint:ts

# 型チェック
check-types:
  @echo "🔍 TypeScript型をチェック中..."
  yarn check:types

# 高速チェック (CI用)
ci-check:
  @echo "🚀 CI用の高速チェックを実行中..."
  cargo check
  cargo clippy --all-targets -- -D warnings
  yarn check:types
  yarn check:lint:ts

# oxlintを使用する場合（高速なJSリント）
check-oxlint:
  @echo "🔍 oxlintでJavaScriptファイルをチェック中..."
  yarn check:oxlint

# 全てのテストを実行
test:
  @echo "🧪 テストを実行中..."
  just build
  just test-for-each-os

# OSごとに適したテスト実行
[windows]
test-for-each-os:
  @echo "🧪 Windowsでテストを実行中..."
  cargo test
  yarn test

[macos]
test-for-each-os:
  @echo "🧪 macOSでテストを実行中..."
  cargo test
  yarn test

[linux]
test-for-each-os:
  @echo "🧪 Xvfbを使用してテストを実行中..."
  ./scripts/run-with-xvfb.sh cargo test
  ./scripts/run-with-xvfb.sh yarn test

# コードをフォーマット
fmt:
  @echo "✨ コードをフォーマット中..."
  cargo fmt --all
  yarn fmt:ts

# リント問題を自動修正
fix:
  @echo "🔧 自動修正可能な問題を修正中..."
  cargo clippy --fix --allow-staged
  yarn lint:fix
  just fmt

# watchモードでの開発
watch *args='':
  watchexec --no-vcs-ignore {{args}}

# watch-checkコマンド
watch-check:
  just watch "just check"

# watch-testコマンド
watch-test:
  just watch "just test"

# gitプッシュ前のチェック
pre-push: ready
  @echo "✅ 全てのチェックが通過しました！"

# 全てのチェックを実行して準備完了
ready:
  @echo "🚀 全ての準備チェックを実行中..."
  just check
  just test
  git status

# Git pre-pushフックをインストール
install-hook:
  @echo "🔄 Git pre-pushフックをインストール中..."
  echo -e "#!/bin/sh\njust pre-push" > .git/hooks/pre-push
  chmod +x .git/hooks/pre-push
  @echo "✅ Git pre-pushフックがインストールされました"

# プロジェクトビルド
build:
  @echo "🏗️ プロジェクトをビルド中..."
  cargo build --release
  yarn build

# ドキュメント生成
[linux]
doc:
  RUSTDOCFLAGS='-D warnings' cargo doc --no-deps

[windows]
doc:
  $Env:RUSTDOCFLAGS='-D warnings'; cargo doc --no-deps

# Linuxの依存関係をインストール
[linux]
install-linux-deps:
  @echo "📦 Linux依存関係をインストール中..."
  sudo apt-get update
  sudo apt-get install -y \
    libx11-dev \
    libxext-dev \
    libxrender-dev \
    libxtst-dev \
    libxinerama-dev \
    xvfb \
    x11-apps \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    xclip \
    x11-apps \
    xvfb \
    xauth

# Linuxの依存関係をインストール（何もしない、Windowsの場合）
[windows]
install-linux-deps:
  @echo "📦 Windows環境では不要なため、何もしません"

[macos]
install-linux-deps:
  @echo "📦 macOS環境では不要なため、何もしません"

# Windowsクロスコンパイル環境のセットアップ
[linux]
setup-windows-cross:
  @echo "🪟 Windowsクロスコンパイル環境をセットアップ中..."
  cargo install cargo-xwin
  rustup target add x86_64-pc-windows-msvc
  @echo "✅ Windowsクロスコンパイル環境のセットアップが完了しました"

# Windows向けのクロスコンパイルビルド
[linux]
build-windows:
  @echo "🏗️ Windows向けにクロスコンパイル中..."
  cargo xwin build --release --target x86_64-pc-windows-msvc
  # または napi-rs を使用する場合
  # napi build --platform --release --target x86_64-pc-windows-msvc
  @echo "✅ Windows向けビルドが完了しました" 