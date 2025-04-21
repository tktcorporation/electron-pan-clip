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

## ライセンス

MIT 