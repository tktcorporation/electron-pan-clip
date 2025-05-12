#![cfg(target_os = "macos")]

use cocoa::base::id;
use objc::{class, msg_send, sel, sel_impl};
use std::io::{Error, ErrorKind};
use std::path::Path;

use super::wrapper::*;

/// クリップボード操作のトレイト定義
#[cfg(test)]
pub trait ClipboardOperations {
  fn copy_files(&self, urls: Vec<id>) -> Result<(), Error>;
}

/// 実際のクリップボード実装
#[cfg(test)]
pub struct RealClipboard;

#[cfg(test)]
impl ClipboardOperations for RealClipboard {
  fn copy_files(&self, urls: Vec<id>) -> Result<(), Error> {
    // AutoreleasePoolを作成
    let _pool = AutoreleasePool::new()?;

    // Pasteboardを取得
    let pasteboard = Pasteboard::general()
      .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to get general pasteboard"))?;

    // 内容をクリア
    pasteboard.clear_contents();

    // タイプを宣言
    let file_url_type = ObjcString::from_str("public.file-url").ok_or_else(|| {
      Error::new(
        ErrorKind::Other,
        "Failed to create NSString for file URL type",
      )
    })?;

    let filenames_type = ObjcString::from_str("NSFilenamesPboardType").ok_or_else(|| {
      Error::new(
        ErrorKind::Other,
        "Failed to create NSString for filenames type",
      )
    })?;

    let types = vec![file_url_type.as_id(), filenames_type.as_id()];
    let types_array = ObjcArray::from_vec(&types)
      .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to create types array"))?;

    pasteboard.declare_types(&types_array);

    // URLの配列を作成
    let urls_array = ObjcArray::from_vec(&urls)
      .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to create URLs array"))?;

    // クリップボードに書き込み
    let success = pasteboard.write_objects(&urls_array);

    if success {
      Ok(())
    } else {
      Err(Error::new(
        ErrorKind::Other,
        "Failed to write file URLs to pasteboard",
      ))
    }
  }
}

/// モッククリップボード実装
#[cfg(test)]
pub struct MockClipboard {
  pub should_succeed: bool,
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

/// ファイルパスをクリップボードにコピーする
pub fn write_clipboard_file_paths(paths: &[String]) -> Result<(), Error> {
  // 入力が空の場合は空の配列を返す
  if paths.is_empty() {
    return Ok(());
  }

  // AutoreleasePoolを作成
  let _pool = AutoreleasePool::new()?;

  // Pasteboardを取得
  let pasteboard = Pasteboard::general()
    .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to get general pasteboard"))?;

  // 内容をクリア
  pasteboard.clear_contents();

  // ファイルURLの配列を作成
  let mut urls = Vec::new();
  let mut errors = Vec::new();

  for path_str in paths {
    let path = Path::new(path_str);
    match path.canonicalize() {
      Ok(abs_path) => {
        if let Some(s) = abs_path.to_str() {
          if let Some(obj_url) = ObjcUrl::from_path(s) {
            urls.push(obj_url.as_id());
          } else {
            errors.push(format!("Failed to create NSURL for path: {}", s));
          }
        } else {
          errors.push(format!("Path contains invalid UTF-8: {:?}", abs_path));
        }
      }
      Err(e) => {
        errors.push(format!("Failed to canonicalize path '{}': {}", path_str, e));
      }
    }
  }

  // 無効なパスが一つでもあればエラー
  if !errors.is_empty() {
    let error_message = format!(
      "Some paths could not be processed: {}",
      errors.join("; ")
    );
    return Err(Error::new(ErrorKind::InvalidInput, error_message));
  }

  // クリップボードのタイプを設定
  let file_url_type = ObjcString::from_str("public.file-url").ok_or_else(|| {
    Error::new(
      ErrorKind::Other,
      "Failed to create NSString for file URL type",
    )
  })?;

  let filenames_type = ObjcString::from_str("NSFilenamesPboardType").ok_or_else(|| {
    Error::new(
      ErrorKind::Other,
      "Failed to create NSString for filenames type",
    )
  })?;

  let types = vec![file_url_type.as_id(), filenames_type.as_id()];
  let types_array = ObjcArray::from_vec(&types)
    .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to create types array"))?;

  // タイプを宣言
  let declared = pasteboard.declare_types(&types_array);
  if !declared {
    return Err(Error::new(
      ErrorKind::Other,
      "Failed to declare pasteboard types",
    ));
  }

  // URLの配列を作成
  let urls_array = ObjcArray::from_vec(&urls)
    .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to create URLs array"))?;

  // クリップボードに書き込み
  let success = pasteboard.write_objects(&urls_array);

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

/// クリップボードからテキストを読み取る
pub fn read_clipboard_text() -> Result<String, Error> {
  // AutoreleasePoolを作成
  let _pool = AutoreleasePool::new()?;

  // Pasteboardを取得
  let pasteboard = Pasteboard::general()
    .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to get general pasteboard"))?;

  // サポートするテキストタイプの優先順位リスト
  let supported_text_types = [
    "public.utf8-plain-text",
    "public.text",
    "NSStringPboardType",
    "com.apple.traditional-mac-plain-text",
  ];

  // 優先タイプを最初に試す
  for type_name in &supported_text_types {
    let type_str = match ObjcString::from_str(type_name) {
      Some(s) => s,
      None => continue,
    };

    // テキストデータを取得
    if let Some(text_obj) = pasteboard.string_for_type(&type_str) {
      if let Some(rust_str) = text_obj.to_rust_string() {
        if !rust_str.is_empty() {
          return Ok(rust_str);
        }
      }
    }
  }

  // 利用可能なタイプを取得して試す
  if let Some(types) = pasteboard.available_types() {
    let count = types.count();
    for i in 0..count {
      if let Some(type_id) = types.object_at_index(i) {
        if let Some(text_obj) = pasteboard.string_for_type_id(type_id) {
          if let Some(rust_str) = text_obj.to_rust_string() {
            if !rust_str.is_empty() {
              return Ok(rust_str);
            }
          }
        }
      }
    }
  }

  Err(Error::new(
    ErrorKind::NotFound,
    "No text content found on clipboard",
  ))
}

/// クリップボードからRAWデータを読み取る
pub fn read_clipboard_raw() -> Result<Vec<u8>, Error> {
  // AutoreleasePoolを作成
  let _pool = AutoreleasePool::new()?;

  // Pasteboardを取得
  let pasteboard = Pasteboard::general()
    .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to get general pasteboard"))?;

  // 利用可能なタイプを取得
  let types = pasteboard
    .available_types()
    .ok_or_else(|| Error::new(ErrorKind::NotFound, "Clipboard is empty"))?;

  if types.count() == 0 {
    return Err(Error::new(
      ErrorKind::NotFound,
      "Clipboard has no available types",
    ));
  }

  // サポートするタイプの優先順位リスト
  let supported_types = [
    "public.utf8-plain-text",
    "public.text",
    "NSStringPboardType",
    "public.data",
  ];

  // 優先タイプを最初に試す
  for type_name in &supported_types {
    let type_str = match ObjcString::from_str(type_name) {
      Some(s) => s,
      None => continue,
    };

    if let Some(data) = pasteboard.data_for_type(type_str.as_id()) {
      let obj_data = ObjcData::from_id(data);
      if let Some(bytes) = obj_data.to_bytes() {
        if !bytes.is_empty() {
          return Ok(bytes);
        }
      }
    }
  }

  // テキストデータを直接試す（データではなく文字列として）
  let text_type = ObjcString::from_str("public.utf8-plain-text");
  if let Some(text_type) = text_type {
    if let Some(text_obj) = pasteboard.string_for_type(&text_type) {
      if let Some(text) = text_obj.to_rust_string() {
        if !text.is_empty() {
          return Ok(text.into_bytes());
        }
      }
    }
  }

  // 利用可能なすべてのタイプを順番に試す
  let count = types.count();
  for i in 0..count {
    if let Some(type_id) = types.object_at_index(i) {
      if let Some(data) = pasteboard.data_for_type(type_id) {
        let obj_data = ObjcData::from_id(data);
        if let Some(bytes) = obj_data.to_bytes() {
          if !bytes.is_empty() {
            return Ok(bytes);
          }
        }
      }
    }
  }

  // ファイルパスが含まれている場合、それをテキストとして返す
  match read_clipboard_file_paths() {
    Ok(paths) => {
      if !paths.is_empty() {
        let joined = paths.join("\n");
        return Ok(joined.into_bytes());
      }
    }
    Err(_) => {} // ファイルパスの読み取りに失敗した場合は無視
  }

  // 何も見つからなかった場合はエラー
  Err(Error::new(
    ErrorKind::NotFound,
    "No data found in clipboard for any available type",
  ))
}

/// クリップボードからファイルパスを読み取る
pub fn read_clipboard_file_paths() -> Result<Vec<String>, Error> {
  // AutoreleasePoolを作成
  let _pool = AutoreleasePool::new()?;

  // Pasteboardを取得
  let pasteboard = Pasteboard::general()
    .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to get general pasteboard"))?;

  // NSURLクラスオブジェクトを取得
  let url_class: id = unsafe { msg_send![class!(NSURL), class] };

  // クラスフィルターの配列を作成
  let classes_array = ObjcArray::with_object(url_class)
    .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to create array for NSURL class"))?;

  // URLオブジェクトを読み取る
  let file_urls = pasteboard.read_objects_for_classes(&classes_array);

  let mut paths = Vec::new();

  if let Some(urls) = file_urls {
    let count = urls.count();

    for i in 0..count {
      if let Some(url_id) = urls.object_at_index(i) {
        let url = ObjcUrl { url: url_id };

        if url.is_file_url() {
          if let Some(path) = url.get_path() {
            paths.push(path);
          }
        }
      }
    }
  }

  // 空の場合でも空配列を返す（エラーにしない）
  Ok(paths)
}
