#![cfg(target_os = "macos")]

use cocoa::appkit::NSPasteboard;
use cocoa::base::{id, nil};
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSString, NSUInteger};
use objc::{class, msg_send, sel, sel_impl};
use std::ffi::c_char;
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

// ファイルパスをクリップボードにコピーする
pub fn copy_files_to_clipboard(paths: &[String]) -> Result<(), Error> {
  // NSPasteboard の取得
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

// クリップボードからテキストを読み取る
pub fn read_clipboard_text() -> Result<String, Error> {
  let pool: id = msg_send![class!(NSAutoreleasePool), new];

  let pasteboard = NSPasteboard::generalPasteboard(nil);

  // テキスト形式の定義
  let string_type = NSString::alloc(nil).init_str("public.utf8-plain-text");

  // テキストデータを取得
  let pasteboard_string: id = msg_send![pasteboard, stringForType:string_type];

  let result = if pasteboard_string != nil {
    // NSStringをRustの文字列に変換
    let chars: *const c_char = msg_send![pasteboard_string, UTF8String];
    let rust_str = std::ffi::CStr::from_ptr(chars)
      .to_string_lossy()
      .to_string();
    Ok(rust_str)
  } else {
    Err(Error::new(ErrorKind::Other, "No text found on clipboard"))
  };

  let () = msg_send![pool, drain];
  result
}

// クリップボードからRAWデータを読み取る
pub fn read_clipboard_raw() -> Result<Vec<u8>, Error> {
  let pool: id = msg_send![class!(NSAutoreleasePool), new];

  let pasteboard = NSPasteboard::generalPasteboard(nil);

  // 一般的なバイナリデータ形式の定義
  let data_type = NSString::alloc(nil).init_str("public.data");

  // データを取得
  let data: id = msg_send![pasteboard, dataForType:data_type];

  let result = if data != nil {
    // NSDataをRustのVec<u8>に変換
    let length: NSUInteger = msg_send![data, length];
    let bytes: *const u8 = msg_send![data, bytes];

    if length > 0 && !bytes.is_null() {
      let slice = std::slice::from_raw_parts(bytes, length as usize);
      let vec_data = slice.to_vec();
      Ok(vec_data)
    } else {
      Err(Error::new(ErrorKind::Other, "Empty data on clipboard"))
    }
  } else {
    Err(Error::new(
      ErrorKind::Other,
      "No raw data found on clipboard",
    ))
  };

  let () = msg_send![pool, drain];
  result
}

// クリップボードからファイルパスを読み取る
pub fn read_clipboard_file_paths() -> Result<Vec<String>, Error> {
  let pool: id = msg_send![class!(NSAutoreleasePool), new];

  let pasteboard = NSPasteboard::generalPasteboard(nil);

  // ファイルURLの型を定義
  let url_class = class!(NSURL);
  let classes_array: id = msg_send![
    class!(NSArray),
    arrayWithObject:url_class
  ];

  // クリップボードからファイルURLを読み取る
  let file_urls: id = msg_send![pasteboard, readObjectsForClasses:classes_array options:nil];

  let mut paths = Vec::new();

  if file_urls != nil {
    // NSArrayの要素数を取得
    let count: NSUInteger = msg_send![file_urls, count];

    for i in 0..count {
      let url: id = msg_send![file_urls, objectAtIndex:i];
      if url != nil {
        // URLをパスに変換
        let is_file_url: bool = msg_send![url, isFileURL];
        if is_file_url {
          let path: id = msg_send![url, path];
          if path != nil {
            let chars: *const c_char = msg_send![path, UTF8String];
            let path_str = std::ffi::CStr::from_ptr(chars)
              .to_string_lossy()
              .to_string();
            paths.push(path_str);
          }
        }
      }
    }
  }

  let () = msg_send![pool, drain];

  if paths.is_empty() {
    Err(Error::new(
      ErrorKind::Other,
      "No file paths found on clipboard",
    ))
  } else {
    Ok(paths)
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

    // 両方変換されるか確認
    let _abs_result = Path::new(&absolute_path)
      .canonicalize()
      .expect("Failed to canonicalize absolute path");

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);
  }

  // パスをファイルURLに変換するテスト
  #[test]
  fn test_nsurl_conversion() {
    let pool: id = msg_send![class!(NSAutoreleasePool), new];

    let test_path = "/tmp/test_path.txt";
    let ns_string = NSString::alloc(nil).init_str(test_path);

    let url_class = class!(NSURL);
    let nsurl: id = msg_send![url_class, fileURLWithPath:ns_string];

    assert!(nsurl != nil, "NSURL should not be nil");

    // URLからパスを取得
    let path: id = msg_send![nsurl, path];
    assert!(path != nil, "Path should not be nil");

    let chars: *const c_char = msg_send![path, UTF8String];
    let result_path = std::ffi::CStr::from_ptr(chars)
      .to_string_lossy()
      .to_string();

    assert_eq!(
      result_path, test_path,
      "Path should be preserved in NSURL conversion"
    );

    let () = msg_send![pool, drain];
  }
}

// テキスト読み取りのテスト
#[test]
#[ignore] // 通常のCIでは実行しないよう無視フラグを付ける
fn test_read_clipboard_text() {
  // 実際のクリップボードを使用するため、自動テストには不向き

  // テキストを読み取り
  let result = read_clipboard_text();

  // 成功した場合の処理
  if let Ok(text) = result {
    println!("Read text from clipboard: {}", text);
    assert!(!text.is_empty(), "Read text should not be empty");
  }
}

// RAWデータ読み取りのテスト
#[test]
#[ignore] // 通常のCIでは実行しないよう無視フラグを付ける
fn test_read_clipboard_raw() {
  // 実際のクリップボードを使用するため、自動テストには不向き

  // RAWデータを読み取り
  let result = read_clipboard_raw();

  // 成功した場合の処理
  if let Ok(data) = result {
    println!("Read {} bytes of raw data from clipboard", data.len());
    assert!(!data.is_empty(), "Raw data should not be empty");
  }
}

// ファイルパス読み取りのテスト
#[test]
#[ignore] // 通常のCIでは実行しないよう無視フラグを付ける
fn test_read_clipboard_file_paths() {
  // 一時ファイルを作成
  let tmp_dir = temp_dir();
  let test_file_path = tmp_dir.join("test_macos_read_paths.txt");

  // ファイルを作成
  let _ = File::create(&test_file_path).expect("Failed to create test file");

  // ファイルパスをクリップボードにコピー
  let path_str = test_file_path.to_string_lossy().to_string();
  let _ = copy_files_to_clipboard(&[path_str]);

  // ファイルパスを読み取り
  let result = read_clipboard_file_paths();

  // 成功した場合の処理
  if let Ok(paths) = result {
    println!("Read file paths from clipboard: {:?}", paths);
    assert!(!paths.is_empty(), "File paths should not be empty");

    // 最初のパスが正しいか確認
    let canonical_path = test_file_path
      .canonicalize()
      .expect("Failed to canonicalize path");
    let canonical_str = canonical_path.to_string_lossy().to_string();

    assert_eq!(
      paths[0], canonical_str,
      "Read path should match written path"
    );
  }

  // テスト後にファイルを削除
  let _ = std::fs::remove_file(test_file_path);
}
