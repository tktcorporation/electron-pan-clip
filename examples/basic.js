const { copyFilePathsToClipboard } = require("clip-filepaths");

// 複数のファイルをクリップボードにコピーする例
try {
	// ファイルパスはOSに合わせて適切に指定してください
	const filePaths = ["/path/to/file1.txt", "/path/to/file2.jpg"];

	copyFilePathsToClipboard(filePaths);
	console.log("ファイルをクリップボードにコピーしました");
} catch (error) {
	console.error("エラーが発生しました:", error);
}
