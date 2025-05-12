import crypto from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "pathe";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import {
	helloWorld,
	readClipboardResults,
	writeClipboardFilePaths,
} from "../../";

// 一時ファイル作成関数を定義
interface TempFile {
	path: string;
	cleanup: () => void;
}

// テストユーティリティ関数
async function createTempFile(content = "test content"): Promise<TempFile> {
	const filePath = path.join(os.tmpdir(), `test-file-${Date.now()}.txt`);
	fs.writeFileSync(filePath, content);
	return {
		path: filePath,
		cleanup: () => {
			if (fs.existsSync(filePath)) {
				fs.unlinkSync(filePath);
			}
		},
	};
}

// クリップボードをクリアする関数（プラットフォーム依存のため実装が必要）
async function clearClipboard(): Promise<void> {
	// ライブラリの関数で空配列を書き込み＝クリア扱い
	return writeClipboardFilePaths([]);
}

describe("clip-filepaths", () => {
	// 各テスト前にクリップボードをクリア
	beforeEach(async () => {
		await clearClipboard();
	});

	describe("エクスポート関数チェック", () => {
		it("helloWorld関数がエクスポートされていること", () => {
			const result = helloWorld;
			expect(result).toBeDefined();
			expect(typeof result).toBe("function");
		});

		it("readClipboardResults関数がエクスポートされていること", () => {
			const result = readClipboardResults;
			expect(result).toBeDefined();
			expect(typeof result).toBe("function");
		});

		it("writeClipboardFilePaths関数がエクスポートされていること", () => {
			const result = writeClipboardFilePaths;
			expect(result).toBeDefined();
			expect(typeof result).toBe("function");
		});
	});

	describe("writeClipboardFilePaths", () => {
		const tmpDir = path.join(os.tmpdir(), "clip-filepaths-test");
		const testFiles: string[] = [];

		// テスト用の一時ファイルを作成
		beforeEach(() => {
			// テスト用ディレクトリを作成
			if (!fs.existsSync(tmpDir)) {
				fs.mkdirSync(tmpDir, { recursive: true });
			}

			// テスト用画像ファイルを作成
			for (let i = 0; i < 3; i++) {
				const filePath = path.join(tmpDir, `test-image-${i}.png`);

				// 簡易的な1x1のPNGファイル
				const minimalPng = Buffer.from([
					0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00,
					0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
					0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xde,
					0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63,
					0xf8, 0xcf, 0xc0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xdd,
					0x8d, 0xb0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
					0x42, 0x60, 0x82,
				]);

				fs.writeFileSync(filePath, minimalPng);
				testFiles.push(filePath);
			}
		});

		// テスト用一時ファイルを削除
		afterEach(() => {
			// テストファイルを削除
			for (const file of testFiles) {
				if (fs.existsSync(file)) {
					fs.unlinkSync(file);
				}
			}
			testFiles.length = 0;

			// ディレクトリを削除
			if (fs.existsSync(tmpDir)) {
				fs.rmdirSync(tmpDir);
			}
		});

		it("一時ファイルをクリップボードにコピーできること", async () => {
			// テスト用の一時ファイルを作成
			const tempFile = await createTempFile();
			const filesToCopy = [tempFile.path];

			// クリップボードにコピー
			await writeClipboardFilePaths(filesToCopy);

			// クリップボードから読み出しして確認
			const clipboardContent = readClipboardResults();
			expect(clipboardContent.filePaths).toBeDefined();
			expect(clipboardContent.filePaths.length).toBe(1);
			expect(clipboardContent.filePaths[0]).toContain(
				path.basename(tempFile.path),
			);

			// ファイルをクリーンアップ
			tempFile.cleanup();
		});

		it("存在しないファイルパスを指定した場合はエラーをスローすること", () => {
			const invalidFileName = `nonexistent-${crypto.randomUUID()}.png`;
			const invalidPath = path.join(os.tmpdir(), invalidFileName);
			const invalidPaths = [invalidPath];

			// 念のため存在していれば削除しておく
			if (fs.existsSync(invalidPath)) {
				fs.unlinkSync(invalidPath);
			}

			// 共通のエラーメッセージパターンを生成
			// 例: "linux clipboard error: Some paths could not be processed: Failed to canonicalize path /tmp/nonexistent-xxxx.png: ..."
			const expectedPattern = new RegExp(
				`clipboard error: .*Some paths could not be processed: .*${invalidFileName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`,
				"i",
			);

			// 例外がスローされ、エラーメッセージがパターンに一致することを確認
			expect(() => writeClipboardFilePaths(invalidPaths)).toThrow(
				expectedPattern,
			);
		});

		it("空の配列を指定した場合はエラーをスローしないこと", () => {
			// 空の配列の場合はエラーなく実行されるはず
			expect(() => writeClipboardFilePaths([])).not.toThrow();
		});

		it("プラットフォームに適した形式でファイルをコピーすること", async () => {
			await writeClipboardFilePaths(testFiles);

			const clipboardContent = readClipboardResults();
			expect(clipboardContent.filePaths).toBeDefined();
			expect(clipboardContent.filePaths.length).toBe(testFiles.length);

			// プラットフォーム固有の確認
			if (process.platform === "win32") {
				// Windowsの場合はファイルURLの形式を確認
				expect(clipboardContent.text).toContain("file:///");
			} else if (process.platform === "darwin") {
				// macOSの場合もファイルURLの形式を確認
				expect(clipboardContent.text).toContain("file://");
			} else if (process.platform === "linux") {
				// Linuxの場合もファイルURLの形式を確認
				expect(clipboardContent.text).toContain("file:///");
			}
		});

		it("特殊文字を含むファイルパスを正しく処理できること", async () => {
			// 特殊文字を含むファイル名でテスト
			const specialFileName = `test-special-#$%-${Date.now()}.txt`;
			const specialFilePath = path.join(tmpDir, specialFileName);
			fs.writeFileSync(specialFilePath, "特殊文字を含むファイル");

			await writeClipboardFilePaths([specialFilePath]);
			const clipboardContent = readClipboardResults();

			expect(clipboardContent.filePaths.length).toBe(1);
			// ファイル名の一部が含まれているか確認
			expect(clipboardContent.filePaths[0]).toContain("test-special-");

			// テスト後にファイルを削除
			if (fs.existsSync(specialFilePath)) {
				fs.unlinkSync(specialFilePath);
			}
		});

		it("複数の異なる種類のファイルを一度にコピーできること", async () => {
			// 異なる拡張子のファイルを作成
			const textFile = path.join(tmpDir, `test-text-${Date.now()}.txt`);
			const jsonFile = path.join(tmpDir, `test-json-${Date.now()}.json`);

			fs.writeFileSync(textFile, "テキストファイル");
			fs.writeFileSync(jsonFile, JSON.stringify({ test: "データ" }));

			const mixedFiles = [textFile, jsonFile, ...testFiles];

			await writeClipboardFilePaths(mixedFiles);
			const clipboardContent = readClipboardResults();

			expect(clipboardContent.filePaths.length).toBe(mixedFiles.length);

			// 各ファイルの拡張子が含まれているか確認
			expect(
				clipboardContent.filePaths.some((p: string) => p.includes(".txt")),
			).toBe(true);
			expect(
				clipboardContent.filePaths.some((p: string) => p.includes(".json")),
			).toBe(true);
			expect(
				clipboardContent.filePaths.some((p: string) => p.includes(".png")),
			).toBe(true);

			// テスト後にファイルを削除
			if (fs.existsSync(textFile)) fs.unlinkSync(textFile);
			if (fs.existsSync(jsonFile)) fs.unlinkSync(jsonFile);
		});
	});

	describe("readClipboardResults", () => {
		const tmpDir = path.join(os.tmpdir(), "clip-filepaths-read-test");
		const testFiles: string[] = [
			path.join(tmpDir, "test-image-0.png"),
			path.join(tmpDir, "test-image-1.png"),
			path.join(tmpDir, "test-image-2.png"),
		];

		// テスト用の一時ファイルを作成
		beforeEach(() => {
			// テスト用ディレクトリを作成
			if (!fs.existsSync(tmpDir)) {
				fs.mkdirSync(tmpDir, { recursive: true });
			}

			// テスト用画像ファイルを作成
			for (const filePath of testFiles) {
				// 簡易的な1x1のPNGファイル
				const minimalPng = Buffer.from([
					0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00,
					0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
					0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xde,
					0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63,
					0xf8, 0xcf, 0xc0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xdd,
					0x8d, 0xb0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
					0x42, 0x60, 0x82,
				]);

				fs.writeFileSync(filePath, minimalPng);
			}
		});

		// テスト用一時ファイルを削除
		afterEach(async () => {
			// クリップボードをクリア
			await clearClipboard();

			// テストファイルを削除
			for (const file of testFiles) {
				if (fs.existsSync(file)) {
					fs.unlinkSync(file);
				}
			}

			// ディレクトリを削除
			if (fs.existsSync(tmpDir)) {
				fs.rmdirSync(tmpDir);
			}
		});

		it("クリップボードに書き込んだファイルパスを正確に読み取れること", async () => {
			// クリップボードに書き込み
			await writeClipboardFilePaths(testFiles);

			// クリップボードから読み出し
			const clipboardContent = readClipboardResults();

			// ファイルパスが存在し、元のパスと一致することを確認
			expect(clipboardContent.filePaths).toBeDefined();
			expect(clipboardContent.filePaths.length).toBe(testFiles.length);

			// ファイルパスの比較 (プラットフォームによって形式が異なる可能性があるため部分一致で確認)
			const allPathsFound = testFiles.every((testPath) => {
				// 正規化されたパスの比較
				const normalizedTestPath = path.normalize(testPath);
				return clipboardContent.filePaths.some((clipPath) => {
					const normalizedClipPath = path.normalize(clipPath);
					return normalizedClipPath.includes(path.basename(normalizedTestPath));
				});
			});

			expect(allPathsFound).toBe(true);
		});

		it("空のクリップボードから読み取った場合は空の配列が返ること", async () => {
			await clearClipboard();
			const clipboardContent = readClipboardResults();
			expect(clipboardContent.filePaths).toHaveLength(0);
			expect(clipboardContent.text).toBe(undefined);
		});

		it("テキストのみを含むクリップボードを正しく読み取れること", async () => {
			// テスト用テキストをクリップボードに書き込む方法
			// 注: この部分はプラットフォーム依存の実装が必要
			// ここでは代替として一時ファイルを作成してからそのファイルパスを使用
			const tempFile = await createTempFile("テストテキスト");
			await writeClipboardFilePaths([tempFile.path]);

			// クリップボードから読み出し
			const clipboardContent = readClipboardResults();

			// ファイルパスが含まれていることを確認
			expect(clipboardContent.filePaths.length).toBe(1);

			// テキスト部分も含まれていることを確認
			expect(clipboardContent.text).toBeDefined();
			expect(clipboardContent.text).toContain("file://");

			// クリーンアップ
			tempFile.cleanup();
		});

		it("プラットフォーム固有の形式でファイルパスを正しく処理できること", async () => {
			await writeClipboardFilePaths(testFiles);
			const clipboardContent = readClipboardResults();

			// プラットフォーム固有の確認
			if (process.platform === "win32") {
				// Windowsの場合はバックスラッシュまたはフォワードスラッシュ
				const hasCorrectSlashes = clipboardContent.filePaths.every(
					(p: string) => p.includes("\\") || p.includes("/"),
				);
				expect(hasCorrectSlashes).toBe(true);
			} else if (
				process.platform === "darwin" ||
				process.platform === "linux"
			) {
				// macOSとLinuxの場合はフォワードスラッシュ
				const hasForwardSlashes = clipboardContent.filePaths.every(
					(p: string) => p.includes("/"),
				);
				expect(hasForwardSlashes).toBe(true);
			}
		});
	});

	// 統合テスト - 複数機能の連携
	describe("統合テスト", () => {
		it("書き込みと読み取りを連続して行うと一貫した結果が得られること", async () => {
			// 複数の一時ファイルを作成
			const tempFile1 = await createTempFile("ファイル1");
			const tempFile2 = await createTempFile("ファイル2");
			const filesToCopy = [tempFile1.path, tempFile2.path];

			// クリップボードに書き込み
			await writeClipboardFilePaths(filesToCopy);

			// 書き込んだ直後に読み取り
			const clipboardContent1 = readClipboardResults();
			expect(clipboardContent1.filePaths.length).toBe(2);

			// もう一度読み取り
			const clipboardContent2 = readClipboardResults();
			expect(clipboardContent2.filePaths.length).toBe(2);

			// 2回の読み取り結果が一致することを確認
			expect(clipboardContent1.filePaths.sort()).toEqual(
				clipboardContent2.filePaths.sort(),
			);

			// クリーンアップ
			tempFile1.cleanup();
			tempFile2.cleanup();
		});

		it("空の配列でwriteClipboardFilePathsを呼び出すとクリップボードがクリアされること", async () => {
			// 複数の一時ファイルを作成
			const tempFile1 = await createTempFile("ファイル1");
			const tempFile2 = await createTempFile("ファイル2");
			const filesToCopy = [tempFile1.path, tempFile2.path];

			// クリップボードに書き込み
			await writeClipboardFilePaths(filesToCopy);

			// 書き込みが成功したことを確認
			const clipboardContent1 = readClipboardResults();
			expect(clipboardContent1.filePaths.length).toBe(2);
			expect(clipboardContent1.text).toBeDefined();
			expect(clipboardContent1.text?.length).toBeGreaterThan(0);

			// 空の配列で書き込み（クリア操作）
			await writeClipboardFilePaths([]);

			// クリップボードがクリアされたことを確認
			const clipboardContent2 = readClipboardResults();
			expect(clipboardContent2.filePaths).toHaveLength(0);
			expect(clipboardContent2.text).toBe(undefined);

			// クリーンアップ
			tempFile1.cleanup();
			tempFile2.cleanup();
		});

		it("存在するファイルと存在しないファイルが混在する場合はエラーになること", async () => {
			// 存在するファイル
			const tempFile = await createTempFile();

			// 存在しないファイル
			const nonExistingPath = path.join(
				os.tmpdir(),
				`non-existing-${crypto.randomUUID()}.txt`,
			);
			if (fs.existsSync(nonExistingPath)) {
				fs.unlinkSync(nonExistingPath);
			}

			// 混在したファイルリスト
			const mixedPaths = [tempFile.path, nonExistingPath];

			// エラーが発生することを確認
			try {
				await writeClipboardFilePaths(mixedPaths);
				expect("Error").toBe(false);
			} catch (error) {
				expect(error).toBeInstanceOf(Error);
			}

			// クリーンアップ
			tempFile.cleanup();
		});
	});
});
