# コントリビュートガイドライン

electron-pan-clipプロジェクトへの貢献をご検討いただき、ありがとうございます。このドキュメントでは、プロジェクトへの貢献方法と各プラットフォームでの開発に関する注意点をまとめています。

## 開発環境のセットアップ

### 基本的なセットアップ

```bash
# 必要なツールのインストール
just init

# 依存関係のインストール
pnpm install
```

### プラットフォーム別のセットアップ

#### Linux

Linuxでの開発には、X11関連の依存関係が必要です：

```bash
just install-linux-deps
```

#### Windows

Windowsでの開発はネイティブに行うか、Linuxからのクロスコンパイルが可能です：

**Linuxからのクロスコンパイル**:

```bash
# Windows向けクロスコンパイル環境のセットアップ
just setup-windows-cross

# Windows向けのビルド
just build-windows
```

## コードスタイル

### Rust

- `cargo fmt`でフォーマットされた状態であること
- `cargo clippy`で警告がないこと
- プラットフォーム固有の実装は `src/platforms/` 以下に配置

### TypeScript/JavaScript

- `biome`でフォーマットされた状態であること
- TypeScriptの型定義を適切に提供すること

## テスト

各プラットフォーム向けの実装には、適切なテストを含めてください：

```bash
# 全てのテストを実行
just test

# Linuxでの特別なテスト（X11関連）
just test-with-xvfb
```

## プラットフォーム別の実装に関する注意点

### Windows実装

- `windows-sys`クレート（バージョン0.52以上）を使用
- `CF_HDROP`形式（値 = 15）は明示的に定義する必要がある
- `DROPFILES`構造体を手動で実装
- ファイルパスはUTF-16（ワイド文字列）に変換してNULL終端すること
- グローバルメモリの確保、ロック/アンロックを適切に管理する
- ポインタ型の比較は`ptr::null_mut()`を使用する（`0`との直接比較は避ける）
- `SetClipboardData`の引数型には注意（ポインタを`isize`にキャストする必要がある）

### macOS実装

（内容を追加予定）

### Linux実装

（内容を追加予定）

## プルリクエスト提出時のチェックリスト

プルリクエストを提出する前に、以下の点を確認してください：

1. すべてのテストが通過すること
2. 新しい機能には適切なテストが追加されていること
3. ドキュメントが更新されていること
4. コードスタイルが適切であること
5. コミットメッセージがConventional Commitsの形式に従っていること

```bash
# 全てのチェックを実行
just ready
```

## コミットメッセージの形式

このプロジェクトでは、[Conventional Commits](https://www.conventionalcommits.org/)の形式を採用しています：

```
<type>[(optional scope)]: <description>

[optional body]

[optional footer]
```

例：
- `fix(windows): ファイルパスの処理の修正`
- `feat(core): 複数ファイルのコピーサポート追加`
- `docs: READMEの更新` 