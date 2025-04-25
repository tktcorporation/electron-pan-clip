#![cfg(target_os = "macos")]

use cocoa::appkit::NSPasteboard;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSString};
use objc::{class, msg_send, sel, sel_impl};
use std::io::{Error, ErrorKind};
use std::path::Path;

pub fn copy_files_to_clipboard(paths: &[String]) -> Result<(), Error> {
  // NSPasteboard の取得
  unsafe {
    let pasteboard = NSPasteboard::generalPasteboard(nil);

    // 既存のデータをクリア
    pasteboard.clearContents();

    // ファイルの NSURL オブジェクトの配列を作成
    let mut urls = Vec::new();

    for path in paths {
      // 絶対パスに変換（相対パスの場合は失敗する可能性があるため）
      let abs_path = match Path::new(path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
          eprintln!("Failed to canonicalize path {}: {}", path, e);
          continue;
        }
      };

      let path_str = match abs_path.to_str() {
        Some(s) => s,
        None => {
          eprintln!("Path contains invalid UTF-8: {:?}", abs_path);
          continue;
        }
      };

      // NSString としてパスを作成
      let ns_string = NSString::alloc(nil).init_str(path_str);

      // NSURL を直接作成する
      #[allow(unexpected_cfgs)]
      let url_class = class!(NSURL);

      #[allow(unexpected_cfgs)]
      let nsurl: id = msg_send![url_class, fileURLWithPath:ns_string];

      if nsurl != nil {
        urls.push(nsurl);
      } else {
        eprintln!("Failed to create NSURL for path: {}", path_str);
      }
    }

    if urls.is_empty() {
      return Err(Error::new(
        ErrorKind::InvalidInput,
        "No valid URLs could be created from the paths",
      ));
    }

    // NSArray にURLを追加
    let urls_array = NSArray::arrayWithObjects(nil, &urls);

    // クリップボードにファイルURLの配列を書き込み
    let success_i8: i8 = pasteboard.writeObjects(urls_array);
    let success = success_i8 != 0;

    if success {
      println!("Copied files to clipboard on macOS: {:?}", paths);
      Ok(())
    } else {
      Err(Error::new(
        ErrorKind::Other,
        "Failed to write file URLs to pasteboard",
      ))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env::temp_dir;
  use std::fs::File;
  use std::path::PathBuf;

  // パス正規化のテスト
  #[test]
  fn test_path_canonicalization() {
    // 一時ファイルを作成
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_macos_canonical.txt");

    // ファイルを作成
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    // 相対パスと絶対パスで同じファイルを指定
    let absolute_path = test_file_path.to_string_lossy().to_string();

    // 現在のディレクトリから相対パスを作成
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let relative_path = if let Ok(rel) = test_file_path.strip_prefix(&current_dir) {
      rel.to_string_lossy().to_string()
    } else {
      // 相対パスの生成に失敗した場合はスキップ
      println!("Skipping relative path test as temp dir is not under current dir");
      test_file_path.to_string_lossy().to_string()
    };

    // 両方のパスが同じファイルを指すことを確認（絶対パスに変換して比較）
    let abs_path1 = PathBuf::from(&absolute_path)
      .canonicalize()
      .expect("Failed to canonicalize absolute path");

    // 相対パスもcanonicalizeできることを確認
    if Path::new(&relative_path).exists() {
      let abs_path2 = PathBuf::from(&relative_path)
        .canonicalize()
        .expect("Failed to canonicalize relative path");

      assert_eq!(
        abs_path1, abs_path2,
        "Canonicalized paths should be identical"
      );
    }

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);
  }

  // 実際のクリップボード操作テスト
  #[test]
  #[ignore]
  fn test_copy_to_clipboard() {
    // 一時ファイルを作成
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_macos_clipboard.txt");

    // ファイルを作成
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    // パスを文字列に変換
    let path_str = test_file_path.to_string_lossy().to_string();

    // クリップボードにコピー
    let result = copy_files_to_clipboard(&[path_str.clone()]);
    assert!(result.is_ok(), "Copy operation failed: {:?}", result);

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);
  }

  // 不正なパスのテスト
  #[test]
  fn test_invalid_paths() {
    // 存在しないパスを使用
    let invalid_paths = vec![
      "/path/does/not/exist/foo.txt".to_string(),
      "invalid-path-that-doesnt-exist.txt".to_string(),
    ];

    // コピーを試みる - エラーになるはず
    let result = copy_files_to_clipboard(&invalid_paths);
    assert!(result.is_err());

    // エラーメッセージをチェック
    if let Err(err) = result {
      assert!(err.kind() == ErrorKind::InvalidInput);
      assert!(err.to_string().contains("No valid URLs"));
    }
  }
}
