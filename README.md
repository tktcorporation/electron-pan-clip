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

# npm自動パブリッシュワークフロー

このリポジトリには、npmパッケージを自動的にパブリッシュするためのGitHub Actionsワークフローが含まれています。

## セットアップ手順

### 1. NPMトークンの設定

1. npmアカウントで新しいアクセストークンを生成します：
   - [npmのウェブサイト](https://www.npmjs.com/)にログイン
   - 右上のプロフィールアイコン → Access Tokens → Generate New Token
   - トークンをコピー

2. GitHubリポジトリに`NPM_TOKEN`シークレットを追加します：
   - リポジトリの「Settings」→「Secrets and variables」→「Actions」
   - 「New repository secret」をクリック
   - 名前: `NPM_TOKEN`、値: コピーしたnpmトークン

### 2. コンベンショナルコミットの利用

このワークフローでは、[git-cliff](https://github.com/orhun/git-cliff)を使用してコミット履歴から自動的にCHANGELOGを生成します。効果的に活用するために、以下のコミットメッセージ形式（[Conventional Commits](https://www.conventionalcommits.org/)）を採用してください：

```
<type>[(optional scope)]: <description>

[optional body]

[optional footer]
```

主なタイプ:
- `feat`: 新機能の追加
- `fix`: バグ修正
- `doc`: ドキュメントの変更のみ
- `perf`: パフォーマンスを向上させるコード変更
- `refactor`: バグ修正でも機能追加でもないコード変更
- `style`: コードの意味に影響しない変更（空白、フォーマット、セミコロン追加など）
- `test`: テストの追加や修正
- `chore`: その他の変更（ビルドプロセスなど）

例:
```
feat(core): ファイルコピー機能の追加
fix(windows): Windowsプラットフォームでのパス解決の問題を修正
doc: READMEの更新
```

**注意**:
- スコープを含めると、変更されたコンポーネントやモジュールを明示できます（例: `feat(core):`, `fix(windows):`）
- 破壊的変更を含む場合は、コミットタイプやスコープの後に `!` を追加するか、フッターに `BREAKING CHANGE:` を記述します

## ワークフローの使い方

### バージョンの更新

1. GitHubリポジトリの「Actions」タブを開く
2. 「Version Bump」ワークフローを選択
3. 「Run workflow」をクリック
4. 更新タイプ（patch, minor, major）を選択して実行

これにより以下が自動的に行われます：
- パッケージバージョンが更新される
- git-cliffによるCHANGELOGの自動生成
- 変更のコミットとタグ作成
- 新しいタグのプッシュ

### リリースの作成

「Version Bump」ワークフローが完了すると、「Create Release」ワークフローが自動的に起動します：
- 自動生成されたCHANGELOGからリリースノートを抽出
- GitHub Releaseを作成
- リリース作成により「Publish NPM Package」ワークフローが起動

### npmへのパブリッシュ

リリースが作成されると、「Publish NPM Package」ワークフローが自動的に実行されます：
- コードをチェックアウト
- 依存関係をインストール
- テストとビルドを実行
- npmにパッケージをパブリッシュ

## 手動パブリッシュ

リポジトリの「Actions」タブから「Publish NPM Package」ワークフローを選択し、「Run workflow」をクリックすることで、手動でパブリッシュすることも可能です。