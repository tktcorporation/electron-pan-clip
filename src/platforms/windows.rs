#![cfg(target_os = "windows")]

use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::{Error, ErrorKind};
use std::iter::once;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::ptr;
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, HWND, POINT};
use windows::Win32::Storage::FileSystem::{DragQueryFileW, HDROP};
use windows::Win32::System::DataExchange::CF_HDROP;
use windows::Win32::System::DataExchange::CF_UNICODETEXT;
use windows::Win32::System::DataExchange::{
  CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
  RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{
  GlobalAlloc, GlobalFree, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
};

// Windows用のDROPFILES構造体定義
#[repr(C)]
struct DROPFILES {
  pFiles: u32,                             // ファイル名へのオフセット
  pt: POINT,                               // ドロップポイント
  fNC: windows::Win32::Foundation::BOOL,   // クライアント領域外かどうか
  fWide: windows::Win32::Foundation::BOOL, // Unicodeかどうか
}

// ワイド文字列（UTF-16）に変換し、NULL終端を追加するヘルパー関数
fn to_wide_null(s: &str) -> Vec<u16> {
  OsStr::new(s).encode_wide().chain(once(0)).collect()
}

pub fn copy_files_to_clipboard(paths: &[String]) -> Result<(), Error> {
  // クリップボードを開く
  if OpenClipboard(HWND(0)) == false {
    return Err(Error::new(
      ErrorKind::Other,
      format!("Failed to open clipboard: {:?}", GetLastError()),
    ));
  }

  // クリップボードをクリア
  if EmptyClipboard() == false {
    CloseClipboard();
    return Err(Error::new(
      ErrorKind::Other,
      format!("Failed to empty clipboard: {:?}", GetLastError()),
    ));
  }

  // CF_HDROP形式のデータを作成する
  let mut total_size = std::mem::size_of::<DROPFILES>() + 2; // ヘッダサイズ + 終端用の2バイト

  // 各ファイルパスのサイズを計算
  let mut wide_paths = Vec::new();
  for path in paths {
    // 絶対パスに変換
    let abs_path = match Path::new(path).canonicalize() {
      Ok(p) => p,
      Err(e) => {
        CloseClipboard();
        return Err(Error::new(
          ErrorKind::InvalidInput,
          format!("Failed to canonicalize path {}: {}", path, e),
        ));
      }
    };

    // パスを表すワイド文字列を作成
    let wide_path = match abs_path.to_str() {
      Some(s) => OsString::from(s),
      None => {
        CloseClipboard();
        return Err(Error::new(
          ErrorKind::InvalidInput,
          format!("Path contains invalid characters: {:?}", abs_path),
        ));
      }
    };

    let wide_path: Vec<u16> = wide_path.encode_wide().chain(Some(0)).collect();
    total_size += wide_path.len() * 2;
    wide_paths.push(wide_path);
  }

  // メモリを確保
  let hmem = GlobalAlloc(GMEM_MOVEABLE, total_size);
  if hmem.is_invalid() {
    CloseClipboard();
    return Err(Error::new(
      ErrorKind::Other,
      format!("Failed to allocate memory: {:?}", GetLastError()),
    ));
  }

  // メモリをロック
  let ptr = GlobalLock(hmem);
  if ptr.is_null() {
    GlobalFree(hmem);
    CloseClipboard();
    return Err(Error::new(
      ErrorKind::Other,
      format!("Failed to lock memory: {:?}", GetLastError()),
    ));
  }

  // DROPFILES構造体を初期化
  let drop_files = ptr as *mut DROPFILES;
  (*drop_files).pFiles = std::mem::size_of::<DROPFILES>() as u32;
  (*drop_files).pt = POINT { x: 0, y: 0 };
  (*drop_files).fNC = false.into();
  (*drop_files).fWide = true.into(); // Unicode (wide char)を使用

  // パスをコピー
  let mut dest = (ptr as usize + std::mem::size_of::<DROPFILES>()) as *mut u16;
  for path in wide_paths {
    std::ptr::copy_nonoverlapping(path.as_ptr(), dest, path.len());
    dest = dest.add(path.len());
  }
  // 最後にダブルヌル終端を追加
  *dest = 0;

  // メモリをアンロック
  GlobalUnlock(hmem);

  // クリップボードにデータをセット
  let result = SetClipboardData(CF_HDROP.0 as u32, hmem);
  if result.is_invalid() {
    GlobalFree(hmem);
    CloseClipboard();
    return Err(Error::new(
      ErrorKind::Other,
      format!("Failed to set clipboard data: {:?}", GetLastError()),
    ));
  }

  // クリップボードを閉じる
  CloseClipboard();

  Ok(())
}

// クリップボードからテキストを読み取る
pub fn read_clipboard_text() -> Result<String, Error> {
  unsafe {
    // クリップボードを開く
    if OpenClipboard(HWND(0)) == false {
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {:?}", GetLastError()),
      ));
    }

    // CF_UNICODETEXTフォーマットが利用可能か確認
    if IsClipboardFormatAvailable(CF_UNICODETEXT.0 as u32) == false {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "No text available in clipboard",
      ));
    }

    // クリップボードからデータを取得
    let handle = GetClipboardData(CF_UNICODETEXT.0 as u32);
    if handle.is_invalid() {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to get clipboard data: {:?}", GetLastError()),
      ));
    }

    // メモリをロック
    let ptr = GlobalLock(handle) as *const u16;
    if ptr.is_null() {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to lock memory: {:?}", GetLastError()),
      ));
    }

    // ワイド文字列をRust文字列に変換
    // ヌル終端の文字列の長さを計算
    let mut len = 0;
    while *ptr.add(len) != 0 {
      len += 1;
    }

    let wstr = std::slice::from_raw_parts(ptr, len);
    let text = String::from_utf16_lossy(wstr);

    // メモリをアンロック
    GlobalUnlock(handle);

    // クリップボードを閉じる
    CloseClipboard();

    Ok(text)
  }
}

// クリップボードからRAWデータを読み取る
pub fn read_clipboard_raw() -> Result<Vec<u8>, Error> {
  unsafe {
    // クリップボードを開く
    if OpenClipboard(HWND(0)) == false {
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {:?}", GetLastError()),
      ));
    }

    // 利用可能なフォーマットを調べる - 最初の利用可能なフォーマットを使用
    let format_id = CF_UNICODETEXT.0 as u32; // テキストをデフォルトとして使用

    // クリップボードからデータを取得
    let handle = GetClipboardData(format_id);
    if handle.is_invalid() {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to get clipboard data: {:?}", GetLastError()),
      ));
    }

    // メモリをロック
    let ptr = GlobalLock(handle);
    if ptr.is_null() {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to lock memory: {:?}", GetLastError()),
      ));
    }

    // グローバルメモリのサイズを取得
    let size = windows::Win32::System::Memory::GlobalSize(handle);

    // バイトデータをコピー
    let data = if size > 0 {
      let slice = std::slice::from_raw_parts(ptr as *const u8, size);
      slice.to_vec()
    } else {
      Vec::new()
    };

    // メモリをアンロック
    GlobalUnlock(handle);

    // クリップボードを閉じる
    CloseClipboard();

    if data.is_empty() {
      Err(Error::new(
        ErrorKind::Other,
        "No data available in clipboard",
      ))
    } else {
      Ok(data)
    }
  }
}

// クリップボードからファイルパスを読み取る
pub fn read_clipboard_file_paths() -> Result<Vec<String>, Error> {
  // クリップボードを開く
  if OpenClipboard(HWND(0)) == false {
    return Err(Error::new(
      ErrorKind::Other,
      format!("Failed to open clipboard: {:?}", GetLastError()),
    ));
  }

  // CF_HDROPフォーマットが利用可能か確認
  if IsClipboardFormatAvailable(CF_HDROP.0 as u32) == false {
    CloseClipboard();
    return Err(Error::new(
      ErrorKind::Other,
      "No file paths available in clipboard",
    ));
  }

  // クリップボードからデータを取得
  let hdrop = GetClipboardData(CF_HDROP.0 as u32);
  if hdrop.is_invalid() {
    CloseClipboard();
    return Err(Error::new(
      ErrorKind::Other,
      format!("Failed to get clipboard data: {:?}", GetLastError()),
    ));
  }

  // ファイル数を取得
  let file_count = DragQueryFileW(HDROP(hdrop.0), 0xFFFFFFFF, None);

  let mut paths = Vec::new();

  // 各ファイルパスを取得
  for i in 0..file_count {
    // 必要なバッファサイズを取得
    let size = DragQueryFileW(HDROP(hdrop.0), i, None) + 1;

    // バッファを確保
    let mut buffer = vec![0u16; size as usize];

    // ファイル名を取得
    let length = DragQueryFileW(HDROP(hdrop.0), i, Some(&mut buffer));

    if length > 0 {
      // 実際の長さに合わせてバッファをトリミング
      buffer.truncate(length as usize);

      // UTF-16文字列をRust文字列に変換
      let path = String::from_utf16_lossy(&buffer);
      paths.push(path);
    }
  }

  // クリップボードを閉じる
  CloseClipboard();

  if paths.is_empty() {
    Err(Error::new(
      ErrorKind::Other,
      "No valid file paths found in clipboard",
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
  use std::path::Path;

  // UTF-16エンコードのテスト
  #[test]
  fn test_to_wide_null() {
    let test_str = "Hello, Windows!";
    let wide = to_wide_null(test_str);

    // NULL終端文字を含むため、長さは元の文字列+1になる
    assert_eq!(wide.len(), test_str.len() + 1);

    // 最後の文字は NULL (0) であることを確認
    assert_eq!(wide[wide.len() - 1], 0);

    // 各文字がUTF-16エンコードされていることを確認
    for (i, c) in test_str.chars().enumerate() {
      assert_eq!(wide[i], c as u16);
    }
  }

  // パスの変換テスト
  #[test]
  fn test_path_conversion() {
    // Windowsパスのテスト
    let test_paths = vec![
      "C:\\Windows\\System32".to_string(),
      "D:\\Documents\\file.txt".to_string(),
    ];

    // パスをワイド文字列に変換
    let mut wide_paths: Vec<u16> = test_paths
      .iter()
      .map(|p| to_wide_null(p))
      .flatten()
      .collect();
    wide_paths.push(0); // ダブルNULL終端

    // 各パスのNULL区切りを確認
    let mut null_count = 0;
    for c in wide_paths.iter() {
      if *c == 0 {
        null_count += 1;
      }
    }

    // パスの数 + ダブルNULL用の追加1個 = NULL文字の総数
    assert_eq!(null_count, test_paths.len() + 1);
  }

  // 実際のクリップボード操作テスト
  #[test]
  #[ignore]
  fn test_copy_real_files() {
    // 一時ファイルを作成
    let tmp_dir = temp_dir();
    let test_file_path = tmp_dir.join("test_windows_clipboard.txt");

    // ファイルを作成
    let _ = File::create(&test_file_path).expect("Failed to create test file");

    // パスを文字列に変換
    let path_str = test_file_path.to_string_lossy().to_string();

    // クリップボードにコピー
    let result = copy_files_to_clipboard(&[path_str]);
    assert!(result.is_ok(), "Failed to copy files: {:?}", result);

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);
  }

  // 空のパスリストのテスト
  // Windowsの実装では空のリストでもメモリ確保などは行うが、
  // エラーにはならない。ただし実用上は空リスト前にチェックする方が良い
  #[test]
  fn test_empty_paths() {
    let result = copy_files_to_clipboard(&[]);
    // この実装では空リストでもエラーにはならない
    // 注: lib.rs側で空チェックを行っているため、通常は到達しない
    assert!(result.is_ok());
  }
}
