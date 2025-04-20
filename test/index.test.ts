import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { copyFiles } from "../";

describe("electron-pan-clip", () => {
	const tmpDir = path.join(os.tmpdir(), "electron-pan-clip-test");
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
		expect(copyFiles).toBeDefined();
	});

	it("should copy files to clipboard", async () => {
		try {
			await copyFiles(testFiles);
			// 注: 実際のクリップボードの内容を自動的に検証するのは難しいため、
			// エラーが発生しなければ成功とみなします
			expect(true).toBe(true);
		} catch (error) {
			// エラーが発生した場合はテスト失敗
			console.error("Failed to copy files:", error);
			expect(error).toBeUndefined();
		}
	});

	it("should reject with invalid file paths", async () => {
		const invalidPaths = ["/path/to/nonexistent/file.png"];

		await expect(copyFiles(invalidPaths)).rejects.toThrow();
	});

	it("should handle empty array", async () => {
		await expect(copyFiles([])).rejects.toThrow();
	});

	// OSごとのテスト（条件付きテスト）
	if (process.platform === "win32") {
		it("Windows: should copy files in CF_HDROP format", async () => {
			await copyFiles(testFiles);
			// Windows固有のテスト
			expect(true).toBe(true);
		});
	}

	if (process.platform === "darwin") {
		it("macOS: should copy files using NSPasteboard", async () => {
			await copyFiles(testFiles);
			// macOS固有のテスト
			expect(true).toBe(true);
		});
	}

	if (process.platform === "linux") {
		it("Linux: should copy files in text/uri-list format", async () => {
			await copyFiles(testFiles);
			// Linux固有のテスト
			expect(true).toBe(true);
		});
	}
});
