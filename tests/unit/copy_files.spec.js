const { copyFiles, helloWorld } = require("../../index");
const fs = require("node:fs");
const path = require("node:path");
const os = require("node:os");

describe("electron-pan-clip", () => {
	describe("helloWorld", () => {
		it("should return a platform-specific greeting", () => {
			const result = helloWorld();
			expect(result).toContain("Rust");

			// OSごとに異なるメッセージを返すか確認
			const platform = os.platform();
			if (platform === "win32") {
				expect(result).toContain("Windows");
			} else if (platform === "darwin") {
				expect(result).toContain("macOS");
			} else if (platform === "linux") {
				expect(result).toContain("Linux");
			}
		});
	});

	describe("copyFiles", () => {
		let testFiles = [];

		// 各テスト前に一時ファイルを作成
		beforeEach(() => {
			// テスト用の一時ファイルを2つ作成
			testFiles = Array(2)
				.fill()
				.map((_, i) => {
					const filePath = path.join(
						os.tmpdir(),
						`electron_pan_clip_js_test_${i}.txt`,
					);
					fs.writeFileSync(filePath, `Test content ${i}`);
					return filePath;
				});
		});

		// 各テスト後に一時ファイルを削除
		afterEach(() => {
			for (const file of testFiles) {
				try {
					fs.unlinkSync(file);
				} catch (e) {
					// ファイルが存在しない場合は無視
				}
			}
			testFiles = [];
		});

		it("should reject empty array", () => {
			expect(() => copyFiles([])).toThrow(/No file paths provided/);
		});

		it("should copy existing files to clipboard", () => {
			// 実際のファイルをクリップボードにコピー
			// 注意: このテストは実際のクリップボードを変更します
			expect(() => copyFiles(testFiles)).not.toThrow();

			// 注: クリップボードの内容を自動的に検証するのは難しいため、
			// エラーが発生しないことだけを確認しています
		});

		it("should throw an error for non-existent files", () => {
			const nonExistentFiles = [
				path.join(os.tmpdir(), "non_existent_file_1.txt"),
				path.join(os.tmpdir(), "non_existent_file_2.txt"),
			];

			// 存在しないファイルをコピーしようとするとエラーになるはず
			expect(() => copyFiles(nonExistentFiles)).toThrow();
		});
	});
}); 