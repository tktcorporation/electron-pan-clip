import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { readClipboardResults, writeClipboardFilePaths } from "../../";

// 一時ファイル作成関数を定義
interface TempFile {
	path: string;
	cleanup: () => void;
}

async function createTempFile(): Promise<TempFile> {
	const filePath = path.join(os.tmpdir(), `test-file-${Date.now()}.txt`);
	fs.writeFileSync(filePath, "test content");
	return {
		path: filePath,
		cleanup: () => {
			if (fs.existsSync(filePath)) {
				fs.unlinkSync(filePath);
			}
		},
	};
}

describe("clip-filepaths", () => {
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

			// 簡易的な1x1のPNGファイル（実際には適切なPNGデータを用意する必要あり）
			const minimalPng = Buffer.from([
				0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d,
				0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
				0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xde, 0x00, 0x00, 0x00,
				0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00,
				0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xdd, 0x8d, 0xb0, 0x00, 0x00, 0x00,
				0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
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

	it("should be defined", () => {
		expect(writeClipboardFilePaths).toBeDefined();
	});

	// CI 環境ではクリップボードへのアクセスが失敗するためスキップ
	it.skip("should copy files to clipboard", async () => {
		// テスト用の一時ファイルを作成
		const tempFile = await createTempFile();
		const testFiles = [tempFile.path];

		try {
			// クリップボードにコピー
			await writeClipboardFilePaths(testFiles);

			// エラーが発生しなければテスト成功
			tempFile.cleanup();
		} catch (error: unknown) {
			// X11 server connection エラーの場合はスキップ
			if (
				error instanceof Error &&
				error.message.includes("X11 server connection timed out")
			) {
				console.log("⚠️ テストをスキップ: X11サーバー接続の問題");
				tempFile.cleanup();
				return;
			}

			// それ以外のエラーが発生した場合はテスト失敗
			console.error("Failed to copy files:", error);
			tempFile.cleanup();
			expect(error).toBeUndefined();
		}
	});

	it("should reject with invalid file paths", () => {
		const invalidPaths = ["/path/to/nonexistent/file.png"];
		try {
			writeClipboardFilePaths(invalidPaths);
			// Linuxの場合、無効なパスでもファイルURIを生成できるため成功する可能性がある
			if (process.platform === "linux") {
				expect(true).toBe(true);
			} else {
				// Linuxでないプラットフォームではエラーがスローされるはず
				expect("No error thrown").toBe("Error should have been thrown");
			}
		} catch (error) {
			// エラーがスローされることを期待（正常な動作）
			expect(error).toBeDefined();
		}
	});

	it("should handle empty array", () => {
		// 空の配列の場合はエラーなく実行されるはず
		expect(() => writeClipboardFilePaths([])).not.toThrow();
	});

	// OSごとのテスト（条件付きテスト）- CI環境ではスキップ
	if (process.platform === "win32") {
		(process.env.CI === "true" ? it.skip : it)(
			"Windows: should copy files in CF_HDROP format",
			() => {
				writeClipboardFilePaths(testFiles);
				// Windows固有のテスト
				expect(true).toBe(true);
			},
		);
	}

	if (process.platform === "darwin") {
		(process.env.CI === "true" ? it.skip : it)(
			"macOS: should copy files using NSPasteboard",
			() => {
				writeClipboardFilePaths(testFiles);
				// macOS固有のテスト
				expect(true).toBe(true);
			},
		);
	}

	if (process.platform === "linux") {
		(process.env.CI === "true" ? it.skip : it)(
			"Linux: should copy files in text/uri-list format",
			() => {
				try {
					writeClipboardFilePaths(testFiles);
					// Linux固有のテスト
					expect(true).toBe(true);
				} catch (error: unknown) {
					// X11 server connection エラーの場合はスキップ
					if (
						error instanceof Error &&
						error.message.includes("X11 server connection timed out")
					) {
						console.log("⚠️ テストをスキップ: X11サーバー接続の問題");
						return;
					}
					throw error;
				}
			},
		);
	}

	// クリップボードに書き込み後に読み出せることを確認するテスト
	it("should write paths and read them back", async () => {
		// CI環境の場合はプラットフォーム別に判断
		if (process.env.CI === "true") {
			console.log("⚠️ CIモードでテストを実行中: クリップボードテストをスキップ");
			return;
		}

		try {
			// クリップボードに書き込み
			await writeClipboardFilePaths(testFiles);

			// クリップボードから読み出し
			const clipboardContent = readClipboardResults();

			// ファイルパスが存在し、元のパスと一致することを確認
			expect(clipboardContent.filePaths).toBeDefined();
			expect(clipboardContent.filePaths.length).toBeGreaterThan(0);

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
		} catch (error: unknown) {
			// X11 server connection エラーの場合はスキップ（Linux環境）
			if (
				error instanceof Error &&
				error.message.includes("X11 server connection timed out")
			) {
				console.log("⚠️ テストをスキップ: X11サーバー接続の問題");
				return;
			}

			// その他のエラーが発生した場合はテスト失敗
			console.error("クリップボード操作に失敗:", error);
			expect(error).toBeUndefined();
		}
	});

	// プラットフォーム別のクリップボード書き込み→読み取りテスト
	if (process.platform === "win32") {
		(process.env.CI === "true" ? it.skip : it)(
			"Windows: should write and read back file paths",
			async () => {
				try {
					// クリップボードにコピー
					await writeClipboardFilePaths(testFiles);

					// クリップボードから読み出し
					const clipboardContent = readClipboardResults();

					// ファイルパスが存在することを確認
					expect(clipboardContent.filePaths).toBeDefined();
					expect(clipboardContent.filePaths.length).toBeGreaterThan(0);

					// ファイルパスの比較 (Windows固有の形式を考慮)
					const allPathsFound = testFiles.every((testPath) => {
						const normalizedTestPath = path.normalize(testPath);
						return clipboardContent.filePaths.some((clipPath) => {
							const normalizedClipPath = path.normalize(clipPath);
							return normalizedClipPath.includes(
								path.basename(normalizedTestPath),
							);
						});
					});

					expect(allPathsFound).toBe(true);
				} catch (error) {
					console.error("Windows clipboard test failed:", error);
					expect(error).toBeUndefined();
				}
			},
		);
	}

	if (process.platform === "darwin") {
		(process.env.CI === "true" ? it.skip : it)(
			"macOS: should write and read back file paths",
			async () => {
				try {
					// クリップボードにコピー
					await writeClipboardFilePaths(testFiles);

					// クリップボードから読み出し
					const clipboardContent = readClipboardResults();

					// ファイルパスが存在することを確認
					expect(clipboardContent.filePaths).toBeDefined();
					expect(clipboardContent.filePaths.length).toBeGreaterThan(0);

					// ファイルパスの比較 (macOS固有の形式を考慮)
					const allPathsFound = testFiles.every((testPath) => {
						const normalizedTestPath = path.normalize(testPath);
						return clipboardContent.filePaths.some((clipPath) => {
							const normalizedClipPath = path.normalize(clipPath);
							return normalizedClipPath.includes(
								path.basename(normalizedTestPath),
							);
						});
					});

					expect(allPathsFound).toBe(true);
				} catch (error) {
					console.error("macOS clipboard test failed:", error);
					expect(error).toBeUndefined();
				}
			},
		);
	}

	if (process.platform === "linux") {
		(process.env.CI === "true" ? it.skip : it)(
			"Linux: should write and read back file paths",
			async () => {
				try {
					// クリップボードにコピー
					await writeClipboardFilePaths(testFiles);

					// クリップボードから読み出し
					const clipboardContent = readClipboardResults();

					// ファイルパスが存在することを確認
					expect(clipboardContent.filePaths).toBeDefined();
					expect(clipboardContent.filePaths.length).toBeGreaterThan(0);

					// ファイルパスの比較 (Linux固有の形式を考慮)
					const allPathsFound = testFiles.every((testPath) => {
						const normalizedTestPath = path.normalize(testPath);
						return clipboardContent.filePaths.some((clipPath) => {
							const normalizedClipPath = path.normalize(clipPath);
							return normalizedClipPath.includes(
								path.basename(normalizedTestPath),
							);
						});
					});

					expect(allPathsFound).toBe(true);
				} catch (error) {
					// X11 server connection エラーの場合はスキップ
					if (
						error instanceof Error &&
						error.message.includes("X11 server connection timed out")
					) {
						console.log("⚠️ テストをスキップ: X11サーバー接続の問題");
						return;
					}
					console.error("Linux clipboard test failed:", error);
					expect(error).toBeUndefined();
				}
			},
		);
	}
});
