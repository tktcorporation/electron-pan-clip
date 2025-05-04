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
pub fn copy_file_paths_to_clipboard(paths: Vec<String>) -> napi::Result<()> {
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
  fn test_copy_file_paths_to_clipboard_empty_input() {
    let result = copy_file_paths_to_clipboard(vec![]);
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
    let result = copy_file_paths_to_clipboard(temp_files.clone());

    // コピー成功を確認
    assert!(result.is_ok(), "Failed to copy files: {:?}", result);

    // ここではクリップボードの内容を自動的に検証することは難しいため、
    // 成功したことだけを確認する

    // テスト後にファイルを削除
    for path in temp_files {
      let _ = std::fs::remove_file(path); // エラーは無視
    }
  }
}
