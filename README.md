# electron-pan-clip

Electronアプリケーションで複数画像を同時にコピー＆ペーストするためのネイティブライブラリです。

## 概要

このライブラリは、Electronアプリケーションから複数のファイル（特に画像）を一度にクリップボードにコピーし、Discordなどの他のアプリケーションに貼り付けることを可能にします。Rustで実装されたネイティブモジュールとして、Windows、macOS、Linuxの各プラットフォームでネイティブのクリップボード操作を提供します。

## 使い方

```javascript
import { copyFiles } from 'electron-pan-clip';

// 複数のファイルパスをクリップボードにコピー
copyFiles([
  '/path/to/image1.png',
  '/path/to/image2.jpg'
]);
```

## 開発環境

このプロジェクトは開発環境としてDevContainerを使用しています。Visual Studio CodeとDockerをインストールしている場合、以下の手順で開発環境を立ち上げることができます。

1. このリポジトリをクローンします
2. Visual Studio Codeでプロジェクトを開きます
3. VSCodeが「Reopen in Container」を提案したら、それをクリックします
   - もしくはコマンドパレット(`Ctrl+Shift+P` / `Cmd+Shift+P`)で「Remote-Containers: Reopen in Container」を実行します

DevContainerには以下の開発ツールが含まれています：

- Rust + Cargo (ネイティブコード開発用)
- Node.js + npm/pnpm (JavaScript/TypeScript開発用)
- napi-rs (RustからNode.jsネイティブモジュールを作成するツール)
- TypeScript
- Biome (JavaScript/TypeScriptのリントとフォーマット)
- 各プラットフォーム用の依存ライブラリ

## プロジェクト構造

```
/
├── crate/          # Rustのソースコード
├── src/            # TypeScriptのラッパーコード
├── examples/       # 使用例
├── .devcontainer/  # 開発環境の定義
└── ...
```

## 開発コマンド

```bash
# ビルド
yarn build

# テスト
yarn test
```

## 貢献

プロジェクトへの貢献は大歓迎です。Issue報告、Pull Request、機能提案などお気軽にどうぞ。

## ライセンス

MITライセンス