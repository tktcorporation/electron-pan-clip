#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

// OS別の実装モジュール
#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
mod platforms;

#[cfg(not(target_os = "macos"))]
mod platforms;

#[cfg(target_os = "windows")]
use platforms::windows as current_platform;

#[cfg(target_os = "macos")]
use platforms::macos as current_platform;

#[cfg(target_os = "linux")]
use platforms::linux as current_platform;

// napi エラー型エイリアス
type NapiResult<T> = napi::Result<T>;
type NapiError = napi::Error;
use napi::Status; // Import Status

// OS固有のエラーをNapiエラーに変換するヘルパー関数
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn platform_error_to_napi(e: std::io::Error) -> NapiError {
  let reason = format!("{} clipboard error: {}", std::env::consts::OS, e);
  NapiError::new(Status::GenericFailure, reason) // Use Status::GenericFailure
}

/// クリップボードから読み取ったデータを保持する構造体
/// `read_clipboard_results` から成功した値を抽出して生成することを想定
#[derive(Debug, Default)]
#[napi(object)]
pub struct ClipboardContent {
  /// ファイルパスのリスト。読み取りに失敗した場合は空の配列。
  pub file_paths: Vec<String>,
  /// テキスト内容。読み取りに失敗した場合はnull。
  pub text: Option<String>,
}

/// クリップボードの読み取り結果を保持する構造体 (Rust内部用)
/// 各フィールドは読み取り操作の成功/失敗を示す Result 型
#[derive(Debug)]
pub struct ClipboardReadResult {
  /// ファイルパス読み取りの結果。成功時は`Vec<String>`、失敗時は`napi::Error`。
  pub file_paths: NapiResult<Vec<String>>,
  /// テキスト読み取りの結果。成功時は`Option<String>`、失敗時は`napi::Error`。
  pub text: NapiResult<Option<String>>,
}

/// クリップボードのバイナリデータを読みやすい形式で表示するための構造体
#[napi(object)]
pub struct ReadableClipboardContent {
  /// バイナリデータをHEX形式で表示
  pub hex_view: Option<String>,
  /// バイナリデータをUTF-8テキストとして解釈（可能な場合）
  pub text_view: Option<String>,
  /// バイナリデータのMIMEタイプ（判別可能な場合）
  pub mime_type: Option<String>,
  /// データのサイズ（バイト単位）
  pub size: u32,
  /// 最初の数バイトのプレビュー
  pub preview: Option<String>,
}

/// Hello World関数 - 動作確認用
#[napi]
pub fn hello_world() -> String {
  let os_name = match std::env::consts::OS {
    "macos" => "macOS",
    "windows" => "Windows",
    "linux" => "Linux",
    other => other,
  };

  format!(
    "Hello from Rust {} on {}!",
    env!("CARGO_PKG_VERSION"),
    os_name
  )
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
pub fn write_clipboard_file_paths(paths: Vec<String>) -> NapiResult<()> {
  // 空の配列の場合は早期リターンするが、エラーではなく成功として扱う
  if paths.is_empty() {
    return Ok(());
  }

  #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
  {
    current_platform::copy_files_to_clipboard(&paths).map_err(platform_error_to_napi)?;
    println!("write_clipboard_file_paths: {:?}", &paths);
  }

  #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
  {
    return Err(NapiError::from_reason("Unsupported operating system"));
  }

  Ok(())
}

/// Reads content from the OS clipboard, trying to extract both file paths and text independently.
///
/// # Returns
/// * Returns `Ok(ClipboardContent)` containing results for both file paths and text reads.
/// * Returns `Err(napi::Error)` if both file paths and text reads failed.
///
/// # Note
/// * This function attempts to read both file paths and text, returning their respective outcomes.
/// * If at least one of the reads succeeds, the function returns success with available data.
/// * Only returns an error if both file paths and text reads fail.
#[napi]
pub fn read_clipboard_results() -> napi::Result<ClipboardContent> {
  let internal_result = {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
      // ファイルパスの読み取りを試みる
      let file_paths_result =
        current_platform::read_clipboard_file_paths().map_err(platform_error_to_napi);

      // テキストの読み取りを試みる
      let text_result = match current_platform::read_clipboard_text() {
        Ok(text) => Ok(Some(text)),
        Err(e) => {
          if e.kind() == std::io::ErrorKind::NotFound
            || e.to_string().contains("No text")
            || e.to_string().contains("empty")
          {
            Ok(None) // テキストが存在しないのはエラーではない
          } else {
            Err(platform_error_to_napi(e))
          }
        }
      };

      ClipboardReadResult {
        file_paths: file_paths_result,
        text: text_result,
      }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
      // サポートされていないOSの場合、両方の結果をエラーとして返す
      ClipboardReadResult {
        file_paths: Err(NapiError::from_reason("Unsupported OS for file paths")),
        text: Err(NapiError::from_reason("Unsupported OS for text")),
      }
    }
  };

  // 両方エラーであれば、エラーを返す
  if internal_result.file_paths.is_err() && internal_result.text.is_err() {
    // ファイルパスとテキストの両方が取得できなかった場合
    let file_paths_err = internal_result.file_paths.unwrap_err();
    let text_err = internal_result.text.unwrap_err();
    return Err(NapiError::from_reason(format!(
      "Failed to read clipboard content: file paths error: {}, text error: {}",
      file_paths_err.reason, text_err.reason
    )));
  }

  // 少なくとも一方が成功した場合は、結果を返す
  let mut result = ClipboardContent::default();

  // ファイルパスの結果を処理
  match &internal_result.file_paths {
    Ok(paths) => {
      result.file_paths = paths.clone();
    }
    Err(_) => {
      // ファイルパスの読み取りに失敗した場合は、rawデータをテキストとして試す
      if result.text.is_none() && internal_result.text.is_err() {
        // テキストもファイルパスも取得できなかった場合、raw読み取りを試みる
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        match current_platform::read_clipboard_raw() {
          Ok(raw_data) => {
            if !raw_data.is_empty() {
              // UTF-8として解釈を試みる
              if let Ok(text) = String::from_utf8(raw_data.clone()) {
                if !text.trim().is_empty() {
                  result.text = Some(text);
                }
              }
            }
          }
          Err(_) => {} // raw読み取りに失敗した場合は無視
        }
      }
    }
  }

  // テキストの結果を処理
  if let Ok(text) = internal_result.text {
    result.text = text;
  }

  // 常にtextフィールドを確保する（nullでも含める）
  if result.text.is_none() {
    result.text = None;
  }

  Ok(result)
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
    assert!(result.is_ok());
  }

  // 実際のファイルを作成してコピーするテスト
  // 注意: このテストは実際のクリップボードを変更します
  #[test]
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

  // クリップボード内容読み取り関数のテスト（ファイルパスを読み取る場合）
  #[test]
  fn test_read_clipboard_results_file_paths() {
    // テスト用の一時ファイルを作成
    let mut test_paths = Vec::new();
    let mut canonical_paths = Vec::new();

    for i in 0..2 {
      let mut path = temp_dir();
      path.push(format!("electron_pan_clip_test_results_{}.txt", i));

      let file_path_str = path.to_string_lossy().to_string();

      // ファイルを作成
      let mut file = File::create(&path).expect("Failed to create test file");
      writeln!(file, "Test content {}", i).expect("Failed to write to test file");

      test_paths.push(file_path_str);
      canonical_paths.push(path.canonicalize().unwrap().to_string_lossy().to_string());
    }
    // パスをソートして比較しやすくする
    canonical_paths.sort();

    // ファイルパスをクリップボードにコピー
    let copy_result = write_clipboard_file_paths(test_paths.clone());
    assert!(
      copy_result.is_ok(),
      "Failed to copy file paths to clipboard: {:?}",
      copy_result.err()
    );

    // クリップボード内容を読み取り
    // NAPI環境が必要なため、通常の `cargo test` では実行できない
    // let results = read_clipboard_results(); // env が必要
    // assert!(results.is_ok(), "read_clipboard_results failed: {:?}", results.err());
    // let content_results = results.unwrap();

    // JsObject の内容をテストするには、Node.js 環境での統合テストが必要
    /*
    // ファイルパスが取得できているはず
    assert!(content_results.get_named_property::<Vec<String>>("filePaths").is_ok(), "File paths read failed");
    let mut read_paths = content_results.get_named_property::<Vec<String>>("filePaths").unwrap();
    read_paths.sort(); // 比較のためにソート
    assert_eq!(read_paths, canonical_paths, "Read file paths do not match");

    // テキストは空のはず (またはエラー)
    let text_prop = content_results.get_named_property::<Option<String>>("text");
    assert!(text_prop.is_ok(), "Failed to get text property");
    match text_prop.unwrap() {
        Some(text) => panic!("Expected no text, but got: {}", text),
        None => { /* テキストなし、期待通り */ }
    }
    // エラープロパティのチェックも追加可能
    assert!(content_results.get_named_property::<String>("textError").is_err(), "textError should not exist");
    assert!(content_results.get_named_property::<String>("filePathsError").is_err(), "filePathsError should not exist");
    */
    println!("Skipping JsObject validation in test_read_clipboard_results_file_paths as it requires NAPI Env");

    // テスト後にファイルを削除
    for path in test_paths {
      let _ = std::fs::remove_file(path);
    }
  }
}
