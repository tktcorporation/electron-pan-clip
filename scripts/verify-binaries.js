#!/usr/bin/env node

/**
 * 公開前にすべての必要なバイナリが存在するか確認するスクリプト
 */

const fs = require('fs');
const path = require('path');
const { exit } = require('process');

// 確認すべきバイナリのリスト
const requiredBinaries = [
  'linux-x64-gnu/electron-pan-clip.linux-x64-gnu.node',
  'linux-arm64-gnu/electron-pan-clip.linux-arm64-gnu.node',
  'darwin-x64/electron-pan-clip.darwin-x64.node',
  'darwin-arm64/electron-pan-clip.darwin-arm64.node',
  'win32-x64-msvc/electron-pan-clip.win32-x64-msvc.node'
];

// オプショナルバイナリとして扱うプラットフォーム
// これらは一部の環境で構築が難しい場合がある
const optionalBinaries = [
  'linux-arm64-gnu/electron-pan-clip.linux-arm64-gnu.node',
  'darwin-arm64/electron-pan-clip.darwin-arm64.node'
];

// パッケージディレクトリでの実行を想定
const npmDir = path.join(__dirname, '..', 'npm');

// package.jsonから必要なプラットフォームを取得
function getPlatformsFromPackageJson() {
  try {
    const packageJson = require('../package.json');
    if (packageJson.optionalDependencies) {
      // optionalDependenciesに記載されているプラットフォームを取得
      return Object.keys(packageJson.optionalDependencies)
        .filter(dep => dep.startsWith('electron-pan-clip-'))
        .map(dep => {
          const platform = dep.replace('electron-pan-clip-', '');
          return `${platform}/electron-pan-clip.${platform}.node`;
        });
    }
  } catch (error) {
    console.warn('package.jsonの読み込みに失敗しました:', error.message);
  }
  return requiredBinaries;
}

const platformsToCheck = getPlatformsFromPackageJson();
console.log('バイナリファイル検証を開始します...');
console.log(`検証対象のプラットフォーム: ${platformsToCheck.length}個`);

let missingBinaries = [];
let warningBinaries = [];

// NPMディレクトリが存在するか確認
if (!fs.existsSync(npmDir)) {
  console.error(`エラー: "${npmDir}" ディレクトリが見つかりません。`);
  console.error('napi prepublish コマンドが正常に実行されていない可能性があります。');
  exit(1);
}

// 各バイナリの存在を確認
for (const binary of platformsToCheck) {
  const binaryPath = path.join(npmDir, binary);
  const isOptional = optionalBinaries.includes(binary);
  
  if (!fs.existsSync(binaryPath)) {
    if (isOptional) {
      warningBinaries.push(binary);
      console.warn(`警告: オプショナルバイナリ "${binary}" が見つかりません。`);
    } else {
      missingBinaries.push(binary);
      console.error(`警告: 必須バイナリ "${binary}" が見つかりません。`);
    }
  } else {
    // ファイルサイズを確認
    const stats = fs.statSync(binaryPath);
    const fileSizeInKB = Math.round(stats.size / 1024);
    
    if (fileSizeInKB < 10) {
      console.warn(`警告: "${binary}" のサイズが小さすぎます (${fileSizeInKB}KB)。正常にビルドされていない可能性があります。`);
      if (!isOptional) {
        missingBinaries.push(binary);
      }
    } else {
      console.log(`✅ "${binary}" が見つかりました (${fileSizeInKB}KB)`);
    }
  }
}

// 存在するすべてのバイナリを表示
console.log('\n実際に存在するバイナリファイル:');
try {
  // 各サブディレクトリを確認
  const subDirs = fs.readdirSync(npmDir);
  for (const subDir of subDirs) {
    const subDirPath = path.join(npmDir, subDir);
    if (fs.statSync(subDirPath).isDirectory()) {
      const files = fs.readdirSync(subDirPath);
      const nodeFiles = files.filter(file => file.endsWith('.node'));
      nodeFiles.forEach(file => console.log(`- ${subDir}/${file}`));
    }
  }
} catch (err) {
  console.error('バイナリファイルのリスト取得中にエラーが発生しました:', err);
}

// 結果出力
if (missingBinaries.length > 0) {
  console.error('\n❌ 以下の必須バイナリファイルが見つかりません:');
  missingBinaries.forEach(binary => console.error(`- ${binary}`));
  console.error('\nビルドプロセスに問題があるか、対象プラットフォームが正しく設定されていない可能性があります。');
  console.error('package.jsonの"napi.triples"セクションを確認し、すべての必要なプラットフォームが含まれていることを確認してください。');
  exit(1);
} else if (warningBinaries.length > 0) {
  console.warn('\n⚠️ 以下のオプショナルバイナリファイルが見つかりません:');
  warningBinaries.forEach(binary => console.warn(`- ${binary}`));
  console.warn('これらは一部の環境で構築が難しい場合があるため、オプショナルとして扱われます。');
  console.log('\n✅ 必須バイナリはすべて存在します。');
} else {
  console.log('\n✅ すべての必要なバイナリファイルが見つかりました。');
} 