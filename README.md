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

## 開発コンテナでのX11サポート

開発コンテナでLinuxのX11関連のテストを実行するための設定手順：

1. 必要な依存関係のインストール
   ```bash
   just install-linux-deps
   ```

2. Xvfbを使用したテストの実行
   ```bash
   just test-with-xvfb
   ```

Xvfbは仮想フレームバッファを提供するX11サーバーで、GUIを持たずにX11アプリケーションを実行できます。
これにより、ヘッドレス環境（実際のディスプレイがない環境）でもクリップボード操作のテストが可能になります。

### トラブルシューティング

X11関連の問題が発生した場合は、以下を確認してください：

1. X11の依存関係が正しくインストールされているか
   ```bash
   dpkg -l | grep x11
   ```

2. Xvfbが正常に動作しているか
   ```bash
   Xvfb :99 -screen 0 1280x1024x24 &
   export DISPLAY=:99
   xdpyinfo | head  # X11サーバーの情報を表示
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

## リリースワークフロー

このリポジトリには2種類のリリースワークフローがあります：

### 1. 自動リリース（Auto Release）

`package.json`のバージョン変更を検出して自動的にリリースプロセスを開始します：

1. **バージョン変更検出**: mainブランチで`package.json`のバージョン変更を検出
2. **CHANGELOG生成**: git-cliffを使用してコミット履歴からCHANGELOGを自動生成
3. **リリース作成**: GitHubリリースを作成し、タグを付与
4. **npmパブリッシュ**: npmレジストリにパッケージをパブリッシュ
5. **スモークテスト**: パブリッシュ後にインストールと基本機能のテストを実行

このワークフローを使用するには、単に`package.json`のバージョンを更新してmainブランチにプッシュするだけです。以降のプロセスは自動的に実行されます。

### 2. 手動リリース（Version Bump）

手動でバージョン更新とリリースを行うワークフローです：

1. GitHubリポジトリの「Actions」タブを開く
2. 「Version Bump」ワークフローを選択
3. 「Run workflow」をクリック
4. 更新タイプ（patch, minor, major）を選択して実行

これにより以下の処理が実行されます：
- パッケージバージョンが更新される
- git-cliffによるCHANGELOGの自動生成
- 変更のコミットとタグ作成
- 新しいタグのプッシュ