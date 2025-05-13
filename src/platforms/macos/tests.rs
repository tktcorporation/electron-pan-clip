#![cfg(target_os = "macos")]

// パス正規化のテスト
#[test]
fn test_path_canonicalization() {
  // 一時ファイルを作成
  use std::env::temp_dir;
  use std::fs::File;
  use std::path::Path;

  let tmp_dir = temp_dir();
  let test_file_path = tmp_dir.join("test_macos_canonical.txt");

  File::create(&test_file_path).expect("Failed to create test file");

  // 絶対パスに変換
  let absolute_path = test_file_path.to_string_lossy().to_string();
  let abs_result = Path::new(&absolute_path)
    .canonicalize()
    .expect("Failed to canonicalize absolute path");

  assert!(abs_result.exists(), "Canonicalized path should exist");

  std::fs::remove_file(test_file_path).expect("Failed to remove test file");
}

// ObjcUrlのテスト
#[test]
fn test_objc_url() {
  use crate::platforms::macos::wrapper::{AutoreleasePool, ObjcString, ObjcUrl};
  use std::env::temp_dir;
  use std::fs::File;

  // テスト前の準備: 実際のファイルを作成
  let tmp_dir = temp_dir();
  let test_file_path = tmp_dir.join("test_objc_url.txt");
  File::create(&test_file_path).expect("Failed to create test file");

  // テストに使用する絶対パス
  let test_path = test_file_path.to_string_lossy().to_string();

  // AutoreleasePoolを作成
  let _pool = AutoreleasePool::new().expect("Failed to create autorelease pool");

  // NSString経由でファイルパス文字列を作成
  let path_string = ObjcString::from_str(&test_path).expect("Failed to create ObjcString");

  // ObjcUrl::from_file_path経由でURLを作成
  let url =
    ObjcUrl::from_file_path(&path_string).expect("Failed to create ObjcUrl via from_file_path");
  assert!(url.is_file_url(), "URL should be a file URL");

  // パスを取得して確認
  let retrieved_path = url.get_path().expect("Failed to get path from URL");
  assert!(
    !retrieved_path.is_empty(),
    "Retrieved path should not be empty"
  );

  // ObjcUrl::from_path経由でURLを作成
  let url2 = ObjcUrl::from_path(&test_path).expect("Failed to create ObjcUrl via from_path");
  assert!(url2.is_file_url(), "URL should be a file URL");

  // テスト終了後にファイルを削除
  std::fs::remove_file(test_file_path).expect("Failed to remove test file");
}

// テキスト読み取りのテスト (実際のクリップボードを使用)
#[test]
#[ignore] // 通常は無視されるテスト
fn test_read_clipboard_text() {
  use crate::platforms::macos::api::read_clipboard_text;

  // 実際にテキストを読み取り
  let result = read_clipboard_text();

  match result {
    Ok(text) => println!("Read text from clipboard: {}", text),
    Err(e) => println!("Failed to read text from clipboard: {:?}", e),
  }
}

// RAWデータ読み取りのテスト (実際のクリップボードを使用)
#[test]
#[ignore] // 通常は無視されるテスト
fn test_read_clipboard_raw() {
  use crate::platforms::macos::api::read_clipboard_raw;

  // 実際にRAWデータを読み取り
  let result = read_clipboard_raw();

  match result {
    Ok(data) => println!("Read {} bytes of raw data from clipboard", data.len()),
    Err(e) => println!("Failed to read raw data from clipboard: {:?}", e),
  }
}

// ファイルパス読み取りのテスト (実際のクリップボードを使用)
#[test]
#[ignore] // 通常は無視されるテスト
fn test_read_clipboard_file_paths() {
  use crate::platforms::macos::api::{read_clipboard_file_paths, write_clipboard_file_paths};
  use std::env::temp_dir;
  use std::fs::File;

  // 一時ファイルを作成
  let tmp_dir = temp_dir();
  let test_file_path = tmp_dir.join("test_macos_read_paths.txt");

  File::create(&test_file_path).expect("Failed to create test file");

  // ファイルパスをクリップボードにコピー
  let path_str = test_file_path.to_string_lossy().to_string();
  let copy_result = write_clipboard_file_paths(&[path_str]);

  match copy_result {
    Ok(_) => {
      // ファイルパスを読み取り
      let result = read_clipboard_file_paths();

      match result {
        Ok(paths) => {
          println!("Read file paths from clipboard: {:?}", paths);
          assert!(!paths.is_empty(), "Should have read at least one file path");
        }
        Err(e) => println!("Failed to read file paths from clipboard: {:?}", e),
      }
    }
    Err(e) => println!("Failed to copy file path to clipboard: {:?}", e),
  }

  std::fs::remove_file(test_file_path).expect("Failed to remove test file");
}
