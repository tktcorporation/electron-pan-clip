#![cfg(target_os = "macos")]

use cocoa::appkit::NSPasteboard;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSString, NSUInteger};
use objc::{class, msg_send, sel, sel_impl};
use std::io::{Error, ErrorKind};
use std::path::Path;

// クリップボード操作のトレイトを定義
#[cfg(test)]
pub trait ClipboardOperations {
  fn copy_files(&self, urls: Vec<id>) -> Result<(), Error>;
}

// 実際のクリップボード実装
#[cfg(test)]
struct RealClipboard;

#[cfg(test)]
impl ClipboardOperations for RealClipboard {
  fn copy_files(&self, urls: Vec<id>) -> Result<(), Error> {
    unsafe {
      let pool: id = msg_send![class!(NSAutoreleasePool), new];
      let pasteboard = NSPasteboard::generalPasteboard(nil);
      pasteboard.clearContents();

      // 型の宣言と配列の作成は実際の実装と同じ
      let file_urls_type = NSString::alloc(nil).init_str("public.file-url");
      let files_type = NSString::alloc(nil).init_str("NSFilenamesPboardType");
      let types = vec![file_urls_type, files_type];
      let types_array: id = msg_send![
          class!(NSArray),
          arrayWithObjects:types.as_ptr()
          count:types.len() as NSUInteger
      ];

      let _: () = msg_send![pasteboard, declareTypes:types_array owner:nil];

      let urls_array: id = msg_send![
          class!(NSArray),
          arrayWithObjects:urls.as_ptr()
          count:urls.len() as NSUInteger
      ];

      let success: i8 = msg_send![pasteboard, writeObjects:urls_array];
      let () = msg_send![pool, drain];

      if success != 0 {
        Ok(())
      } else {
        Err(Error::new(
          ErrorKind::Other,
          "Failed to write file URLs to pasteboard",
        ))
      }
    }
  }
}

// モッククリップボード実装
#[cfg(test)]
pub struct MockClipboard {
  pub should_succeed: bool,
  pub copied_files: std::cell::RefCell<Vec<String>>,
}

#[cfg(test)]
impl ClipboardOperations for MockClipboard {
  fn copy_files(&self, _urls: Vec<id>) -> Result<(), Error> {
    if self.should_succeed {
      Ok(())
    } else {
      Err(Error::new(ErrorKind::Other, "Mock clipboard failure"))
    }
  }
}

// クリップボード実装を注入できるように関数を修正
#[cfg(test)]
fn copy_files_to_clipboard_with_impl(
  paths: &[String],
  clipboard: &dyn ClipboardOperations,
) -> Result<(), Error> {
  unsafe {
    let pool: id = msg_send![class!(NSAutoreleasePool), new];

    let mut urls = Vec::new();

    for path in paths {
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

      let ns_string = NSString::alloc(nil).init_str(path_str);
      let url_class = class!(NSURL);
      let nsurl: id = msg_send![url_class, fileURLWithPath:ns_string];

      if nsurl != nil {
        urls.push(nsurl);
      } else {
        eprintln!("Failed to create NSURL for path: {}", path_str);
      }
    }

    if urls.is_empty() {
      let () = msg_send![pool, drain];
      return Err(Error::new(
        ErrorKind::InvalidInput,
        "No valid URIs could be created from the paths",
      ));
    }

    let result = clipboard.copy_files(urls);
    let () = msg_send![pool, drain];

    result
  }
}

// 従来の関数をそのまま維持
pub fn copy_files_to_clipboard(paths: &[String]) -> Result<(), Error> {
  // NSPasteboard の取得
  unsafe {
    // AutoreleasePool を作成
    let pool: id = msg_send![class!(NSAutoreleasePool), new];

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
      // プールをドレインしてから戻る
      let () = msg_send![pool, drain];
      return Err(Error::new(
        ErrorKind::InvalidInput,
        "No valid URIs could be created from the paths",
      ));
    }

    // NSPasteboard用のタイプを宣言
    let file_urls_type = NSString::alloc(nil).init_str("public.file-url");
    let files_type = NSString::alloc(nil).init_str("NSFilenamesPboardType");

    // 用意するタイプの配列を作成
    let types = vec![file_urls_type, files_type];
    let types_array: id = msg_send![
      class!(NSArray),
      arrayWithObjects:types.as_ptr()
      count:types.len() as NSUInteger
    ];

    // クリップボードに対してタイプを宣言
    let _: () = msg_send![pasteboard, declareTypes:types_array owner:nil];

    // NSArray にURLを追加（正しい方法で配列を生成）
    let urls_array: id = msg_send![
      class!(NSArray),
      arrayWithObjects:urls.as_ptr()
      count:urls.len() as NSUInteger
    ];

    // クリップボードにファイルURLの配列を書き込み
    let success: i8 = msg_send![pasteboard, writeObjects:urls_array];

    // AutoreleasePool をドレイン
    let () = msg_send![pool, drain];

    if success != 0 {
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

  // 実際のクリップボード操作テスト - 実際のクリップボードを使う場合は無視
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

  // モックを使ったテスト - 新規追加
  #[test]
  fn test_copy_to_clipboard_with_mock() {
    // モックの準備
    let mock_clipboard = MockClipboard {
      should_succeed: true,
      copied_files: std::cell::RefCell::new(Vec::new()),
    };

    // 一時ファイルを作成
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_macos_clipboard_mock.txt");
    let _ = File::create(&test_file_path).expect("Failed to create test file");
    let path_str = test_file_path.to_string_lossy().to_string();

    // モックを使ってテスト
    let result = copy_files_to_clipboard_with_impl(&[path_str.clone()], &mock_clipboard);

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);

    // 結果の検証
    assert!(
      result.is_ok(),
      "Copy operation failed with mock: {:?}",
      result
    );
  }

  // 失敗するモックを使ったテスト - 新規追加
  #[test]
  fn test_mock_clipboard_failure() {
    // 失敗するモックの準備
    let failing_mock = MockClipboard {
      should_succeed: false,
      copied_files: std::cell::RefCell::new(Vec::new()),
    };

    // 一時ファイルを作成
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_macos_clipboard_mock_fail.txt");
    let _ = File::create(&test_file_path).expect("Failed to create test file");
    let path_str = test_file_path.to_string_lossy().to_string();

    // モックを使ってテスト
    let result = copy_files_to_clipboard_with_impl(&[path_str.clone()], &failing_mock);

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);

    // 結果の検証 - 失敗することを期待
    assert!(result.is_err());
    if let Err(e) = result {
      assert_eq!(e.to_string(), "Mock clipboard failure");
    }
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
      assert!(err.to_string().contains("No valid URIs"));
    }
  }
}
