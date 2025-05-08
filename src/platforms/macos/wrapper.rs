#![cfg(target_os = "macos")]

use cocoa::appkit::NSPasteboard;
use cocoa::base::{id, nil};
use cocoa::foundation::NSUInteger;
use objc::{class, msg_send, sel, sel_impl};
use std::ffi::c_char;
use std::io::{Error, ErrorKind};

// ------------------------------------
// 安全なラッパー実装
// ------------------------------------

/// AutoreleasePoolのラッパー構造体
/// Dropトレイトを実装して、スコープを抜けたときに自動的にdrainを呼び出す
pub struct AutoreleasePool {
  pool: id,
}

impl AutoreleasePool {
  /// 新しいAutoReleasePoolを作成
  pub fn new() -> Result<Self, Error> {
    let pool = unsafe { msg_send![class!(NSAutoreleasePool), new] };
    if pool == nil {
      Err(Error::new(
        ErrorKind::Other,
        "Failed to create AutoreleasePool",
      ))
    } else {
      Ok(Self { pool })
    }
  }

  /// 内部poolをidとして取得
  #[allow(dead_code)]
  pub fn as_id(&self) -> id {
    self.pool
  }
}

impl Drop for AutoreleasePool {
  fn drop(&mut self) {
    unsafe {
      let _: () = msg_send![self.pool, drain];
    }
  }
}

/// NSStringのラッパー構造体
pub struct ObjcString {
  ns_string: id,
}

impl ObjcString {
  /// Rust文字列からNSStringを作成
  pub fn from_str(s: &str) -> Option<Self> {
    let alloc: id = unsafe { msg_send![class!(NSString), alloc] };
    let ns_string: id =
      unsafe { msg_send![alloc, initWithUTF8String: s.as_ptr() as *const c_char] };

    if ns_string != nil {
      Some(Self { ns_string })
    } else {
      None
    }
  }

  /// NSStringオブジェクトをidとして取得
  pub fn as_id(&self) -> id {
    self.ns_string
  }

  /// NSStringをRust文字列に変換
  pub fn to_rust_string(&self) -> Option<String> {
    let utf8_str: *const c_char = unsafe { msg_send![self.ns_string, UTF8String] };
    if !utf8_str.is_null() {
      unsafe {
        let rust_str = std::ffi::CStr::from_ptr(utf8_str)
          .to_string_lossy()
          .to_string();
        Some(rust_str)
      }
    } else {
      None
    }
  }
}

/// NSURLのラッパー構造体
pub struct ObjcUrl {
  pub url: id,
}

impl ObjcUrl {
  /// ファイルパスからNSURLを作成
  pub fn from_file_path(path_string: &ObjcString) -> Option<Self> {
    let url_class: id = unsafe { msg_send![class!(NSURL), class] };
    let url: id = unsafe { msg_send![url_class, fileURLWithPath:path_string.as_id()] };

    if url != nil {
      Some(Self { url })
    } else {
      None
    }
  }

  /// Rust文字列のパスから直接NSURLを作成
  pub fn from_path(path: &str) -> Option<Self> {
    ObjcString::from_str(path).and_then(|path_string| Self::from_file_path(&path_string))
  }

  /// URLがファイルURLかどうかを確認
  pub fn is_file_url(&self) -> bool {
    unsafe { msg_send![self.url, isFileURL] }
  }

  /// URLからファイルパスを取得
  pub fn get_path(&self) -> Option<String> {
    let path_string: id = unsafe { msg_send![self.url, path] };
    if path_string != nil {
      let utf8_str: *const c_char = unsafe { msg_send![path_string, UTF8String] };
      if !utf8_str.is_null() {
        unsafe {
          let path_str = std::ffi::CStr::from_ptr(utf8_str)
            .to_string_lossy()
            .to_string();
          Some(path_str)
        }
      } else {
        None
      }
    } else {
      None
    }
  }

  /// NSURLオブジェクトをidとして取得
  pub fn as_id(&self) -> id {
    self.url
  }
}

/// NSArrayのラッパー構造体
pub struct ObjcArray {
  array: id,
}

impl ObjcArray {
  /// idのベクターからNSArrayを作成
  pub fn from_vec(items: &[id]) -> Option<Self> {
    if items.is_empty() {
      return None;
    }

    let array: id = unsafe {
      msg_send![
          class!(NSArray),
          arrayWithObjects:items.as_ptr()
          count:items.len() as NSUInteger
      ]
    };

    if array != nil {
      Some(Self { array })
    } else {
      None
    }
  }

  /// 単一のオブジェクトを含むNSArrayを作成
  pub fn with_object(object: id) -> Option<Self> {
    let array: id = unsafe { msg_send![class!(NSArray), arrayWithObject:object] };

    if array != nil {
      Some(Self { array })
    } else {
      None
    }
  }

  /// 配列内の要素数を取得
  pub fn count(&self) -> NSUInteger {
    unsafe { msg_send![self.array, count] }
  }

  /// インデックスで要素を取得
  pub fn object_at_index(&self, index: NSUInteger) -> Option<id> {
    if index >= self.count() {
      return None;
    }

    let object: id = unsafe { msg_send![self.array, objectAtIndex:index] };
    if object != nil {
      Some(object)
    } else {
      None
    }
  }

  /// NSArrayオブジェクトをidとして取得
  pub fn as_id(&self) -> id {
    self.array
  }
}

/// NSPasteboardのラッパー構造体
pub struct Pasteboard {
  pasteboard: id,
}

impl Pasteboard {
  /// 一般的なシステムペーストボードを取得
  pub fn general() -> Option<Self> {
    let pasteboard: id = unsafe { NSPasteboard::generalPasteboard(nil) };
    if pasteboard != nil {
      Some(Self { pasteboard })
    } else {
      None
    }
  }

  /// ペーストボードの内容をクリア
  pub fn clear_contents(&self) -> NSUInteger {
    unsafe { msg_send![self.pasteboard, clearContents] }
  }

  /// ペーストボードにタイプを宣言
  pub fn declare_types(&self, types_array: &ObjcArray) -> bool {
    unsafe { msg_send![self.pasteboard, declareTypes:types_array.as_id() owner:nil] }
  }

  /// ペーストボードにオブジェクトを書き込む
  pub fn write_objects(&self, objects_array: &ObjcArray) -> bool {
    unsafe { msg_send![self.pasteboard, writeObjects:objects_array.as_id()] }
  }

  /// 指定されたクラスのオブジェクトをペーストボードから読み取る
  pub fn read_objects_for_classes(&self, class_array: &ObjcArray) -> Option<ObjcArray> {
    let objects: id = unsafe {
      msg_send![
          self.pasteboard,
          readObjectsForClasses:class_array.as_id()
          options:nil
      ]
    };

    if objects != nil {
      Some(ObjcArray { array: objects })
    } else {
      None
    }
  }

  /// ペーストボードから特定タイプの文字列を読み取る
  pub fn string_for_type(&self, type_string: &ObjcString) -> Option<ObjcString> {
    let string: id = unsafe { msg_send![self.pasteboard, stringForType:type_string.as_id()] };

    if string != nil {
      Some(ObjcString { ns_string: string })
    } else {
      None
    }
  }

  /// ペーストボードからidで指定された特定タイプの文字列を読み取る
  pub fn string_for_type_id(&self, type_id: id) -> Option<ObjcString> {
    let string: id = unsafe { msg_send![self.pasteboard, stringForType:type_id] };

    if string != nil {
      Some(ObjcString { ns_string: string })
    } else {
      None
    }
  }

  /// ペーストボードの利用可能なタイプを取得
  pub fn available_types(&self) -> Option<ObjcArray> {
    let types: id = unsafe { msg_send![self.pasteboard, types] };

    if types != nil {
      Some(ObjcArray { array: types })
    } else {
      None
    }
  }

  /// ペーストボードから特定タイプのデータを取得
  pub fn data_for_type(&self, type_id: id) -> Option<id> {
    let data: id = unsafe { msg_send![self.pasteboard, dataForType:type_id] };

    if data != nil {
      Some(data)
    } else {
      None
    }
  }

  /// NSPasteboardオブジェクトをidとして取得
  #[allow(dead_code)]
  pub fn as_id(&self) -> id {
    self.pasteboard
  }
}

/// NSDataのラッパー構造体
pub struct ObjcData {
  data: id,
}

impl ObjcData {
  /// idからObjcDataを作成
  pub fn from_id(data_id: id) -> Self {
    Self { data: data_id }
  }

  /// データの長さを取得
  pub fn length(&self) -> NSUInteger {
    unsafe { msg_send![self.data, length] }
  }

  /// データをバイト配列に変換
  pub fn to_bytes(&self) -> Option<Vec<u8>> {
    let length = self.length();
    let bytes: *const u8 = unsafe { msg_send![self.data, bytes] };

    if bytes.is_null() {
      return None;
    }

    unsafe {
      let slice = std::slice::from_raw_parts(bytes, length as usize);
      Some(slice.to_vec())
    }
  }

  /// NSDataオブジェクトをidとして取得
  #[allow(dead_code)]
  pub fn as_id(&self) -> id {
    self.data
  }
}
