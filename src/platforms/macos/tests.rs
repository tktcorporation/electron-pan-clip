#![cfg(target_os = "macos")]

use super::api::*;
use super::wrapper::*;
use std::env::temp_dir;
use std::fs::File;
use std::path::Path;

// パス正規化のテスト
#[test]
fn test_path_canonicalization() {
  // 一時ファイルを作成
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
  let _pool = AutoreleasePool::new().unwrap();

  let test_path = "/tmp/test_path.txt";
  let url = ObjcUrl::from_path(test_path).expect("Failed to create URL");

  let path = url.get_path().expect("Failed to get path from URL");
  assert_eq!(path, test_path, "Path should match original");

  assert!(url.is_file_url(), "Should be a file URL");
}

// テキスト読み取りのテスト (実際のクリップボードを使用)
#[test]
#[ignore] // 通常は無視されるテスト
fn test_read_clipboard_text() {
  // 実際にテキストを読み取り
  let result = read_clipboard_text();

  if let Ok(text) = result {
    println!("Read text from clipboard: {}", text);
  } else {
    println!("Failed to read text from clipboard: {:?}", result.err());
  }
}

// RAWデータ読み取りのテスト (実際のクリップボードを使用)
#[test]
#[ignore] // 通常は無視されるテスト
fn test_read_clipboard_raw() {
  // 実際にRAWデータを読み取り
  let result = read_clipboard_raw();

  if let Ok(data) = result {
    println!("Read {} bytes of raw data from clipboard", data.len());
  } else {
    println!("Failed to read raw data from clipboard: {:?}", result.err());
  }
}

// ファイルパス読み取りのテスト (実際のクリップボードを使用)
#[test]
#[ignore] // 通常は無視されるテスト
fn test_read_clipboard_file_paths() {
  // 一時ファイルを作成
  let tmp_dir = temp_dir();
  let test_file_path = tmp_dir.join("test_macos_read_paths.txt");

  File::create(&test_file_path).expect("Failed to create test file");

  // ファイルパスをクリップボードにコピー
  let path_str = test_file_path.to_string_lossy().to_string();
  let copy_result = copy_files_to_clipboard(&[path_str]);
  assert!(copy_result.is_ok(), "Failed to copy file path to clipboard");

  // ファイルパスを読み取り
  let result = read_clipboard_file_paths();

  if let Ok(paths) = result {
    println!("Read file paths from clipboard: {:?}", paths);
    assert!(!paths.is_empty(), "Should have read at least one file path");
  } else {
    println!(
      "Failed to read file paths from clipboard: {:?}",
      result.err()
    );
  }

  std::fs::remove_file(test_file_path).expect("Failed to remove test file");
}
