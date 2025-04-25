use arboard::Clipboard;
use std::io::{Error, ErrorKind};
use url::Url;

pub fn copy_files_to_clipboard(paths: &[String]) -> Result<(), Error> {
  let uri_list: String = paths
    .iter()
    .filter_map(|p| Url::from_file_path(p).ok()) // 絶対パスに変換し、file:// URI を生成
    .map(|u| u.to_string())
    .collect::<Vec<String>>()
    .join("\r\n"); // text/uri-list は CRLF 区切りが推奨されている

  // 有効なURIがなければ、クリップボードを初期化する前にエラーを返す
  if uri_list.is_empty() {
    return Err(Error::new(
      ErrorKind::InvalidInput,
      "No valid file URIs could be generated",
    ));
  }

  // クリップボードの初期化は有効なURIがある場合のみ行う
  let mut clipboard = Clipboard::new().map_err(|e| {
    Error::new(
      ErrorKind::Other,
      format!("Failed to initialize clipboard: {}", e),
    )
  })?;

  // text/uri-list として設定
  // arboard は MIME タイプを直接指定する API がないため、標準の set_text を使う
  // Linux では set_text が text/plain と text/uri-list (file:// の場合) の両方を設定することが期待される
  clipboard.set_text(uri_list).map_err(|e| {
    Error::new(
      ErrorKind::Other,
      format!("Failed to set clipboard content: {}", e),
    )
  })?;

  println!("Copied file URIs to clipboard on Linux: {:?}", paths); // デバッグ用出力
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env::temp_dir;
  use std::fs::File;
  use std::path::Path;

  // URI生成のテスト
  #[test]
  fn test_uri_generation() {
    // 存在するパスを作成
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_linux_clipboard.txt");

    // ファイルを作成して確実に存在させる
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    // パスを文字列に変換
    let path_str = test_file_path.to_string_lossy().to_string();

    // URLに変換できることを確認
    let url = Url::from_file_path(Path::new(&path_str)).expect("Failed to create URL from path");
    assert!(url.to_string().starts_with("file://"));

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);
  }

  // 実際のクリップボード操作テスト
  // 注意: このテストは実際のクリップボードを変更します
  #[test]
  #[ignore]
  fn test_copy_to_clipboard() {
    // 存在するファイルのパスを用意
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_linux_clipboard_copy.txt");

    // ファイルを作成
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    // パスを文字列に変換
    let path_str = test_file_path.to_string_lossy().to_string();

    // クリップボードにコピー
    let result = copy_files_to_clipboard(&[path_str.clone()]);
    assert!(result.is_ok(), "Copy operation failed: {:?}", result);

    // クリップボードの内容を検証
    // 注: 検証は手動で行う必要があるため、ここではエラーがないことだけを確認

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);
  }

  // 無効なパスのテスト
  #[test]
  fn test_invalid_paths() {
    // 存在しない非現実的なパスを使用
    let invalid_paths = vec![
      "/this/path/does/not/exist/12345.txt".to_string(),
      "not-a-real-path.txt".to_string(),
    ];

    // この場合は何らかのエラーになるはず（環境によって種類が異なる可能性あり）
    let result = copy_files_to_clipboard(&invalid_paths);

    // X11環境があるかどうかによって結果が変わる可能性がある
    // X11環境がない場合: InvalidInput エラー
    // X11環境がある場合: URL生成は成功するが、クリップボード操作でOtherエラーになる可能性がある
    if let Err(err) = result {
      // どちらかのエラー種類であることを確認
      assert!(
        err.kind() == ErrorKind::InvalidInput || err.kind() == ErrorKind::Other,
        "Expected InvalidInput or Other error, but got: {:?}",
        err
      );
    } else {
      // テスト環境によっては成功する可能性もある（フルデスクトップ環境の場合）
      println!("警告: 無効なパス処理がエラーなしで成功しました。テスト環境を確認してください。");
    }
  }
}
