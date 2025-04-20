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

## 開発

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