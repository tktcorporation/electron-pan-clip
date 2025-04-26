#!/usr/bin/env node

/**
 * このスクリプトは、ビルドされたすべてのバイナリが適切に配置されていることを検証します。
 * CI/CDパイプラインで使用され、バイナリがすべてのプラットフォーム用に正しく生成されたかをチェックします。
 */

const fs = require("node:fs");
const path = require("node:path");

// 検証するプラットフォーム
const platforms = [
	"linux-x64-gnu",
	"darwin-x64",
	"darwin-arm64",
	"win32-x64-msvc",
	"win32-arm64-msvc",
];

// 結果を保存する配列
const results = [];
let hasError = false;

// 各プラットフォームのバイナリを検証
for (const platform of platforms) {
	const binaryPath = path.join(
		".", // Look in the current directory
		`electron-pan-clip.${platform}.node`,
	);

	console.log(`検証中: ${binaryPath}`);

	if (!fs.existsSync(binaryPath)) {
		console.error(`エラー: ${binaryPath} が見つかりません`);
		results.push({ platform, exists: false, size: 0 });
		hasError = true;
		continue;
	}

	// ファイルサイズを取得
	const stats = fs.statSync(binaryPath);
	const sizeMB = (stats.size / (1024 * 1024)).toFixed(2);

	if (stats.size < 10000) {
		// 最小サイズを10KBとする
		console.error(
			`エラー: ${binaryPath} のサイズが小さすぎます (${sizeMB} MB)`,
		);
		results.push({ platform, exists: true, size: stats.size, valid: false });
		hasError = true;
		continue;
	}

	console.log(`成功: ${binaryPath} (${sizeMB} MB)`);
	results.push({ platform, exists: true, size: stats.size, valid: true });
}

// 結果を表示
console.log("\n検証結果:");
console.table(results);

// エラーがあれば終了コード1で終了
if (hasError) {
	console.error("一部のバイナリが検証に失敗しました。");
	process.exit(1);
}

console.log("すべてのバイナリが正常に検証されました。");
process.exit(0);
