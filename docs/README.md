# electron-pan-clip

マルチプラットフォーム対応のElectronアプリケーション用ファイルクリップボードユーティリティ

## プロジェクト構造

```
electron-pan-clip/
├── docs/               # ドキュメント
│   ├── api/            # API ドキュメント
│   └── README.md       # プロジェクト概要
├── examples/           # 使用例
├── src/                # ソースコード
│   ├── platforms/      # プラットフォーム別実装
│   │   ├── windows/    # Windows実装
│   │   ├── macos/      # macOS実装
│   │   └── linux/      # Linux実装
│   └── lib.rs          # メインライブラリ実装
├── tests/              # テスト
│   ├── unit/           # ユニットテスト
│   └── integration/    # 統合テスト
├── index.js            # JavaScript APIエントリポイント
└── index.d.ts          # TypeScript型定義
```

## 開発

### 依存関係のインストール

```bash
yarn install
```

### ビルド

```bash
yarn build
```

### テスト

```bash
yarn test
```

## プラットフォーム別の実装に関する注意点

### Windows実装

Windows実装では、ファイルをクリップボードにコピーするために、`CF_HDROP`形式と`DROPFILES`構造体を使用しています。主な特徴:

- `windows-sys` crate（バージョン0.52以上）を使用
- `CF_HDROP`形式（値 = 15）を明示的に定義
- `DROPFILES`構造体を手動で実装
- ファイルパスをUTF-16（ワイド文字列）に変換してNULL終端
- グローバルメモリの確保、ロック/アンロックを適切に管理

注意点:
- クロスプラットフォームのビルド時には `cargo-xwin` ツールと、適切なターゲット（`x86_64-pc-windows-msvc`など）が必要
- エラーハンドリングが充実しており、メモリリークを防止
- エラー発生時には詳細なエラーメッセージを提供

`rustup target add x86_64-pc-windows-msvc` でWindows向けのクロスコンパイル環境を準備できます。

### macOS実装

（内容を追加予定）

### Linux実装

（内容を追加予定）

## ライセンス

MIT 