const { copyFiles } = require("electron-pan-clip");

// 複数のファイルをクリップボードにコピーする例
try {
	// ファイルパスはOSに合わせて適切に指定してください
	const filePaths = ["/path/to/file1.txt", "/path/to/file2.jpg"];

	copyFiles(filePaths);
	console.log("ファイルをクリップボードにコピーしました");
} catch (error) {
	console.error("エラーが発生しました:", error);
}
