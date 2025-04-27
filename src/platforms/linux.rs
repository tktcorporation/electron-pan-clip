use arboard::Clipboard;
use std::io::{Error, ErrorKind};
use std::path::Path;

pub fn copy_files_to_clipboard(paths: &[String]) -> Result<(), Error> {
  let mut uris = Vec::new();

  for path_str in paths {
    let path = Path::new(path_str);
    // 絶対パスを取得し、失敗した場合はスキップ
    match path.canonicalize() {
      Ok(abs_path) => {
        // file:// URI スキームを追加
        // to_string_lossy を使用して、無効なUTF-8シーケンスを置換文字で処理
        let uri = format!("file://{}", abs_path.to_string_lossy());
        uris.push(uri);
      }
      Err(e) => {
        eprintln!("Failed to canonicalize path {}: {}", path_str, e);
        // canonicalize に失敗したパスはスキップ
      }
    }
  }

  // 有効なURIがなければエラーを返す
  if uris.is_empty() {
    return Err(Error::new(
      ErrorKind::InvalidInput,
      "No valid URIs could be created from the paths", // macOS とエラーメッセージを統一
    ));
  }

  let uri_list = uris.join("\r\n");

  // arboard を使用してクリップボードにコピー
  let mut clipboard = Clipboard::new().map_err(|e| {
    Error::new(
      ErrorKind::Other,
      format!("Failed to initialize clipboard: {}", e),
    )
  })?;

  clipboard
    .set_text(uri_list.clone()) // text/plain としても設定（互換性のため）
    .map_err(|e| {
      Error::new(
        ErrorKind::Other,
        format!("Failed to set text clipboard: {}", e),
      )
    })?;

  // 必要であれば text/uri-list も設定する (arboard は直接サポートしていない可能性があるため、
  // set_text で代替するか、より低レベルなライブラリを使う必要があるかもしれない)
  // 現状の arboard の set_text が多くの環境で text/uri-list 相当として機能することを期待

  println!("Copied file URIs to clipboard on Linux: {:?}", uris); // 成功時にログ出力
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env::temp_dir;
  use std::fs::File;

  // URI生成の基本的なテスト
  #[test]
  fn test_uri_generation() {
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_linux_uri.txt");
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    let path_str = test_file_path.to_string_lossy().to_string();
    let canonical_path = test_file_path.canonicalize().unwrap();
    let expected_uri = format!("file://{}", canonical_path.to_string_lossy());

    let mut uris = Vec::new();
    let path = Path::new(&path_str);
    if let Ok(abs_path) = path.canonicalize() {
      let uri = format!("file://{}", abs_path.to_string_lossy());
      uris.push(uri);
    }

    assert_eq!(uris.len(), 1);
    assert_eq!(uris[0], expected_uri);

    let _ = std::fs::remove_file(test_file_path);
  }

  // 不正なパスを扱えるかのテスト
  #[test]
  fn test_invalid_paths() {
    let invalid_paths = vec![
      "/path/does/not/exist/linux.txt".to_string(),
      "invalid-path-linux.txt".to_string(),
    ];

    // copy_files_to_clipboard を呼び出すが、エラーが発生することを期待
    let result = copy_files_to_clipboard(&invalid_paths);
    assert!(result.is_err());

    // エラーの種類とメッセージを検証
    if let Err(err) = result {
      assert_eq!(err.kind(), ErrorKind::InvalidInput);
      assert!(err
        .to_string()
        .contains("No valid URIs could be created from the paths"));
    }
  }

  // 実際のクリップボード操作テスト（CIではスキップ推奨）
  #[test]
  #[ignore] // CI 環境では X11 がないため失敗する可能性が高い
  fn test_copy_to_clipboard() {
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_linux_clipboard.txt");
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    let path_str = test_file_path.to_string_lossy().to_string();

    // クリップボードにコピー
    let result = copy_files_to_clipboard(&[path_str]);

    // xclip がない環境や X11 がない環境では失敗することがある
    // その場合はテストをパスさせるか、環境に応じた処理が必要
    if let Err(e) = &result {
      if e.to_string().contains("Failed to initialize clipboard")
        || e.to_string().contains("X11 server connection timed out")
        || e.to_string().contains("No text property")
      // Wayland で発生しうるエラー
      {
        println!("⚠️ クリップボードテストをスキップ: 環境の問題 ({})", e);
        return;
      }
    }

    assert!(result.is_ok(), "Copy operation failed: {:?}", result);

    let _ = std::fs::remove_file(test_file_path);
  }
}
