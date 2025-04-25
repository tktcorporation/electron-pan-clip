# macOSへのクロスコンパイル設定

このドキュメントでは、Linuxホスト環境からmacOS（x86_64-apple-darwin）へのクロスコンパイル設定について説明します。

## 問題の概要

Linuxホスト上で以下のコマンドを実行しようとした際に問題が発生しました：

```
napi build --platform --release --target x86_64-apple-darwin
```

### 発生したエラー

1. `objc`クレートのマクロに関する警告
   ```
   warning: unexpected `cfg` condition value: `cargo-clippy`
     --> src/platforms/macos.rs:43:23
      |
   43 |       let url_class = class!(NSURL);
      |                       ^^^^^^^^^^^^^
   ```

2. リンカーエラー
   ```
   error: linking with `cc` failed: exit status: 1
   ...
   cc: error: unrecognized command-line option '-framework'
   cc: error: unrecognized command-line option '-arch'; did you mean '-march='?
   ```

## 対応策

### 1. マクロ警告への対応

`src/platforms/macos.rs`ファイル内のマクロに`#[allow(unexpected_cfgs)]`アノテーションを追加して警告を抑制しました：

```rust
// NSURL を直接作成する
#[allow(unexpected_cfgs)]
let url_class = class!(NSURL);

#[allow(unexpected_cfgs)]
let nsurl: id = msg_send![url_class, fileURLWithPath:ns_string];
```

### 2. クロスコンパイル環境の整備

#### a. Cargoの設定追加

`.cargo/config.toml`にmacOS向けのクロスコンパイル設定を追加：

```toml
[target.x86_64-apple-darwin]
linker = "x86_64-apple-darwin-clang"
ar = "x86_64-apple-darwin-ar"
```

#### b. 必要なツールのインストール

```bash
# clangなどの基本的なビルドツールをインストール
sudo apt-get install -y clang curl git build-essential

# Zigのインストール（クロスコンパイルサポート用）
wget -O - https://ziglang.org/download/0.12.0/zig-linux-aarch64-0.12.0.tar.xz | tar -xJ
sudo mv zig-linux-aarch64-0.12.0 /usr/local/zig
export PATH=$PATH:/usr/local/zig

# cargo-zigbuildのインストール
cargo install cargo-zigbuild
```

## 残存する課題

以上の対応を行っても、完全なクロスコンパイルには以下の課題が残ります：

1. macOS SDKが必要
   - Xcode Command Line Toolsからの抽出が必要
   - OSXCrossなどの専用ツールのセットアップが必要

2. 動的ライブラリの問題
   ```
   error: unable to find dynamic system library 'objc' using strategy 'paths_first'
   ```

## 推奨される解決策

1. **macOS環境での直接ビルド**
   ```
   cargo build --release --target x86_64-apple-darwin
   napi build --platform --release --target x86_64-apple-darwin
   ```

2. **GitHub Actionsなどを利用したマルチプラットフォームビルド**
   - macOS, Windows, Linuxそれぞれの環境でネイティブビルドを実行
   - 各プラットフォーム向けのバイナリを生成して統合

3. **Docker + OSXCrossを利用した完全なクロスコンパイル環境の構築**
   - より高度な設定が必要
   - 詳細は[OSXCross](https://github.com/tpoechtrager/osxcross)のドキュメントを参照

## 参考リンク

- [Rustのクロスコンパイル公式ドキュメント](https://rust-lang.github.io/rustup/cross-compilation.html)
- [OSXCross](https://github.com/tpoechtrager/osxcross)
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)
- [napi-rs ドキュメント](https://napi.rs/docs/introduction/building-for-multiple-platforms)
