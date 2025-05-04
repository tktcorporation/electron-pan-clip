#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

// OS別の実装モジュール
mod platforms;

#[cfg(target_os = "windows")]
use platforms::windows as current_platform;

#[cfg(target_os = "macos")]
use platforms::macos as current_platform;

#[cfg(target_os = "linux")]
use platforms::linux as current_platform;

/// クリップボードから読み取ったデータを保持する構造体
#[napi(object)]
#[derive(Debug)]
pub struct ClipboardContent {
  /// ファイルパスのリスト。ファイルパスがない場合は空の配列。
  pub file_paths: Vec<String>,

  /// テキスト内容。テキストがない場合はnull。
  pub text: Option<String>,
}

/// Hello World関数 - 動作確認用
#[napi]
pub fn hello_world() -> String {
  #[cfg(target_os = "windows")]
  {
    "Hello from Rust on Windows!".to_string()
  }
  #[cfg(target_os = "macos")]
  {
    "Hello from Rust on macOS!".to_string()
  }
  #[cfg(target_os = "linux")]
  {
    "Hello from Rust on Linux!".to_string()
  }
  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    "Hello from Rust on an unknown OS!".to_string()
  }
}

/// Copies the given list of file paths to the OS clipboard.
///
/// # Arguments
/// * `paths` - A list of absolute or relative file paths to copy.
///   - The paths will be registered to the clipboard in the appropriate format for each OS.
///   - Passing an empty list will result in an error.
///
/// # Returns
/// * Returns `Ok(())` if the operation succeeds.
/// * Returns `Err(napi::Error)` if an error occurs.
///
/// # Note
/// * This function will actually change the contents of the system clipboard.
/// * Please be careful when running tests.
#[napi]
pub fn write_clipboard_file_paths(paths: Vec<String>) -> napi::Result<()> {
  if paths.is_empty() {
    return Err(napi::Error::from_reason("No file paths provided"));
  }

  #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
  {
    current_platform::copy_files_to_clipboard(&paths).map_err(|e| {
      napi::Error::from_reason(format!("{} clipboard error: {}", std::env::consts::OS, e))
    })?;
  }

  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    return Err(napi::Error::from_reason("Unsupported operating system"));
  }

  Ok(())
}

/// Reads raw binary data from the OS clipboard.
///
/// # Returns
/// * Returns `Ok(Vec<u8>)` with the clipboard raw content if successful.
/// * Returns `Err(napi::Error)` if an error occurs.
///
/// # Note
/// * This function reads the current raw contents of the system clipboard.
/// * The format of the data depends on what application wrote to the clipboard.
#[napi]
pub fn read_clipboard_raw() -> napi::Result<Vec<u8>> {
  #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
  {
    current_platform::read_clipboard_raw().map_err(|e| {
      napi::Error::from_reason(format!("{} clipboard error: {}", std::env::consts::OS, e))
    })
  }

  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    return Err(napi::Error::from_reason("Unsupported operating system"));
  }
}

/// Reads content from the OS clipboard, trying to extract both file paths and text.
///
/// # Returns
/// * Returns `Ok(ClipboardContent)` with the clipboard content if successful.
/// * If clipboard is empty, returns an object with empty file_paths and null text.
///
/// # Note
/// * This function attempts to read both file paths and text from the clipboard.
/// * It's possible for both, either, or neither type of data to be present.
#[napi]
pub fn read_clipboard_content() -> napi::Result<ClipboardContent> {
  #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
  {
    // ファイルパスの読み取りを試みる
    let file_paths = current_platform::read_clipboard_file_paths().unwrap_or_default();

    // テキストの読み取りを試みる
    let text = current_platform::read_clipboard_text().ok();

    // どちらも取得できなかった場合でもエラーにせず空データで返す
    Ok(ClipboardContent { file_paths, text })
  }

  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    return Err(napi::Error::from_reason("Unsupported operating system"));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::env::temp_dir;
  use std::fs::File;
  use std::io::Write;

  // hello_world関数のテスト
  #[test]
  fn test_hello_world() {
    let result = hello_world();
    // OSによって出力が異なるため、特定の文字列は検証せず
    // 空でないことだけ確認する
    assert!(!result.is_empty());
    assert!(result.contains("Rust"));
  }

  // 空の入力に対するエラーテスト
  #[test]
  fn test_write_clipboard_file_paths_empty_input() {
    let result = write_clipboard_file_paths(vec![]);
    assert!(result.is_err());
    if let Err(err) = result {
      assert!(err.reason.contains("No file paths provided"));
    }
  }

  // 実際のファイルを作成してコピーするテスト
  // 注意: このテストは実際のクリップボードを変更します
  #[test]
  #[ignore] // 通常のCIでは実行しないよう無視フラグを付ける
  fn test_copy_real_files() {
    // テスト用の一時ファイルを作成
    let mut temp_files = Vec::new();

    for i in 0..2 {
      let mut path = temp_dir();
      path.push(format!("electron_pan_clip_test_{}.txt", i));

      let file_path = path.to_string_lossy().to_string();

      // ファイルを作成して何か書き込む
      let mut file = File::create(&path).expect("Failed to create test file");
      writeln!(file, "Test content {}", i).expect("Failed to write to test file");

      temp_files.push(file_path);
    }

    // ファイルをクリップボードにコピー
    let result = write_clipboard_file_paths(temp_files.clone());

    // コピー成功を確認
    assert!(result.is_ok(), "Failed to copy files: {:?}", result);

    // ここではクリップボードの内容を自動的に検証することは難しいため、
    // 成功したことだけを確認する

    // テスト後にファイルを削除
    for path in temp_files {
      let _ = std::fs::remove_file(path); // エラーは無視
    }
  }

  // RAWデータ読み取り関数のテスト
  #[test]
  #[ignore]
  fn test_read_clipboard_raw() {
    // テスト用のバイナリデータをクリップボードに書き込む必要がある
    // 各プラットフォーム固有のAPIを使用

    // データを読み取り
    let result = read_clipboard_raw();

    // エラーがなければOK（内容の検証は難しいため）
    if let Ok(data) = result {
      assert!(!data.is_empty(), "Raw clipboard data should not be empty");
    }
  }

  // クリップボード内容読み取り関数のテスト（ファイルパスを読み取る場合）
  #[test]
  #[ignore]
  fn test_read_clipboard_content_file_paths() {
    // テスト用の一時ファイルを作成
    let mut test_paths = Vec::new();

    for i in 0..2 {
      let mut path = temp_dir();
      path.push(format!("electron_pan_clip_test_content_{}.txt", i));

      let file_path = path.to_string_lossy().to_string();

      // ファイルを作成
      let mut file = File::create(&path).expect("Failed to create test file");
      writeln!(file, "Test content {}", i).expect("Failed to write to test file");

      test_paths.push(file_path);
    }

    // ファイルパスをクリップボードにコピー
    let copy_result = write_clipboard_file_paths(test_paths.clone());
    assert!(
      copy_result.is_ok(),
      "Failed to copy file paths to clipboard"
    );

    // クリップボード内容を読み取り
    let result = read_clipboard_content();
    assert!(
      result.is_ok(),
      "Failed to read clipboard content: {:?}",
      result
    );

    let content = result.unwrap();

    // ファイルパスが取得できているはず
    assert!(!content.file_paths.is_empty());
    assert_eq!(content.file_paths.len(), test_paths.len());

    // テスト後にファイルを削除
    for path in test_paths {
      let _ = std::fs::remove_file(path);
    }
  }

  // クリップボード内容読み取り関数のテスト（空の場合）
  #[test]
  #[ignore]
  fn test_read_clipboard_content_empty() {
    // クリップボードを消去する方法はプラットフォーム依存
    // ここでは実装していないため、単純にAPI呼び出しのテスト

    let result = read_clipboard_content();
    assert!(result.is_ok(), "Should not fail on read_clipboard_content");

    // エラーは返さず何らかの結果が返ってくること
    let _ = result.unwrap();
  }
}
