# electron-pan-clip

マルチプラットフォーム対応のElectronアプリケーション用ファイルクリップボードユーティリティ

## 概要

このライブラリは、Electronアプリケーションでファイルをクリップボードにコピーするための機能を提供します。Windows、macOS、Linuxの各プラットフォームに対応しています。

## インストール

```bash
yarn add electron-pan-clip
```

または

```bash
npm install electron-pan-clip
```

## 使用方法

```javascript
const { copyFiles } = require('electron-pan-clip');

// 複数のファイルをクリップボードにコピー
try {
  const filePaths = ['/path/to/file1.txt', '/path/to/file2.jpg'];
  copyFiles(filePaths);
  console.log('ファイルをクリップボードにコピーしました');
} catch (error) {
  console.error('エラーが発生しました:', error);
}
```

詳細な使用例は [examples](./examples) ディレクトリを参照してください。

## pnpm の使い方

このプロジェクトはpnpm対応になりました。以下のコマンドで開発を始めることができます：

```bash
# 依存関係のインストール
pnpm install

# デバッグビルド
pnpm build:debug

# リリースビルド
pnpm build

# テスト実行
pnpm test

# ドキュメント生成
pnpm docs
```

## 開発

このプロジェクトは[napi-rs](https://napi.rs/)を使用しており、Node.jsのネイティブアドオンをRustで作成します。

詳細な開発情報は [docs/README.md](./docs/README.md) を参照してください。

### セットアップ

```bash
# 依存関係のインストール
yarn install

# ビルド
yarn build

# テスト
yarn test
```

## ライセンス

MIT