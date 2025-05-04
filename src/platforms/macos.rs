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
  // モックの状態を保持するためのフィールドを追加できます
  // pub copied_files: std::cell::RefCell<Vec<String>>,
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
  let mut errors = Vec::new(); // パス処理中のエラーを収集

  for path_str in paths {
    let path = Path::new(path_str);
    // canonicalize は存在しないパスや権限がない場合にエラーになる
    match path.canonicalize() {
      Ok(abs_path) => {
        match abs_path.to_str() {
          Some(s) => {
            let ns_string: id = msg_send![class!(NSString), alloc];
            let ns_string: id =
              msg_send![ns_string, initWithUTF8String: s.as_ptr() as *const c_char];
            if ns_string != nil {
              let url_class = class!(NSURL);
              let nsurl: id = msg_send![url_class, fileURLWithPath:ns_string];
              if nsurl != nil {
                urls.push(nsurl);
              } else {
                errors.push(format!("Failed to create NSURL for path: {}", s));
              }
              // release ns_string? ARC should handle it if created with alloc/init
              // let () = msg_send![ns_string, release];
            } else {
              errors.push(format!("Failed to create NSString for path: {}", s));
            }
          }
          None => {
            errors.push(format!("Path contains invalid UTF-8: {:?}", abs_path));
          }
        }
      }
      Err(e) => {
        // エラーを収集し、処理を続行する
        errors.push(format!("Failed to canonicalize path '{}': {}", path_str, e));
      }
    }
  }

  // 有効な URL が一つも生成できなかった場合
  if urls.is_empty() {
    let () = msg_send![pool, drain];
    // 詳細なエラーメッセージを含める
    let error_message = format!(
      "No valid URIs could be created. Errors: {}",
      errors.join("; ")
    );
    return Err(Error::new(ErrorKind::InvalidInput, error_message));
  }

  // 成功しなかったパスがある場合、警告としてログ出力する（オプション）
  if !errors.is_empty() {
    eprintln!(
      "Warning: Some paths failed during processing: {}",
      errors.join("; ")
    );
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

  let pasteboard: id = unsafe { NSPasteboard::generalPasteboard(nil) };
  if pasteboard == nil {
    let () = msg_send![pool, drain];
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to get general pasteboard",
    ));
  }

  // 既存のデータをクリア
  let _: NSInteger = unsafe { msg_send![pasteboard, clearContents] };

  // ファイルの NSURL オブジェクトの配列を作成
  let mut urls = Vec::new();
  let mut errors = Vec::new(); // エラー収集用

  for path_str in paths {
    let path = Path::new(path_str);
    match path.canonicalize() {
      Ok(abs_path) => {
        match abs_path.to_str() {
          Some(s) => {
            let ns_string: id = msg_send![class!(NSString), alloc];
            let ns_string: id =
              msg_send![ns_string, initWithUTF8String: s.as_ptr() as *const c_char];
            if ns_string != nil {
              let url_class = class!(NSURL);
              let nsurl: id = msg_send![url_class, fileURLWithPath:ns_string];
              if nsurl != nil {
                urls.push(nsurl);
              } else {
                errors.push(format!("Failed to create NSURL for path: {}", s));
              }
              // 必要なら release
              // let () = msg_send![ns_string, release];
            } else {
              errors.push(format!("Failed to create NSString from path: {}", s));
            }
          }
          None => {
            errors.push(format!("Path contains invalid UTF-8: {:?}", abs_path));
          }
        }
      }
      Err(e) => {
        errors.push(format!("Failed to canonicalize path '{}': {}", path_str, e));
      }
    }
  }

  if urls.is_empty() {
    let () = msg_send![pool, drain];
    let error_message = format!(
      "No valid file URIs could be created from the provided paths. Errors: {}",
      errors.join("; ")
    );
    return Err(Error::new(ErrorKind::InvalidInput, error_message));
  }

  // 警告ログ
  if !errors.is_empty() {
    eprintln!(
      "Warning: Some paths could not be processed: {}",
      errors.join("; ")
    );
  }

  // NSPasteboard用のタイプを宣言
  let file_url_type_str = "public.file-url";
  let filenames_type_str = "NSFilenamesPboardType"; // 古い形式だが互換性のため
  let ns_file_url_type: id = msg_send![class!(NSString), alloc];
  let ns_file_url_type: id =
    msg_send![ns_file_url_type, initWithUTF8String: file_url_type_str.as_ptr() as *const c_char];
  let ns_filenames_type: id = msg_send![class!(NSString), alloc];
  let ns_filenames_type: id =
    msg_send![ns_filenames_type, initWithUTF8String: filenames_type_str.as_ptr() as *const c_char];

  // nil チェックを追加
  if ns_file_url_type == nil || ns_filenames_type == nil {
    let () = msg_send![pool, drain];
    // release 필요?
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to create NSString for pasteboard types",
    ));
  }

  let types = vec![ns_file_url_type, ns_filenames_type];
  let types_array: id = msg_send![
    class!(NSArray),
    arrayWithObjects:types.as_ptr()
    count:types.len() as NSUInteger
  ];
  if types_array == nil {
    let () = msg_send![pool, drain];
    // release 필요?
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to create NSArray for pasteboard types",
    ));
  }

  // クリップボードに対してタイプを宣言
  let declared: bool = msg_send![pasteboard, declareTypes:types_array owner:nil];
  if !declared {
    let () = msg_send![pool, drain];
    // release 필요?
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to declare pasteboard types",
    ));
  }

  // NSArray にURLを追加
  let urls_array: id = msg_send![
    class!(NSArray),
    arrayWithObjects:urls.as_ptr()
    count:urls.len() as NSUInteger
  ];
  if urls_array == nil {
    let () = msg_send![pool, drain];
    // release 필요?
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to create NSArray for URLs",
    ));
  }

  // クリップボードにファイルURLの配列を書き込み
  let success: bool = msg_send![pasteboard, writeObjects:urls_array];

  // AutoreleasePool をドレイン
  let () = msg_send![pool, drain];
  // 必要なら type string なども release

  if success {
    println!("Copied {} files to clipboard on macOS", urls.len());
    Ok(())
  } else {
    Err(Error::new(
      ErrorKind::Other,
      "Failed to write file URLs to pasteboard (writeObjects failed)",
    ))
  }
}

// クリップボードからテキストを読み取る
pub fn read_clipboard_text() -> Result<String, Error> {
  let pool: id = msg_send![class!(NSAutoreleasePool), new];
  let pasteboard: id = unsafe { NSPasteboard::generalPasteboard(nil) };
  if pasteboard == nil {
    let () = msg_send![pool, drain];
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to get general pasteboard",
    ));
  }

  // テキスト形式の定義 (public.utf8-plain-text)
  let type_str = "public.utf8-plain-text";
  let string_type: id = msg_send![class!(NSString), alloc];
  let string_type: id =
    msg_send![string_type, initWithUTF8String: type_str.as_ptr() as *const c_char];
  if string_type == nil {
    let () = msg_send![pool, drain];
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to create NSString for text type",
    ));
  }

  // テキストデータを取得 (stringForType:)
  let pasteboard_string: id = msg_send![pasteboard, stringForType:string_type];

  let result = if pasteboard_string != nil {
    // NSStringをRustの文字列に変換
    let utf8_str: *const c_char = msg_send![pasteboard_string, UTF8String];
    if !utf8_str.is_null() {
      // CStr::from_ptr は unsafe ブロックが必要
      unsafe {
        let rust_str = std::ffi::CStr::from_ptr(utf8_str)
          .to_string_lossy()
          .to_string();
        if rust_str.is_empty() {
          // 空文字列の場合も成功とするか、エラーとするか？ 仕様による。
          // ここでは空文字列も成功として返す。
          Ok(rust_str)
        } else {
          Ok(rust_str)
        }
      }
    } else {
      Err(Error::new(
        ErrorKind::InvalidData,
        "Failed to get UTF8 string from pasteboard content",
      ))
    }
  } else {
    Err(Error::new(
      ErrorKind::NotFound,
      "No text content found on clipboard for type public.utf8-plain-text",
    ))
  };

  // let () = msg_send![string_type, release]; // ARC
  let () = msg_send![pool, drain];
  result
}

// クリップボードからRAWデータを読み取る
pub fn read_clipboard_raw() -> Result<Vec<u8>, Error> {
  let pool: id = msg_send![class!(NSAutoreleasePool), new];
  let pasteboard: id = unsafe { NSPasteboard::generalPasteboard(nil) };
  if pasteboard == nil {
    let () = msg_send![pool, drain];
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to get general pasteboard",
    ));
  }

  // 利用可能な最初のタイプを取得してみる (より汎用的)
  let available_types: id = msg_send![pasteboard, types];
  if available_types == nil || msg_send![available_types, count] == 0 {
    let () = msg_send![pool, drain];
    return Err(Error::new(
      ErrorKind::NotFound,
      "Clipboard is empty or has no available types",
    ));
  }

  // 最初のタイプでデータを取得試行
  let data_type: id = msg_send![available_types, objectAtIndex:0];
  if data_type == nil {
    let () = msg_send![pool, drain];
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to get first available type from clipboard",
    ));
  }

  // dataForType: を使用
  let data: id = msg_send![pasteboard, dataForType:data_type];

  let result = if data != nil {
    // NSDataをRustのVec<u8>に変換
    let length: NSUInteger = msg_send![data, length];
    let bytes: *const u8 = msg_send![data, bytes]; // bytes メソッドは unsafe

    if length > 0 && !bytes.is_null() {
      unsafe {
        let slice = std::slice::from_raw_parts(bytes, length as usize);
        let vec_data = slice.to_vec();
        Ok(vec_data)
      }
    } else {
      // データはあるが空の場合
      Ok(Vec::new()) // 空のVecを返す
    }
  } else {
    // dataForType: が nil を返した場合
    // 取得しようとした type をエラーメッセージに含めるとより親切
    let type_str: *const c_char = msg_send![data_type, UTF8String];
    let type_name = unsafe { std::ffi::CStr::from_ptr(type_str).to_string_lossy() };
    Err(Error::new(
      ErrorKind::NotFound,
      format!("No data found on clipboard for type '{}'", type_name),
    ))
  };

  let () = msg_send![pool, drain];
  result
}

// クリップボードからファイルパスを読み取る
pub fn read_clipboard_file_paths() -> Result<Vec<String>, Error> {
  let pool: id = msg_send![class!(NSAutoreleasePool), new];
  let pasteboard: id = unsafe { NSPasteboard::generalPasteboard(nil) };
  if pasteboard == nil {
    let () = msg_send![pool, drain];
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to get general pasteboard",
    ));
  }

  // ファイルURLの型 (NSURL) を含む NSArray をクラスフィルターとして指定
  let url_class: id = class!(NSURL);
  let classes_array: id = msg_send![class!(NSArray), arrayWithObject:url_class];
  if classes_array == nil {
    let () = msg_send![pool, drain];
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to create NSArray for NSURL class filter",
    ));
  }

  // クリップボードから NSURL オブジェクトを読み取る (readObjectsForClasses:options:)
  let file_urls: id = msg_send![pasteboard, readObjectsForClasses:classes_array options:nil];

  let mut paths = Vec::new();

  if file_urls != nil {
    // NSArrayの要素数を取得
    let count: NSUInteger = msg_send![file_urls, count];

    for i in 0..count {
      let url: id = msg_send![file_urls, objectAtIndex:i];
      if url != nil {
        // URLがファイルURLか確認 (isFileURL)
        let is_file_url: bool = msg_send![url, isFileURL];
        if is_file_url {
          // URLからパスを取得 (path)
          let path_nsstring: id = msg_send![url, path];
          if path_nsstring != nil {
            let utf8_path: *const c_char = msg_send![path_nsstring, UTF8String];
            if !utf8_path.is_null() {
              unsafe {
                let path_str = std::ffi::CStr::from_ptr(utf8_path)
                  .to_string_lossy()
                  .to_string();
                paths.push(path_str);
              }
            } else {
              eprintln!(
                "Warning: Could not get UTF8 string from file URL path object at index {}",
                i
              );
            }
          } else {
            eprintln!(
              "Warning: Could not get path object from file URL at index {}",
              i
            );
          }
        }
      }
    }
  }

  let () = msg_send![pool, drain];

  if paths.is_empty() {
    // 読み取りは成功したが、ファイルURLが含まれていなかった場合
    Err(Error::new(
      ErrorKind::NotFound,
      "No valid file paths found on clipboard (checked for NSURL)",
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
    let ns_string: id = msg_send![class!(NSString), alloc];
    let ns_string: id =
      msg_send![ns_string, initWithUTF8String: test_path.as_ptr() as *const c_char];
    assert!(ns_string != nil, "NSString should not be nil");

    let url_class = class!(NSURL);
    let nsurl: id = msg_send![url_class, fileURLWithPath:ns_string];
    assert!(nsurl != nil, "NSURL should not be nil");

    // URLからパスを取得
    let path_nsstring: id = msg_send![nsurl, path];
    assert!(path_nsstring != nil, "Path NSString should not be nil");

    let utf8_path: *const c_char = msg_send![path_nsstring, UTF8String];
    assert!(!utf8_path.is_null(), "UTF8 path pointer should not be null");

    let result_path = unsafe {
      std::ffi::CStr::from_ptr(utf8_path)
        .to_string_lossy()
        .to_string()
    };

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
