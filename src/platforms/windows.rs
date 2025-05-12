#![cfg(target_os = "windows")]

use std::ffi::c_void;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::{Error, ErrorKind};
use std::iter::once;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::{
  Foundation::{GetLastError, HWND},
  System::{
    DataExchange::{
      CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
      SetClipboardData,
    },
    Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE},
  },
  UI::Shell::DragQueryFileW,
};

// シェルフォーマットの定数
const CF_HDROP: u32 = 15;

// Shell.h から DROPFILES 構造体を定義
#[repr(C)]
#[allow(non_snake_case)]
struct DROPFILES {
  pFiles: u32,
  pt: windows_sys::Win32::Foundation::POINT,
  fNC: windows_sys::Win32::Foundation::BOOL,
  fWide: windows_sys::Win32::Foundation::BOOL,
}

// ワイド文字列（UTF-16）に変換し、NULL終端を追加するヘルパー関数
fn to_wide_null(s: &str) -> Vec<u16> {
  OsStr::new(s).encode_wide().chain(once(0)).collect()
}

// HGLOBAL用のGlobalFree関数（Kernel32.dllから直接インポート）
#[link(name = "kernel32")]
extern "system" {
  fn GlobalFree(hMem: *mut c_void) -> *mut c_void;
}

/// ファイルパスをクリップボードにコピーする
pub fn write_clipboard_file_paths(paths: &[String]) -> Result<(), Error> {
  unsafe {
    // クリップボードを開く (所有者を指定しない場合は NULL)
    if OpenClipboard(0) == 0 {
      let err = GetLastError();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {}", err),
      ));
    }

    // クリップボードを空にする
    if EmptyClipboard() == 0 {
      let err = GetLastError();
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to empty clipboard: {}", err),
      ));
    }

    // 空の配列の場合はクリップボードをクリアして終了
    if paths.is_empty() {
      CloseClipboard();
      println!("Cleared clipboard data (empty file list)");
      return Ok(());
    }

    // 1. パスリストをワイド文字列（UTF-16）に変換し、ダブルNULL終端形式にする
    let mut wide_paths: Vec<u16> = paths
      .iter()
      .map(|p| to_wide_null(p)) // 各パスを NULL 終端 UTF-16 に変換
      .flatten() // 平坦化して連結
      .collect();
    wide_paths.push(0); // リストの最後に NULL を追加してダブルNULL終端にする

    // 2. DROPFILES 構造体とパスリストを格納するためのメモリサイズを計算
    let dropfiles_size = std::mem::size_of::<DROPFILES>();
    let paths_size = wide_paths.len() * std::mem::size_of::<u16>();
    let total_size = dropfiles_size + paths_size;

    // 3. グローバルメモリを確保
    // CF_HDROP は GMEM_MOVEABLE である必要がある
    let h_global = GlobalAlloc(GMEM_MOVEABLE, total_size);
    if h_global == ptr::null_mut() {
      let err = GetLastError();
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to allocate global memory: {}", err),
      ));
    }

    // 4. メモリをロックしてポインタを取得
    let buffer_ptr = GlobalLock(h_global) as *mut u8;
    if buffer_ptr.is_null() {
      let err = GetLastError();
      // GlobalFree(h_global); // windows-sys 0.52 では GlobalFree が直接使えないので代わりに Windows API を直接呼び出す
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to lock global memory: {}", err),
      ));
    }

    // 5. DROPFILES 構造体をメモリに書き込む
    let dropfiles_ptr = buffer_ptr as *mut DROPFILES;
    *dropfiles_ptr = DROPFILES {
      pFiles: dropfiles_size as u32, // パスリストへのオフセット
      pt: zeroed(),                  // 使わないのでゼロ初期化
      fNC: 0,                        // 非クライアント領域座標ではない
      fWide: 1,                      // ワイド文字（UTF-16）を使用
    };

    // 6. パスリスト（ダブルNULL終端）を DROPFILES 構造体の直後に書き込む
    let paths_ptr = buffer_ptr.add(dropfiles_size);
    ptr::copy_nonoverlapping(wide_paths.as_ptr() as *const u8, paths_ptr, paths_size);

    // 7. メモリをアンロック
    GlobalUnlock(h_global);

    // 10. CF_HDROP 形式でデータをクリップボードに設定
    // SetClipboardData が成功すると、OS がメモリの所有権を持つため、GlobalFree を呼んではいけない
    if SetClipboardData(CF_HDROP, h_global as isize) == 0 {
      let err = GetLastError();
      CloseClipboard();
      // GlobalFree(h_global); // GlobalFree が直接使えない
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to set clipboard data: {}", err),
      ));
    }

    // 11. クリップボードを閉じる
    if CloseClipboard() == 0 {
      // この時点ではデータは設定されているが、閉じるのに失敗した
      // エラーとして報告するべきか？ 일단 ここでは警告としておく
      eprintln!("Warning: Failed to close clipboard: {}", GetLastError());
    }

    println!("Copied {} files to clipboard on Windows", paths.len());
    Ok(())
  } // unsafe ブロック終了
}

// クリップボードからテキストを読み取る
pub fn read_clipboard_text() -> Result<String, Error> {
  unsafe {
    // クリップボードを開く
    if OpenClipboard(0) == 0 {
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {:?}", GetLastError()),
      ));
    }

    // CF_UNICODETEXTフォーマットが利用可能か確認
    if IsClipboardFormatAvailable(13) == 0 {
      // CF_UNICODETEXT = 13
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "No text available in clipboard",
      ));
    }

    // クリップボードからデータを取得
    let handle = GetClipboardData(13); // CF_UNICODETEXT = 13
    if handle == 0 {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to get clipboard data: {:?}", GetLastError()),
      ));
    }

    // メモリをロック
    let ptr = GlobalLock(handle as *mut c_void) as *const u16;
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
    GlobalUnlock(handle as *mut c_void);

    // クリップボードを閉じる
    CloseClipboard();

    Ok(text)
  }
}

// クリップボードからRAWデータを読み取る
pub fn read_clipboard_raw() -> Result<Vec<u8>, Error> {
  unsafe {
    // クリップボードを開く
    if OpenClipboard(0) == 0 {
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {:?}", GetLastError()),
      ));
    }

    // 利用可能なフォーマットを調べる - 最初の利用可能なフォーマットを使用
    let format_id = 13; // CF_UNICODETEXT = 13

    // クリップボードからデータを取得
    let handle = GetClipboardData(format_id);
    if handle == 0 {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to get clipboard data: {:?}", GetLastError()),
      ));
    }

    // メモリをロック
    let ptr = GlobalLock(handle as *mut c_void);
    if ptr.is_null() {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to lock memory: {:?}", GetLastError()),
      ));
    }

    // グローバルメモリのサイズを取得
    let size = GlobalSize(handle as *mut c_void);

    // バイトデータをコピー
    let data = if size > 0 {
      let slice = std::slice::from_raw_parts(ptr as *const u8, size);
      slice.to_vec()
    } else {
      Vec::new()
    };

    // メモリをアンロック
    GlobalUnlock(handle as *mut c_void);

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
  unsafe {
    // クリップボードを開く
    if OpenClipboard(0) == 0 {
      return Err(Error::new(
        ErrorKind::Other,
        format!("Failed to open clipboard: {:?}", GetLastError()),
      ));
    }

    // CF_HDROPフォーマットが利用可能か確認
    if IsClipboardFormatAvailable(CF_HDROP) == 0 {
      CloseClipboard();
      return Err(Error::new(
        ErrorKind::Other,
        "No file paths available in clipboard",
      ));
    }

    // クリップボードからデータを取得
    let hdrop = GetClipboardData(CF_HDROP);
    if hdrop == 0 {
      CloseClipboard();
      return Err(Error::new(ErrorKind::Other, "Failed to get clipboard data"));
    }

    // ファイル数を取得 (0xFFFFFFFFを指定すると、ファイル数が返る)
    let file_count = DragQueryFileW(hdrop as isize, 0xFFFFFFFF, ptr::null_mut(), 0);

    let mut paths = Vec::new();

    // 各ファイルパスを取得
    for i in 0..file_count {
      // 必要なバッファサイズを取得
      let size = DragQueryFileW(hdrop as isize, i, ptr::null_mut(), 0) + 1;

      // バッファを確保
      let mut buffer = vec![0u16; size as usize];

      // ファイル名を取得
      let length = DragQueryFileW(hdrop as isize, i, buffer.as_mut_ptr(), size);

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
    let result = write_clipboard_file_paths(&[path_str]);
    assert!(result.is_ok(), "Failed to copy files: {:?}", result);

    // テスト後にファイルを削除
    let _ = std::fs::remove_file(test_file_path);
  }

  // 空のパスリストのテスト
  // Windowsの実装では空のリストでもメモリ確保などは行うが、
  // エラーにはならない。ただし実用上は空リスト前にチェックする方が良い
  #[test]
  fn test_empty_paths() {
    let result = write_clipboard_file_paths(&[]);
    // この実装では空リストでもエラーにはならない
    // 注: lib.rs側で空チェックを行っているため、通常は到達しない
    assert!(result.is_ok());
  }
}
